//! 关键词搜索：空格分词（引号短语整体成词）、全部词命中（AND）、加权评分、
//! 生成片段。
//!
//! 匹配基于大小写折叠后的子串查找，因此中文按子串直接命中，无需分词。
//! 权重见下方常量：标签精确 > 标签子串 > 摘要 > 正文；摘要/正文按命中
//! 次数计分（封顶 3 次），纯 ASCII 词在词边界命中额外加分，引号短语
//! 整体命中再加分。片段窗口在多个候选位置中选取覆盖不同词最多的一个。

use crate::model::Memory;

/// 权重：标签名精确命中（每个命中的标签累计）。
const W_TAG_EXACT: i64 = 40;
/// 权重：标签名子串命中（每个命中的标签累计）。
const W_TAG_SUBSTR: i64 = 25;
/// 权重：摘要命中（每次；封顶 MAX_TF_HITS 次）。
const W_SUMMARY: i64 = 10;
/// 权重：正文命中（每次；封顶 MAX_TF_HITS 次）。
const W_CONTENT: i64 = 3;
/// 权重：纯 ASCII 词在词边界命中（摘要/正文各一次）。
/// 子串匹配对中文是特性（"记忆"命中"记忆库"），对 ASCII 会误伤
/// （查 "age" 命中 "message"）；词边界加分让整词命中排前而不收窄召回。
const W_WORD_SUMMARY: i64 = 5;
const W_WORD_CONTENT: i64 = 2;
/// 权重：引号短语整体命中（摘要或正文）。
const W_PHRASE: i64 = 8;
/// 命中次数计分封顶：超过后不再加分，避免长文碾压字段权重。
const MAX_TF_HITS: usize = 3;

pub struct Hit {
    pub idx: usize,
    pub score: i64,
    pub snippet: String,
}

const SNIPPET_BEFORE: usize = 40;
const SNIPPET_AFTER: usize = 120;

/// 拆分查询：空白分词；双引号内的空白不切分（短语整体成词）。
/// 所有词统一小写化。
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

pub fn run(memories: &[Memory], query: &str, tag_filter: &[String]) -> Vec<Hit> {
    let terms = parse_query(query);
    if terms.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for (idx, m) in memories.iter().enumerate() {
        if !tag_filter.is_empty() && !tag_filter.iter().any(|t| m.tags.iter().any(|x| x == t)) {
            continue;
        }
        let lc_summary = m.summary.to_lowercase();
        let lc_tags: Vec<String> = m.tags.iter().map(|t| t.to_lowercase()).collect();
        // 正文折叠一次，所有词共享（见 find_all_in_folded）
        let (folded, map) = fold_with_map(&m.content);

        let mut score = 0i64;
        let mut all_matched = true;
        // 每个词的正文命中位置（原文字节偏移），供片段窗口优选
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
                // 词边界判断在折叠文本上做即可：ASCII 大小写折叠是 1:1 且
                // 不改变 ASCII 字母数字属性；非 ASCII 字符两种形态都不是
                if term.is_ascii() && has_word_bounded(&folded, term) {
                    term_score += W_WORD_CONTENT;
                }
            }
            // 引号短语（词内含空白即视为短语）整体命中额外加分
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

/// 在候选锚点中选片段窗口：优先覆盖不同词数最多的窗口，平局取最靠前的。
/// 锚点候选取每个词的前几个正文命中位置。
fn choose_snippet_anchor(content_hits: &[Vec<usize>]) -> Option<usize> {
    let mut candidates: Vec<usize> = Vec::new();
    for positions in content_hits {
        candidates.extend(positions.iter().take(4).copied());
    }
    candidates.sort_unstable();
    candidates.dedup();
    let mut best: Option<(usize, usize)> = None; // (覆盖词数, 锚点)
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

/// 非重叠子串计数（调用方保证已小写化对齐）。
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

/// 命中是否处于"词边界"：前后相邻字符都不是 ASCII 字母数字。
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

/// 逐字符折叠出小写文本，并记录折叠文本每个字节对应的原文字节偏移。
///
/// 不能直接在 `to_lowercase()` 结果上 find 再把偏移用于原文：个别字符
/// 小写化会改变字节长度（如 U+0130 "İ" → "i̇"），导致偏移错位。折叠
/// 时逐字节记录原文偏移，位置换算永远精确。
fn fold_with_map(haystack: &str) -> (String, Vec<usize>) {
    let mut folded = String::with_capacity(haystack.len());
    // folded 的每个字节位置 → 原文字节偏移（一个输出字符可能占多个字节，
    // 每个字节都要各记一条，索引才能与 folded 对齐）
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

/// 在折叠文本上找出 term 的全部非重叠命中，返回原文字节偏移。
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

/// 生成匹配位置附近的单行片段；没有正文命中时回退为正文开头。
/// 匹配位置来自折叠查找，恒为原文字节偏移；窗口边界
/// 兜底对齐字符边界，保证不切在字符中间。
///
/// 结果做 HTML 转义：正文是多 agent 共写的外部输入，片段会被管理界面
/// 以 v-html 渲染，不转义等于开放注入。
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
    let inner = escape_html(inner.trim());
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

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
        let hits = run(&[a, b], "borrow rust", &[]);
        // 只有两词都命中的才返回；m1 标签+摘要双命中应排在 m2（仅正文命中）前面。
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
    }

    #[test]
    fn tag_filter_restricts() {
        let a = mem("m1", &["rust"], "has keyword here", "x", 1);
        let b = mem("m2", &["other"], "has keyword here", "x", 2);
        let hits = run(&[a, b], "keyword", &["rust".to_string()]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
    }

    /// 小写化会变长度的字符（U+0130 "İ" → "i̇"）不得使片段窗口错位：
    /// 片段必须以省略号开头且完整包含目标词，且精确落在原文匹配点附近。
    #[test]
    fn snippet_stays_aligned_when_case_folding_changes_length() {
        // "İ" 小写化从 2 字节变 3 字节：旧实现对偏移的换算会偏差 1 字节
        let content = format!("İ{}TARGET{}", "前".repeat(60), "后".repeat(60));
        let a = mem("m1", &[], "s", &content, 1);
        let hits = run(&[a], "target", &[]);
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

        // 非重叠多次命中全部返回
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
        let hits = run(std::slice::from_ref(&a), "借用检查器", &[]);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].snippet.contains("借用检查器"));

        // AND 语义：不存在的词导致无结果
        let hits = run(&[a], "借用检查器 完全不存在的词", &[]);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn case_insensitive_ascii() {
        let a = mem("m1", &[], "Config Loading", "uses Serde for JSON", 1);
        let hits = run(&[a], "serde json", &[]);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn snippet_window_and_ellipses() {
        let long = format!("{}TARGET{}", "前".repeat(200), "后".repeat(200));
        let a = mem("m1", &[], "s", &long, 1);
        let hits = run(&[a], "target", &[]);
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(sn.starts_with('…') && sn.ends_with('…'));
        assert!(sn.contains("TARGET"));

        // 无正文命中时回退到开头
        let b = mem("m2", &[], "TARGET in summary", "short", 1);
        let hits = run(&[b], "target", &[]);
        assert_eq!(hits[0].snippet, "short");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let a = mem("m1", &[], "s", "c", 1);
        assert!(run(&[a], "   ", &[]).is_empty());
    }

    #[test]
    fn repeated_hits_rank_higher() {
        // 两条都命中 "rust"：正文出现 3 次的应排在只出现 1 次的前面
        let a = mem("m1", &[], "s", "rust rust rust and more rust mentions", 1);
        let b = mem("m2", &[], "s", "rust once", 1);
        let hits = run(&[b, a], "rust", &[]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].idx, 1, "m1 (3+ content hits) must rank first");
    }

    #[test]
    fn ascii_word_boundary_ranks_above_substring() {
        // 查 "age"：整词命中的 m1 应排在子串误伤（"message"）的 m2 前面
        let a = mem("m1", &[], "s", "storage age limits apply here", 1);
        let b = mem("m2", &[], "message summary here", "s", 1);
        let hits = run(&[b, a], "age", &[]);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].idx, 0, "word-bounded hit must rank first");
    }

    #[test]
    fn quoted_phrase_requires_adjacency_and_bonuses() {
        let adjacent = mem("m1", &[], "s", "the borrow checker is strict", 1);
        let separated = mem("m2", &[], "s", "borrow the checker later", 1);

        // 引号短语："borrow checker" 只命中相邻出现的 m1
        let hits = run(
            &[adjacent.clone(), separated.clone()],
            "\"borrow checker\"",
            &[],
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);

        // 不加引号退回 AND 语义：两个词各自命中即可，两条都返回
        let hits = run(&[adjacent, separated], "borrow checker", &[]);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn snippet_window_prefers_multi_term_coverage() {
        // 旧实现取第一个命中位置（只看得到 alpha）；
        // 新实现应选覆盖 alpha 与 beta 两个词的窗口（alpha 的第二次出现处）
        let content = format!(
            "alpha{}beta{}alpha{}",
            "x".repeat(120),
            "x".repeat(15),
            "x".repeat(10)
        );
        let a = mem("m1", &[], "s", &content, 1);
        let hits = run(std::slice::from_ref(&a), "alpha beta", &[]);
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(
            sn.contains("alpha") && sn.contains("beta"),
            "snippet must cover both terms: {sn}"
        );
    }

    #[test]
    fn snippet_escapes_html() {
        let a = mem(
            "m1",
            &[],
            "s",
            "hello <img src=x onerror=alert(1)> world",
            1,
        );
        let hits = run(std::slice::from_ref(&a), "img", &[]);
        assert_eq!(hits.len(), 1);
        let sn = &hits[0].snippet;
        assert!(sn.contains("&lt;img"), "snippet must escape HTML: {sn}");
        assert!(!sn.contains("<img"), "raw HTML must not survive: {sn}");
    }

    #[test]
    fn parse_query_respects_quotes() {
        assert_eq!(
            parse_query("  rust   \"borrow checker\"  "),
            vec!["rust".to_string(), "borrow checker".to_string()]
        );
        // 未闭合引号：吞到结尾
        assert_eq!(
            parse_query("rust \"borrow checker"),
            vec!["rust".to_string(), "borrow checker".to_string()]
        );
    }
}
