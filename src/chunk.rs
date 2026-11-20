//! Structure-aware splitting of memory bodies for embedding.
//!
//! A memory body is agent-written Markdown. Long bodies cannot be embedded in one piece (the
//! model window cuts the tail away silently), so they are split at structural boundaries instead:
//! ATX headings define sections, blank lines define blocks, and fenced code blocks are atomic —
//! a `#` inside a fence is content, never a heading. Sections are then packed into chunks under a
//! token budget measured with the model's own tokenizer, so the cut is exactly what the model sees.
//!
//! Each chunk carries a header line with its section path (`部署 > 回滚步骤`), the cheap form of
//! contextual retrieval: the path is already in the text, so no extra model call is needed to tell
//! the embedder where the passage sits. Bodies that fit the budget stay a single chunk with no
//! header line — identical to the un-chunked text, so short memories embed exactly as before.

use std::collections::VecDeque;

use tokenizers::Tokenizer;

/// Target size of one chunk, in tokens. Retrieval quality favours focused passages; 512 keeps a
/// chunk small enough to stay on-topic while large enough to hold a full section.
pub const CHUNK_TOKENS: usize = 512;

/// Upper bound on chunks per memory. Beyond this the target grows so the body still fits the
/// model window instead of being cut: `CHUNK_TOKENS * MAX_CHUNKS` covers the long-context class.
pub const MAX_CHUNKS: usize = 16;

/// A block never shrinks below this many tokens when an overlong block has to be split further —
/// except when the model window itself is smaller.
const MIN_PIECE_TOKENS: usize = 32;

/// Version tag of the split layout, part of the vector cache fingerprint: changing how bodies are
/// cut (boundaries, header form, packing) changes every chunk embedding, so it must invalidate.
pub const POLICY_TAG: &str = "chunk-v1";

/// One structural block: a run of non-blank lines (a paragraph, a list, a whole fenced code
/// block) together with the heading path it sits under.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub path: Vec<String>,
    pub text: String,
}

/// Split a Markdown body into blocks. Fence-aware: everything between matching ``` / ~~~ fences is
/// one block regardless of blank lines, and never contributes headings.
pub fn blocks(content: &str) -> Vec<Block> {
    let mut out = Vec::new();
    // Heading stack: (level, title). Absolute levels matter, not the stack depth — a body may
    // start at `##`, and a sibling at the same level must still pop its predecessor.
    let mut path: Vec<(usize, String)> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    // (fence char, fence length) while inside a fenced code block
    let mut fence: Option<(char, usize)> = None;

    let flush = |current: &mut Vec<&str>, path: &[(usize, String)], out: &mut Vec<Block>| {
        while current.last().is_some_and(|l| l.trim().is_empty()) {
            current.pop();
        }
        if current.is_empty() {
            return;
        }
        let text = current.join("\n");
        if !text.trim().is_empty() {
            out.push(Block {
                path: path.iter().map(|(_, t)| t.clone()).collect(),
                text,
            });
        }
        current.clear();
    };

    for line in content.lines() {
        match fence {
            Some((marker, len)) => {
                current.push(line);
                if closes_fence(line, marker, len) {
                    fence = None;
                }
            }
            None => {
                if let Some((marker, len, _)) = opens_fence(line) {
                    flush(&mut current, &path, &mut out);
                    fence = Some((marker, len));
                    current.push(line);
                } else if let Some((level, title)) = heading(line) {
                    flush(&mut current, &path, &mut out);
                    while path.last().is_some_and(|(l, _)| *l >= level) {
                        path.pop();
                    }
                    path.push((level, title));
                } else if line.trim().is_empty() {
                    flush(&mut current, &path, &mut out);
                } else {
                    current.push(line);
                }
            }
        }
    }
    flush(&mut current, &path, &mut out);
    out
}

/// An ATX heading: up to three leading spaces, 1–6 `#`, then a space or end of line.
fn heading(line: &str) -> Option<(usize, String)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let level = rest.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let after = &rest[level..];
    if !after.is_empty() && !after.starts_with(' ') {
        return None;
    }
    let title = after.trim();
    // `#` alone is an empty heading: it still opens a section level, without a name to show.
    Some((level, title.to_string()))
}

/// A fence opener: ``` or ~~~ (three or more), optionally with an info string.
fn opens_fence(line: &str) -> Option<(char, usize, String)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    let rest = &line[indent.min(3)..];
    let marker = rest.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let len = rest.chars().take_while(|c| *c == marker).count();
    if len < 3 {
        return None;
    }
    let info = rest[len..].trim().to_string();
    // A backtick fence's info string cannot contain backticks (CommonMark), which also keeps
    // inline code spans from opening a block.
    if marker == '`' && info.contains('`') {
        return None;
    }
    Some((marker, len, info))
}

fn closes_fence(line: &str, marker: char, len: usize) -> bool {
    let rest = line.trim_start();
    let count = rest.chars().take_while(|c| *c == marker).count();
    count >= len && rest[count..].trim().is_empty()
}

/// The single chunk every body used to produce: title, blank line, body.
pub fn plain_text(summary: &str, content: &str) -> String {
    let mut text = summary.trim().to_string();
    if !content.trim().is_empty() {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(content.trim());
    }
    text
}

fn header_line(path: &[String]) -> String {
    path.iter()
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" > ")
}

fn count(tok: &Tokenizer, text: &str) -> usize {
    tok.encode(text, false)
        .map(|e| e.get_ids().len())
        .unwrap_or(0)
}

/// Split one body into embeddable chunks of at most `limit` tokens each.
///
/// `Ok(vec![plain_text(..)])` when the whole body fits the target: no headers are added, so the
/// embedding input matches the single-vector form exactly. Otherwise blocks that share a section
/// are packed together up to the target, a section change starts a new chunk, and blocks too large
/// for a chunk on their own are cut at token boundaries (never inside a fence's semantics — the
/// cut is textual, the fence markers still surround each piece in order).
pub fn chunk_texts(tok: &Tokenizer, summary: &str, content: &str, limit: usize) -> Vec<String> {
    let limit = limit.max(1);
    let plain = plain_text(summary, content);
    let target = CHUNK_TOKENS.clamp(1, limit);
    let total = count(tok, &plain);
    if total <= target {
        return vec![plain];
    }

    // More sections than MAX_CHUNKS: grow the target so the body still lands in the window, and
    // allow chunks to span sections (a section-per-chunk layout is better when it fits).
    let effective = if total > target.saturating_mul(MAX_CHUNKS) {
        total.div_ceil(MAX_CHUNKS).clamp(target, limit)
    } else {
        target
    };

    let sections = blocks(content);
    let mut pieces: VecDeque<Piece> = VecDeque::new();
    for block in &sections {
        let header = header_line(&block.path);
        let overhead = count(tok, &format!("{summary}\n\n{header}\n\n"));
        let budget = effective
            .saturating_sub(overhead)
            .max(MIN_PIECE_TOKENS.min(effective));
        for text in split_block(tok, &block.text, budget) {
            pieces.push_back(Piece {
                path: block.path.clone(),
                text,
            });
        }
    }

    let mut chunks = pack(tok, summary, pieces.clone(), effective, true);
    if chunks.len() > MAX_CHUNKS {
        chunks = pack(tok, summary, pieces, effective, false);
    }
    if chunks.len() == 1 {
        // A single chunk is the un-chunked body by another route: keep the plain form.
        return vec![plain];
    }
    chunks
}

#[derive(Clone, Debug)]
struct Piece {
    path: Vec<String>,
    text: String,
}

/// Cut one block into pieces of at most `budget` tokens (token-level cut, decoded back to text).
fn split_block(tok: &Tokenizer, text: &str, budget: usize) -> Vec<String> {
    let budget = budget.max(1);
    let Ok(enc) = tok.encode(text, false) else {
        return vec![text.to_string()];
    };
    let ids = enc.get_ids();
    if ids.len() <= budget {
        return vec![text.to_string()];
    }
    ids.chunks(budget)
        .filter_map(|part| tok.decode(part, true).ok())
        .filter(|s| !s.trim().is_empty())
        .collect()
}

/// Greedy packing. With `respect_sections`, a chunk never spans two heading paths.
fn pack(
    tok: &Tokenizer,
    summary: &str,
    pieces: VecDeque<Piece>,
    target: usize,
    respect_sections: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Vec<Piece> = Vec::new();
    for piece in pieces {
        let starts_new_section =
            respect_sections && current.last().is_some_and(|prev| prev.path != piece.path);
        let candidate = render(summary, &current, Some(&piece));
        let overflow = count(tok, &candidate) > target;
        if !current.is_empty() && (starts_new_section || overflow) {
            out.push(render(summary, &current, None));
            current.clear();
        }
        current.push(piece);
    }
    if !current.is_empty() {
        out.push(render(summary, &current, None));
    }
    out
}

/// Render one chunk: title, the section path of its first piece as a header line, then the bodies.
fn render(summary: &str, pieces: &[Piece], next: Option<&Piece>) -> String {
    let all = || pieces.iter().chain(next);
    let path = all().next().map(|p| p.path.clone()).unwrap_or_default();
    let header = header_line(&path);
    let mut text = summary.trim().to_string();
    if !header.is_empty() {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(&header);
    }
    for piece in all() {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(piece.text.trim());
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tokenizer fixture shared with `tokenizer` tests: whitespace pre-tokenization with whole
    /// words in the vocabulary, so one word = one token and counts are easy to reason about.
    fn tok() -> Tokenizer {
        crate::tokenizer::test_tokenizer()
    }

    #[test]
    fn headings_build_a_section_path_and_fences_are_atomic() {
        let md =
            "# Deploy\n\nstep one\n\n```sh\n# not a heading\nsleep 1\n```\n\n## Rollback\n\nundo\n";
        let blocks = blocks(md);
        let paths: Vec<Vec<String>> = blocks.iter().map(|b| b.path.clone()).collect();
        assert_eq!(
            paths,
            vec![
                vec!["Deploy".to_string()],
                vec!["Deploy".to_string()],
                vec!["Deploy".to_string(), "Rollback".to_string()],
            ],
            "the fence stays one block under its section, its `#` is not a heading"
        );
        assert!(blocks[1].text.contains("# not a heading"));
        assert!(blocks[1].text.starts_with("```"));
    }

    #[test]
    fn sibling_heading_pops_the_stack() {
        let md = "## A\n\na\n\n## B\n\nb\n\n### C\n\nc\n\n## D\n\nd";
        let paths: Vec<String> = blocks(md).iter().map(|b| b.path.join("/")).collect();
        assert_eq!(paths, vec!["A", "B", "B/C", "D"]);
    }

    #[test]
    fn blank_lines_split_blocks_outside_fences_only() {
        let md = "para one\n\n~~~\nkeep\n\nblank\n~~~\n\nafter";
        let blocks = blocks(md);
        assert_eq!(blocks.len(), 3);
        assert!(blocks[1].text.contains("keep\n\nblank"));
    }

    #[test]
    fn short_body_is_one_chunk_without_headers() {
        let text = chunk_texts(&tok(), "title", "hello world", 4096);
        assert_eq!(text, vec!["title\n\nhello world".to_string()]);
    }

    #[test]
    fn long_body_is_split_on_sections_and_each_chunk_names_its_path() {
        let body = format!(
            "# deploy\n\n{}\n\n# rollback\n\n{}",
            "alpha ".repeat(400),
            "beta ".repeat(400)
        );
        let chunks = chunk_texts(&tok(), "ops", &body, 4096);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.starts_with("ops\n\n")));
        assert!(
            chunks[0].contains("deploy"),
            "first chunk names its section"
        );
        assert!(
            chunks.last().unwrap().contains("rollback"),
            "the tail section is reachable — the point of chunking"
        );
        // Every chunk stays inside the window.
        for chunk in &chunks {
            assert!(count(&tok(), chunk) <= 4096);
        }
    }

    #[test]
    fn chunk_count_is_bounded_and_sections_merge_when_too_many() {
        // 40 sections of ~40 tokens: section-per-chunk would exceed MAX_CHUNKS, so the packer is
        // allowed to merge across sections.
        let body: String = (0..40)
            .map(|i| format!("## s{i}\n\n{}\n\n", "alpha beta gamma delta ".repeat(10)))
            .collect();
        let chunks = chunk_texts(&tok(), "t", &body, 4096);
        assert!(chunks.len() <= MAX_CHUNKS, "got {}", chunks.len());
        assert!(chunks.len() > 1);
    }

    #[test]
    fn overlong_block_is_cut_at_token_boundaries() {
        let body = format!("# one\n\n{}", "alpha ".repeat(2000));
        let chunks = chunk_texts(&tok(), "t", &body, 4096);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(count(&tok(), chunk) <= 4096);
        }
    }

    #[test]
    fn window_smaller_than_default_target_is_respected() {
        let body = "alpha ".repeat(200);
        let chunks = chunk_texts(&tok(), "t", &body, 64);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(count(&tok(), chunk) <= 64, "chunk exceeds the model window");
        }
    }
}
