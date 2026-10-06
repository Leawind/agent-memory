//! Text rendering of the three list tools on the MCP face.
//!
//! The tool-result text block is the single data channel, and a JSON encoding makes the caller's
//! model pay the row scaffolding (key names, quotes, braces) once per record. `tag_list`,
//! `memory_list` and `memory_search` therefore render as a compact line format here; every other
//! tool, and the whole REST face (parsed by the admin UI), keeps the structured value as compact
//! JSON. The line grammar is specified verbatim in the tools' `defs.rs` descriptions — this module
//! only renders, it never re-derives data.
//!
//! Row grammar (one record per line, newline is the record separator):
//! - `tag_list`      → `<count> <[*]name>[: <description>]`
//! - `memory_list`   → header `total: N | offset: N`, rows `<id> [<tags>] <updated> <summary>`
//! - `memory_search` → header `total_matches: N | offset: N | returned: N | mode: <mode>`,
//!   optional flag lines (`semantic_fallback` / `semantic` / `hint` / `note`, only when they
//!   apply), rows `<id> [<tags>] <updated> <score> <summary>` plus an indented `  > <snippet>`
//!   continuation line

use serde_json::Value;

use super::{MEMORY_LIST, MEMORY_SEARCH, TAG_LIST};

/// The text-channel payload of a successful tool call: the line format for the three list tools,
/// compact JSON for everything else (writes, memory_get, ...). A value whose shape does not match
/// the expected record layout falls back to JSON rather than rendering a silently wrong line.
pub fn tool_text(tool: &str, v: &Value) -> String {
    let rendered = match tool {
        TAG_LIST => render_tag_list(v),
        MEMORY_LIST => render_memory_list(v),
        MEMORY_SEARCH => render_memory_search(v),
        _ => None,
    };
    rendered.unwrap_or_else(|| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()))
}

/// `<count> <[*]name>[: <description>]` per line; the description is omitted when empty or absent.
fn render_tag_list(v: &Value) -> Option<String> {
    let tags = v.get("tags")?.as_array()?;
    let mut out = String::new();
    for t in tags {
        let count = t.get("count")?.as_u64()?;
        let name = t.get("name")?.as_str()?;
        out.push_str(&count.to_string());
        out.push(' ');
        if t.get("reserved").and_then(Value::as_bool).unwrap_or(false) {
            out.push('*');
        }
        out.push_str(&quote_name(name));
        if let Some(desc) = t.get("description").and_then(Value::as_str) {
            if !desc.is_empty() {
                out.push_str(": ");
                out.push_str(&escape_text(desc));
            }
        }
        out.push('\n');
    }
    Some(out)
}

/// Header `total: N | offset: N`, then `<id> [<tags>] <updated> <summary>` per line.
fn render_memory_list(v: &Value) -> Option<String> {
    let total = v.get("total")?.as_u64()?;
    let offset = v.get("offset")?.as_u64()?;
    let memories = v.get("memories")?.as_array()?;
    let mut out = format!("total: {total} | offset: {offset}\n");
    for m in memories {
        push_memory_row(&mut out, m, None)?;
    }
    Some(out)
}

/// Header `total_matches: N | offset: N | returned: N | mode: <mode>`, then the optional flag
/// lines in handler attach order, then `<id> [<tags>] <updated> <score> <summary>` per result,
/// each followed by an indented `  > <snippet>` continuation line.
fn render_memory_search(v: &Value) -> Option<String> {
    let total = v.get("total_matches")?.as_u64()?;
    let offset = v.get("offset")?.as_u64()?;
    let returned = v.get("returned")?.as_u64()?;
    let mode = v.get("mode")?.as_str()?;
    let results = v.get("results")?.as_array()?;
    let mut out = format!(
        "total_matches: {total} | offset: {offset} | returned: {returned} | mode: {mode}\n"
    );
    if v.get("semantic_fallback")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        out.push_str("semantic_fallback: true\n");
    }
    if let Some(semantic) = v.get("semantic").and_then(Value::as_str) {
        out.push_str("semantic: ");
        out.push_str(semantic);
        out.push('\n');
    }
    for key in ["hint", "note"] {
        if let Some(text) = v.get(key).and_then(Value::as_str) {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(&escape_text(text));
            out.push('\n');
        }
    }
    for r in results {
        push_memory_row(&mut out, r, r.get("score"))?;
        if let Some(snippet) = r.get("snippet").and_then(Value::as_str) {
            out.push_str("  > ");
            out.push_str(&escape_text(snippet));
            out.push('\n');
        }
    }
    Some(out)
}

/// `<id> [<tag,tag>] <updated>[ <score>] <summary>` — the shared row of list and search; the
/// score column exists only on search rows. Ids and timestamps are store-generated (`m{n}`,
/// 'YYYY-MM-DD HH:MM') and stay verbatim; everything free-form is escaped.
fn push_memory_row(out: &mut String, m: &Value, score: Option<&Value>) -> Option<()> {
    let id = m.get("id")?.as_str()?;
    let tags = m.get("tags")?.as_array()?;
    let updated = m.get("updated")?.as_str()?;
    let summary = m.get("summary")?.as_str()?;
    out.push_str(id);
    out.push_str(" [");
    for (i, t) in tags.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&quote_name(t.as_str()?));
    }
    out.push_str("] ");
    out.push_str(updated);
    if let Some(score) = score {
        out.push(' ');
        out.push_str(&score.to_string());
    }
    out.push(' ');
    out.push_str(&escape_text(summary));
    out.push('\n');
    Some(())
}

/// Quote a name only when it would blur the row grammar (whitespace, the `,` `[` `]` `|`
/// separators, quotes). Single quotes are preferred; a name containing a quote, backslash or
/// line break goes into double quotes with backslash escapes instead.
fn quote_name(name: &str) -> String {
    let bare = !name.is_empty()
        && !name
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, ',' | '[' | ']' | '|' | '\'' | '"' | '\\'));
    if bare {
        return name.to_string();
    }
    if !name
        .chars()
        .any(|c| matches!(c, '\'' | '"' | '\\' | '\n' | '\r'))
    {
        return format!("'{name}'");
    }
    let mut out = String::with_capacity(name.len() + 2);
    out.push('"');
    for c in name.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Escape free text (description, summary, hint, note, snippet) so one line stays one record:
/// backslash and line breaks become their literal two-character forms.
fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn non_list_tools_stay_compact_json() {
        assert_eq!(
            tool_text("memory_update", &json!({"replaced": 1})),
            r#"{"replaced":1}"#
        );
    }

    #[test]
    fn tag_list_lines_are_sparse() {
        let v = json!({"tags": [
            {"count": 3, "description": "参考知识", "name": "doc"},
            {"count": 1, "name": "git"},
            {"count": 0, "reserved": true, "name": "conventions", "description": "Reserved tag: anchors"},
        ]});
        assert_eq!(
            tool_text(TAG_LIST, &v),
            "3 doc: 参考知识\n\
             1 git\n\
             0 *conventions: Reserved tag: anchors\n"
        );
    }

    #[test]
    fn tag_names_outside_the_bare_grammar_are_quoted() {
        let v = json!({"tags": [
            {"count": 2, "name": "rust & life"},
            {"count": 1, "name": "a,b"},
            {"count": 1, "name": "say \"hi\""},
            {"count": 1, "name": "it's"},
            {"count": 1, "name": "back\\slash"},
            {"count": 1, "name": "proj/alpha"},
        ]});
        assert_eq!(
            tool_text(TAG_LIST, &v),
            "2 'rust & life'\n\
             1 'a,b'\n\
             1 \"say \\\"hi\\\"\"\n\
             1 \"it's\"\n\
             1 \"back\\\\slash\"\n\
             1 proj/alpha\n"
        );
    }

    #[test]
    fn tag_description_newlines_stay_inside_the_line() {
        let v = json!({"tags": [{"count": 1, "name": "x", "description": "one\ntwo\r\nthree"}]});
        assert_eq!(tool_text(TAG_LIST, &v), "1 x: one\\ntwo\\r\\nthree\n");
    }

    #[test]
    fn memory_list_header_and_rows() {
        let v = json!({
            "total": 2, "offset": 20, "limit": 20,
            "memories": [
                {"id": "m7", "tags": ["doc", "trap"], "summary": "first", "updated": "2026-10-05 18:23"},
                {"id": "m3", "tags": [], "summary": "multi\nline", "updated": "2026-10-04 09:00"},
            ],
        });
        assert_eq!(
            tool_text(MEMORY_LIST, &v),
            "total: 2 | offset: 20\n\
             m7 [doc,trap] 2026-10-05 18:23 first\n\
             m3 [] 2026-10-04 09:00 multi\\nline\n"
        );
    }

    #[test]
    fn memory_search_flag_lines_snippets_and_score() {
        let v = json!({
            "total_matches": 1, "offset": 0, "returned": 1, "mode": "hybrid",
            "semantic_fallback": true,
            "hint": "call memory_get on the ids worth reading",
            "results": [
                {"id": "m7", "tags": ["doc"], "summary": "summ", "score": 0.031,
                 "updated": "2026-10-05 18:23", "snippet": "…summ…"},
            ],
        });
        assert_eq!(
            tool_text(MEMORY_SEARCH, &v),
            // concat!, not a `\`-continued literal: the continuation would swallow the snippet
            // line's leading indent
            concat!(
                "total_matches: 1 | offset: 0 | returned: 1 | mode: hybrid\n",
                "semantic_fallback: true\n",
                "hint: call memory_get on the ids worth reading\n",
                "m7 [doc] 2026-10-05 18:23 0.031 summ\n",
                "  > …summ…\n",
            )
        );
    }

    #[test]
    fn memory_search_empty_result_is_header_plus_note() {
        let v = json!({
            "total_matches": 0, "offset": 10, "returned": 0, "mode": "keyword",
            "note": "no memory matched every term; drop some terms or try broader ones",
            "results": [],
        });
        assert_eq!(
            tool_text(MEMORY_SEARCH, &v),
            "total_matches: 0 | offset: 10 | returned: 0 | mode: keyword\n\
             note: no memory matched every term; drop some terms or try broader ones\n"
        );
    }

    #[test]
    fn unexpected_shape_falls_back_to_json_instead_of_wrong_lines() {
        let text = tool_text(TAG_LIST, &json!({"tags": "not-an-array"}));
        assert!(serde_json::from_str::<Value>(&text).is_ok(), "{text}");
    }
}
