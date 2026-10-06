//! Keyword search: whitespace tokenization (quoted phrases become single terms), all-terms matching (AND), weighted scoring,
//! and snippet generation.
//!
//! Matching is substring lookup over case-folded text, so Chinese hits directly by substring with no word segmentation.
//! Weights are the constants below: exact tag > tag substring > title > content; title/content score per hit
//! (capped at 3 hits), pure-ASCII words get a bonus for word-boundary hits, and quoted phrases
//! get a further bonus for whole-phrase hits. Among candidate positions, the snippet window chosen is the one covering the most distinct terms.

use crate::model::Memory;

/// Weight: exact tag-name hit (accumulated per matching tag).
const W_TAG_EXACT: i64 = 40;
/// Weight: tag-name substring hit (accumulated per matching tag).
const W_TAG_SUBSTR: i64 = 25;
/// Weight: title hit (each one; capped at MAX_TF_HITS).
const W_SUMMARY: i64 = 10;
/// Weight: content hit (each one; capped at MAX_TF_HITS).
const W_CONTENT: i64 = 3;
/// Weight: pure-ASCII word hit at a word boundary (once each for title/content).
/// Substring matching is a feature for Chinese ("记忆" hits "记忆库") but a hazard for ASCII
/// (searching "age" hits "message"); the word-boundary bonus ranks whole-word hits first without narrowing recall.
const W_WORD_SUMMARY: i64 = 5;
const W_WORD_CONTENT: i64 = 2;
/// Weight: whole quoted-phrase hit (title or content).
const W_PHRASE: i64 = 8;
/// Hit-count scoring cap: no further points beyond it, so long documents cannot steamroll field weights.
const MAX_TF_HITS: usize = 3;

pub struct Hit {
    pub idx: usize,
    pub score: i64,
    pub snippet: String,
}

const SNIPPET_BEFORE: usize = 40;
const SNIPPET_AFTER: usize = 120;

/// Split the query: whitespace tokenization; whitespace inside double quotes does not split (the phrase becomes one term).
/// All terms are lowercased.
fn parse_query(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in query.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                terms.push(std::mem::take(&mut current).to_lowercase());
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        terms.push(current.to_lowercase());
    }
    terms
}

/// Keyword search over memories. Tag filtering lives one layer up (the tag expression narrows
/// the ranked candidates in the tool handler), so both recall channels stay filter-symmetric.
pub fn run(memories: &[Memory], query: &str) -> Vec<Hit> {
    let terms = parse_query(query);
    if terms.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for (idx, m) in memories.iter().enumerate() {
        let lc_summary = m.summary.to_lowercase();
        let lc_tags: Vec<String> = m.tags.iter().map(|t| t.to_lowercase()).collect();
        // Content is folded once and shared by all terms (see find_all_in_folded)
        let (folded, map) = fold_with_map(&m.content);

        let mut score = 0i64;
        let mut all_matched = true;
        // Content hit positions per term (original byte offsets), for snippet window selection
        let mut content_hits: Vec<Vec<usize>> = Vec::new();

        for term in &terms {
            let mut term_score = 0i64;
            for t in &lc_tags {
                if *t == *term {
                    term_score += W_TAG_EXACT;
                } else if t.contains(term.as_str()) {
                    term_score += W_TAG_SUBSTR;
                }
            }
            let summary_hits = count_non_overlapping(&lc_summary, term);
            if summary_hits > 0 {
                term_score += summary_hits.min(MAX_TF_HITS) as i64 * W_SUMMARY;
                if term.is_ascii() && has_word_bounded(&lc_summary, term) {
                    term_score += W_WORD_SUMMARY;
                }
            }
            let positions = find_all_in_folded(&folded, &map, term);
            if !positions.is_empty() {
                term_score += positions.len().min(MAX_TF_HITS) as i64 * W_CONTENT;
                // Word-boundary checks can run on the folded text: ASCII case folding is 1:1 and
                // preserves ASCII alphanumerics; for non-ASCII characters, neither form is
                if term.is_ascii() && has_word_bounded(&folded, term) {
                    term_score += W_WORD_CONTENT;
                }
            }
            // Quoted phrases (any term containing whitespace counts as a phrase) get a bonus for whole-phrase hits
            if term.contains(' ') && (summary_hits > 0 || !positions.is_empty()) {
                term_score += W_PHRASE;
            }
            if term_score == 0 {
                all_matched = false;
                break;
            }
            score += term_score;
            content_hits.push(positions);
        }

        if !all_matched {
            continue;
        }
        let anchor = choose_snippet_anchor(&content_hits);
        let snippet = make_snippet(&m.content, anchor);
        hits.push(Hit {
            idx,
            score,
            snippet,
        });
    }

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(memories[b.idx].updated_at.cmp(&memories[a.idx].updated_at))
            .then(memories[a.idx].id.cmp(&memories[b.idx].id))
    });
    hits
}

/// Pick the snippet window among candidate anchors: prefer the window covering the most distinct terms, ties broken by the earliest.
/// Anchor candidates are the first few content hit positions of each term.
fn choose_snippet_anchor(content_hits: &[Vec<usize>]) -> Option<usize> {
    let mut candidates: Vec<usize> = Vec::new();
    for positions in content_hits {
        candidates.extend(positions.iter().take(4).copied());
    }
    candidates.sort_unstable();
    candidates.dedup();
    let mut best: Option<(usize, usize)> = None; // (distinct terms covered, anchor)
    for &p in &candidates {
        let lo = p.saturating_sub(SNIPPET_BEFORE);
        let hi = p + SNIPPET_AFTER;
        let covered = content_hits
            .iter()
            .filter(|ps| ps.iter().any(|&x| x >= lo && x <= hi))
            .count();
        if best.is_none_or(|(bc, bp)| covered > bc || (covered == bc && p < bp)) {
            best = Some((covered, p));
        }
    }
    best.map(|(_, p)| p)
}

/// Non-overlapping substring count (the caller guarantees lowercase alignment).
fn count_non_overlapping(haystack: &str, term: &str) -> usize {
    if term.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut from = 0;
    while let Some(rel) = haystack[from..].find(term) {
        count += 1;
        from += rel + term.len();
        if from > haystack.len() {
            break;
        }
    }
    count
}

/// Whether a hit sits at a "word boundary": neither neighboring character is an ASCII alphanumeric.
fn has_word_bounded(haystack: &str, term: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = haystack[from..].find(term) {
        let s = from + rel;
        let e = s + term.len();
        let before_ok = s == 0
            || !haystack[..s]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric());
        let after_ok = e == haystack.len()
            || !haystack[e..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        from = e;
        if from > haystack.len() {
            break;
        }
    }
    false
}

/// Fold text to lowercase character by character, recording the original byte offset behind each byte of the folded text.
///
/// You cannot find on `to_lowercase()` output and reuse the offsets on the original: for a few characters
/// lowercasing changes the byte length (e.g. U+0130 "İ" → "i̇"), misaligning offsets. Folding
/// records original offsets byte by byte, so position mapping is always exact.
fn fold_with_map(haystack: &str) -> (String, Vec<usize>) {
    let mut folded = String::with_capacity(haystack.len());
    // Every byte position of folded → original byte offset (one output character may span several bytes,
    // and each byte needs its own entry so indexes stay aligned with folded)
    let mut map: Vec<usize> = Vec::with_capacity(haystack.len() + 1);
    for (orig_off, ch) in haystack.char_indices() {
        for l in ch.to_lowercase() {
            for _ in 0..l.len_utf8() {
                map.push(orig_off);
            }
            folded.push(l);
        }
    }
    map.push(haystack.len());
    (folded, map)
}

/// Find all non-overlapping hits of term in the folded text, returning original byte offsets.
fn find_all_in_folded(folded: &str, map: &[usize], term: &str) -> Vec<usize> {
    if term.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = folded[from..].find(term) {
        let pos = from + rel;
        out.push(map[pos.min(map.len() - 1)]);
        from = pos + term.len();
        if from > folded.len() {
            break;
        }
    }
    out
}

fn floor_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Fallback snippet when there are no content hit positions (start of the content). Vector-only semantic hits
/// have no term anchors and take this path.
pub fn fallback_snippet(content: &str) -> String {
    make_snippet(content, None)
}

/// Generate a single-line snippet around the match positions; falls back to the start of the content when there are no content hits.
/// Match positions come from folded lookup and are always original byte offsets; window edges
/// fall back to character-boundary alignment so nothing is cut mid-character.
///
/// The result is plain text, unescaped: snippets are data, and rendering concerns (HTML escaping
/// among them) belong to the display layer — the web UI renders them via text interpolation,
/// so no escaping is needed anywhere.
fn make_snippet(content: &str, pos: Option<usize>) -> String {
    let (start, end) = match pos {
        None => (0, SNIPPET_AFTER.min(content.len())),
        Some(p) => (
            p.saturating_sub(SNIPPET_BEFORE),
            (p + SNIPPET_AFTER).min(content.len()),
        ),
    };
    let start = floor_boundary(content, start);
    let end = ceil_boundary(content, end);
    let inner: String = content[start..end]
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' || c == '\t' {
                ' '
            } else {
                c
            }
        })
        .collect();
    let inner = inner.trim().to_string();
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.push_str(&inner);
    if end < content.len() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem(id: &str, tags: &[&str], summary: &str, content: &str, updated_at: u64) -> Memory {
        Memory {
            id: id.to_string(),
            summary: summary.to_string(),
            content: content.to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            created_at: 0,
            updated_at,
        }
    }

    #[test]
    fn and_semantics_and_ranking() {
        let a = mem(
            "m1",
            &["rust"],
            "Rust borrow checker notes",
            "plain text",
            5,
        );
        let b = mem("m2", &[], "unrelated", "the borrow checker is strict", 9);
        let hits = run(&[a, b], "borrow rust");
        // Only memories hit by both terms are returned; m1 (tag + title double hit) should rank ahead of m2 (content hit only).
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
    }

    /// Characters whose lowercase form changes length (U+0130 "İ" → "i̇") must not misalign the snippet window:
    /// the snippet must start with an ellipsis, fully contain the target term, and land precisely near the original match point.
    #[test]
    fn snippet_stays_aligned_when_case_folding_changes_length() {
        // "İ" grows from 2 to 3 bytes when lowercased: the old implementation's offset mapping was off by 1 byte
        let content = format!("İ{}TARGET{}", "前".repeat(60), "后".repeat(60));
        let a = mem("m1", &[], "s", &content, 1);
        let hits = run(&[a], "target");
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(sn.starts_with('…') && sn.ends_with('…'));
        assert!(sn.contains("TARGET"), "snippet: {sn}");
        assert!(
            sn.contains("前前前"),
            "snippet should include context before the match: {sn}"
        );
    }

    #[test]
    fn folded_find_returns_original_offsets() {
        fn first(haystack: &str, term: &str) -> Option<usize> {
            let (folded, map) = fold_with_map(haystack);
            find_all_in_folded(&folded, &map, &term.to_lowercase())
                .first()
                .copied()
        }
        assert_eq!(first("Hello WÖrld", "wörld"), Some(6));
        assert_eq!(first("xİy TARGET", "target"), Some(5));
        assert_eq!(first("", "a"), None);
        assert_eq!(first("abc", ""), None);
        assert_eq!(first("中文内容", "内容"), Some(6));

        // All non-overlapping multiple hits are returned
        let (folded, map) = fold_with_map("aXbXc");
        assert_eq!(find_all_in_folded(&folded, &map, "x"), vec![1, 3]);
    }

    #[test]
    fn cjk_substring_match() {
        let a = mem(
            "m1",
            &["笔记"],
            "关于借用检查",
            "Rust 的借用检查器会在编译期阻止数据竞争。",
            1,
        );
        let hits = run(std::slice::from_ref(&a), "借用检查器");
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.contains("借用检查器"));

        // AND semantics: a nonexistent term yields no results
        let hits = run(&[a], "借用检查器 完全不存在的词");
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn case_insensitive_ascii() {
        let a = mem("m1", &[], "Config Loading", "uses Serde for JSON", 1);
        let hits = run(&[a], "serde json");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn snippet_window_and_ellipses() {
        let long = format!("{}TARGET{}", "前".repeat(200), "后".repeat(200));
        let a = mem("m1", &[], "s", &long, 1);
        let hits = run(&[a], "target");
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(sn.starts_with('…') && sn.ends_with('…'));
        assert!(sn.contains("TARGET"));

        // Falls back to the start when there are no content hits
        let b = mem("m2", &[], "TARGET in summary", "short", 1);
        let hits = run(&[b], "target");
        assert_eq!(hits[0].snippet, "short");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let a = mem("m1", &[], "s", "c", 1);
        assert!(run(&[a], "   ").is_empty());
    }

    #[test]
    fn repeated_hits_rank_higher() {
        // Both hit "rust": the one with 3 content occurrences should rank ahead of the one with just 1
        let a = mem("m1", &[], "s", "rust rust rust and more rust mentions", 1);
        let b = mem("m2", &[], "s", "rust once", 1);
        let hits = run(&[b, a], "rust");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].idx, 1, "m1 (3+ content hits) must rank first");
    }

    #[test]
    fn ascii_word_boundary_ranks_above_substring() {
        // Searching "age": whole-word hit m1 should rank ahead of substring collateral ("message") m2
        let a = mem("m1", &[], "s", "storage age limits apply here", 1);
        let b = mem("m2", &[], "message summary here", "s", 1);
        let hits = run(&[b, a], "age");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].idx, 0, "word-bounded hit must rank first");
    }

    #[test]
    fn quoted_phrase_requires_adjacency_and_bonuses() {
        let adjacent = mem("m1", &[], "s", "the borrow checker is strict", 1);
        let separated = mem("m2", &[], "s", "borrow the checker later", 1);

        // Quoted phrase: "borrow checker" only hits m1, where the words are adjacent
        let hits = run(&[adjacent.clone(), separated.clone()], "\"borrow checker\"");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);

        // Without quotes it reverts to AND semantics: each term hitting anywhere is enough, so both are returned
        let hits = run(&[adjacent, separated], "borrow checker");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn snippet_window_prefers_multi_term_coverage() {
        // The old implementation took the first hit position (seeing only alpha);
        // the new implementation should pick the window covering both alpha and beta (at alpha's second occurrence)
        let content = format!(
            "alpha{}beta{}alpha{}",
            "x".repeat(120),
            "x".repeat(15),
            "x".repeat(10)
        );
        let a = mem("m1", &[], "s", &content, 1);
        let hits = run(std::slice::from_ref(&a), "alpha beta");
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(
            sn.contains("alpha") && sn.contains("beta"),
            "snippet must cover both terms: {sn}"
        );
    }

    #[test]
    fn snippet_stays_plain_text() {
        let a = mem(
            "m1",
            &[],
            "s",
            "hello <img src=x onerror=alert(1)> & \"quotes\" world",
            1,
        );
        let hits = run(std::slice::from_ref(&a), "img");
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        // Snippets are data, not markup: raw text goes out verbatim and the display layer
        // decides how to render it (the UI interpolates it as text)
        assert!(sn.contains("<img"), "raw text must survive: {sn}");
        assert!(sn.contains("& \"quotes\""), "no entity noise: {sn}");
        assert!(!sn.contains("&lt;"), "no HTML escaping: {sn}");
        assert!(!sn.contains("&quot;"), "no HTML escaping: {sn}");
    }

    #[test]
    fn parse_query_respects_quotes() {
        assert_eq!(
            parse_query("  rust   \"borrow checker\"  "),
            vec!["rust".to_string(), "borrow checker".to_string()]
        );
        // Unclosed quote: swallows to the end
        assert_eq!(
            parse_query("rust \"borrow checker"),
            vec!["rust".to_string(), "borrow checker".to_string()]
        );
    }
}
