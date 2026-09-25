//! 关键词搜索：空格分词、全部词命中（AND）、加权评分、生成片段。
//!
//! 匹配基于大小写折叠后的子串查找，因此中文按子串直接命中，无需分词。
//! 权重见下方常量：标签精确 > 标签子串 > 摘要 > 正文。

use crate::model::Memory;

/// 权重：标签名精确命中（每个命中的标签累计）。
const W_TAG_EXACT: i64 = 40;
/// 权重：标签名子串命中（每个命中的标签累计）。
const W_TAG_SUBSTR: i64 = 25;
/// 权重：摘要命中。
const W_SUMMARY: i64 = 10;
/// 权重：正文命中。
const W_CONTENT: i64 = 3;

pub struct Hit {
    pub idx: usize,
    pub score: i64,
    pub snippet: String,
}

const SNIPPET_BEFORE: usize = 40;
const SNIPPET_AFTER: usize = 120;

pub fn run(memories: &[Memory], query: &str, tag_filter: &[String]) -> Vec<Hit> {
    let terms: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
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

        let mut score = 0i64;
        let mut all_matched = true;
        let mut first_pos: Option<usize> = None;

        for term in &terms {
            let mut term_score = 0i64;
            for t in &lc_tags {
                if *t == *term {
                    term_score += W_TAG_EXACT;
                } else if t.contains(term.as_str()) {
                    term_score += W_TAG_SUBSTR;
                }
            }
            if lc_summary.contains(term.as_str()) {
                term_score += W_SUMMARY;
            }
            if let Some(pos) = find_case_insensitive(&m.content, term) {
                term_score += W_CONTENT;
                first_pos = Some(match first_pos {
                    None => pos,
                    Some(p) => p.min(pos),
                });
            }
            if term_score == 0 {
                all_matched = false;
                break;
            }
            score += term_score;
        }

        if !all_matched {
            continue;
        }
        let snippet = make_snippet(&m.content, first_pos);
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

/// 大小写不敏感的子串查找，返回**原始**字符串中的字节偏移。
///
/// 不能直接在 `to_lowercase()` 结果上 find 再把偏移用于原文：个别字符
/// 小写化会改变字节长度（如 U+0130 "İ" → "i̇"），导致偏移错位。这里
/// 逐字符折叠并记录每个输出字节对应的原文偏移，偏移永远精确。
fn find_case_insensitive(haystack: &str, term: &str) -> Option<usize> {
    if term.is_empty() {
        return None;
    }
    let term_lc = term.to_lowercase();
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
    folded
        .find(term_lc.as_str())
        .map(|pos| map[pos.min(map.len() - 1)])
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
/// 匹配位置来自 `find_case_insensitive`，恒为原文字节偏移；窗口边界
/// 兜底对齐字符边界，保证不切在字符中间。
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
    let inner = inner.trim();
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.push_str(inner);
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
        assert!(sn.contains("TARGET"), "snippet: {}", sn);
        assert!(
            sn.contains("前前前"),
            "snippet should include context before the match: {}",
            sn
        );
    }

    #[test]
    fn find_case_insensitive_returns_original_offsets() {
        assert_eq!(find_case_insensitive("Hello WÖrld", "wörld"), Some(6));
        assert_eq!(find_case_insensitive("xİy TARGET", "target"), Some(5));
        assert_eq!(find_case_insensitive("", "a"), None);
        assert_eq!(find_case_insensitive("abc", ""), None);
        assert_eq!(find_case_insensitive("中文内容", "内容"), Some(6));
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
}
