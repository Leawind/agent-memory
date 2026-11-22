//! Tag set algebra: parse and evaluate boolean expressions over tag names.
//!
//! Every tag is a set of memories, so a tag expression is a set expression: `(a&b)|c` selects
//! memories carrying both `a` and `b`, or carrying `c`. A leaf names tags exactly — bare
//! (`rust`) or scoped (`system::pc1sw0`) — and passes only when the memory carries that name;
//! `scope::*` widens it to every name in that scope, so `system::*` means "carries some value in
//! system". Grammar (precedence `!` > `&` > `|`, parentheses group, `&&` / `||` accepted as
//! aliases):
//!
//! ```text
//! expr    := or
//! or      := and ("|" | "||" and)*
//! and     := unary ("&" | "&&" unary)*
//! unary   := ("!" unary) | primary
//! primary := "(" expr ")" | "mutex(" expr ("," expr)+ ")" | named | tag | regex
//! named   := "@" word
//! tag     := quoted | word | word "::" "*"
//! word    := 1+ chars outside whitespace, parentheses, "&|!=", quotes and "*"
//! quoted  := '"' (escape | char)* '"' | "'" (escape | char)* "'"
//! regex   := "/" (escape | char)* "/"
//! ```
//!
//! A word holds at most one `::` separator, and both halves must be identifier words
//! (`model::normalize_tag_name`): a lone `:` and a `*` that does not spell out a scope wildcard
//! are syntax errors. Quoting escapes the expression operators only — the text still goes
//! through the tag-name rule, so a quoted name is never a wildcard.
//!
//! A regex atom passes when ANY of the memory's tag names matches (Rust regex syntax, case
//! sensitive by default — inline flags like `(?i)` work). A `/` opens a regex only at a token
//! boundary; inside a word it stays an ordinary character, which the name rule then rejects.
//! Inside a regex, `\/` escapes the delimiter and every other backslash sequence passes through
//! to the regex engine untouched (`\d`, `\\`, ...).
//!
//! Parsing is pure syntax; the caller validates the leaves against the store (unknown names are
//! an error with a did-you-mean hint, mirroring tag linking on writes). `tag_names` reports the
//! exact names a query needs, `scope_names` the scopes its wildcards need. Regex atoms carry no
//! names and match dynamically.

use crate::model::{normalize_ident_word, normalize_tag_name, tag_scope_of};
use crate::predicate::Predicate;

pub type TagExpr = Predicate<TagAtom>;

/// Query atoms use names; persisted predicate definitions bind literal atoms to stable ids.
#[derive(Debug, Clone)]
pub enum TagAtom {
    /// An exact tag name, bare (`rust`) or scoped (`system::pc1sw0`).
    Tag(String),
    /// The scope half of `scope::*`: matches any name in that scope and nothing else.
    Scope(String),
    Named(String),
    Regex(Box<regex::Regex>),
}

impl TagExpr {
    /// Evaluate against one memory's tag set: a leaf passes when the set carries the name (or,
    /// for a scope wildcard, a name in that scope), a regex when any name matches.
    pub fn eval(&self, tags: &[String]) -> bool {
        self.eval_with(&|atom| match atom {
            TagAtom::Tag(name) => tags.iter().any(|t| t == name),
            TagAtom::Scope(scope) => tags
                .iter()
                .any(|t| tag_scope_of(t.as_str()) == Some(scope.as_str())),
            TagAtom::Regex(re) => tags.iter().any(|t| re.is_match(t)),
            TagAtom::Named(_) => {
                unreachable!("named predicates must be resolved before evaluation")
            }
        })
    }

    /// Leaf tag names in first-appearance order, deduplicated (for store-side existence checks).
    /// Regex atoms match dynamically and contribute none.
    pub fn tag_names(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        self.visit_atoms(&mut |atom| {
            if let TagAtom::Tag(name) = atom {
                if !out.contains(&name.as_str()) {
                    out.push(name.as_str());
                }
            }
        });
        out
    }

    /// Leaf scope names in first-appearance order, deduplicated (the store checks they exist).
    /// Only a scope wildcard carries one.
    pub fn scope_names(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        self.visit_atoms(&mut |atom| {
            if let TagAtom::Scope(scope) = atom {
                if !out.contains(&scope.as_str()) {
                    out.push(scope.as_str());
                }
            }
        });
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    LParen,
    RParen,
    Amp,
    Pipe,
    Bang,
    Comma,
    Tag(String),
    Scope(String),
    Named(String),
    Regex(String),
}

/// Parse an expression; errors carry the char position (1-based) of the offending token.
pub fn parse(input: &str) -> Result<TagExpr, String> {
    parse_bounded(input, 4096)
}

/// Persistent rules have room for up to 256 renamed tags of 100 characters each.
pub fn parse_rule(input: &str) -> Result<TagExpr, String> {
    parse_bounded(input, 32768)
}

fn parse_bounded(input: &str, max_chars: usize) -> Result<TagExpr, String> {
    if input.chars().count() > max_chars {
        return Err(format!("expression exceeds {max_chars} characters"));
    }
    let tokens = tokenize(input)?;
    if tokens.len() > 256 {
        return Err("expression exceeds 256 tokens".into());
    }
    if tokens.is_empty() {
        return Err("empty expression".to_string());
    }
    let mut p = Parser {
        tokens,
        pos: 0,
        depth: 0,
    };
    let expr = p.parse_or()?;
    if let Some((tok, pos)) = p.peek() {
        return Err(format!(
            "unexpected {} at position {pos}: end of expression expected",
            describe(tok)
        ));
    }
    Ok(expr)
}

fn tokenize(input: &str) -> Result<Vec<(Tok, usize)>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Positions are 1-based char offsets, matching how editors count columns
        let pos = i + 1;
        match c {
            '(' => out.push((Tok::LParen, pos)),
            ')' => out.push((Tok::RParen, pos)),
            '&' => {
                if chars.get(i + 1) == Some(&'&') {
                    i += 1;
                }
                out.push((Tok::Amp, pos));
            }
            '|' => {
                if chars.get(i + 1) == Some(&'|') {
                    i += 1;
                }
                out.push((Tok::Pipe, pos));
            }
            '!' => out.push((Tok::Bang, pos)),
            ',' => out.push((Tok::Comma, pos)),
            '=' => {
                // Values live in scopes now; '=' only ever reaches here from a name word
                return Err(format!(
                    "unexpected '=' at position {pos}: a tag value is written 'scope::value'"
                ));
            }
            '"' | '\'' => {
                let quote = c;
                let mut name = String::new();
                loop {
                    i += 1;
                    match chars.get(i) {
                        None => {
                            return Err(format!(
                                "unterminated quoted tag name starting at position {pos}"
                            ))
                        }
                        Some('\\') => {
                            // Backslash escapes the next character (quote, backslash, anything);
                            // a trailing lone backslash stays literal
                            if let Some(&next) = chars.get(i + 1) {
                                name.push(next);
                                i += 1;
                            } else {
                                name.push('\\');
                            }
                        }
                        Some(&ch) if ch == quote => break,
                        Some(&ch) => name.push(ch),
                    }
                }
                if name.trim().is_empty() {
                    return Err(format!("empty quoted tag name at position {pos}"));
                }
                // Quoting escapes the operators, not the name rule: the text is still one name
                out.push((Tok::Tag(normalize_tag_name(&name)?), pos));
            }
            c if c.is_whitespace() => {}
            '/' => {
                // A slash at a token boundary opens a regex atom; inside a word (handled below)
                // it stays ordinary, so it never splits a name
                i += 1; // skip the opening delimiter
                let mut pattern = String::new();
                let mut closed = false;
                while i < chars.len() {
                    let ch = chars[i];
                    if ch == '\\' {
                        // Only '\/' escapes the delimiter; every other backslash sequence
                        // (including '\\') passes through verbatim to the regex engine
                        match chars.get(i + 1) {
                            Some('/') => {
                                pattern.push('/');
                                i += 2;
                            }
                            Some(&next) => {
                                pattern.push('\\');
                                pattern.push(next);
                                i += 2;
                            }
                            None => {
                                pattern.push('\\');
                                i += 1;
                            }
                        }
                        continue;
                    }
                    if ch == '/' {
                        // Stop ON the closing delimiter; the outer loop's i += 1 steps past it
                        // (same convention as the quoted-name arm)
                        closed = true;
                        break;
                    }
                    pattern.push(ch);
                    i += 1;
                }
                if !closed {
                    return Err(format!(
                        "unterminated regular expression starting at position {pos}"
                    ));
                }
                if pattern.is_empty() {
                    return Err(format!("empty regular expression at position {pos}"));
                }
                out.push((Tok::Regex(pattern), pos));
            }
            _ => {
                // One name word: whitespace and the operators end it, so a glued operator is a
                // syntax error rather than part of the name
                let mut word: Vec<(usize, char)> = Vec::new();
                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_whitespace() || "(),&|!=\"'".contains(ch) {
                        break;
                    }
                    word.push((i, ch));
                    i += 1;
                }
                i -= 1;
                out.push(name_token(&word)?);
            }
        }
        i += 1;
    }
    Ok(out)
}

/// Classify one name word: `@name` references a predicate, `scope::*` is a scope wildcard, and
/// anything else is an exact tag name. Both halves of a scoped name are identifier words.
fn name_token(word: &[(usize, char)]) -> Result<(Tok, usize), String> {
    let start = word[0].0 + 1;
    let text: String = word.iter().map(|(_, c)| *c).collect();
    if let Some(reference) = text.strip_prefix('@') {
        if reference.is_empty() {
            return Err(format!("empty named predicate at position {start}"));
        }
        return Ok((Tok::Named(normalize_tag_name(reference)?), start));
    }
    // One '::' pair at most, and a '*' only where it spells out the wildcard value
    let mut separator: Option<usize> = None;
    let mut i = 0;
    while i < word.len() {
        let at = word[i].0 + 1;
        match word[i].1 {
            '*' => {
                let closes_wildcard =
                    i + 1 == word.len() && i >= 2 && word[i - 1].1 == ':' && word[i - 2].1 == ':';
                if !closes_wildcard {
                    return Err(format!(
                        "unexpected '*' at position {at}: '*' is the value of a scope wildcard"
                    ));
                }
                i += 1;
            }
            ':' => {
                if !matches!(word.get(i + 1), Some((_, ':'))) {
                    return Err(format!(
                        "unexpected ':' at position {at}: write a scoped name as 'scope::value'"
                    ));
                }
                if separator.is_some() {
                    return Err(format!(
                        "unexpected '::' at position {at}: a tag name holds at most one '::'"
                    ));
                }
                separator = Some(i);
                i += 2;
            }
            _ => i += 1,
        }
    }
    let Some(split_at) = separator else {
        return Ok((Tok::Tag(normalize_tag_name(&text)?), start));
    };
    let scope: String = word[..split_at].iter().map(|(_, c)| *c).collect();
    let separator_at = word[split_at].0 + 1;
    if scope.is_empty() {
        return Err(format!(
            "missing scope name before '::' at position {separator_at}"
        ));
    }
    let value: String = word[split_at + 2..].iter().map(|(_, c)| *c).collect();
    if value.is_empty() {
        return Err(format!(
            "missing value after '::' at position {separator_at}"
        ));
    }
    if value == "*" {
        return Ok((
            Tok::Scope(normalize_ident_word(&scope, "tag scope")?),
            start,
        ));
    }
    Ok((Tok::Tag(normalize_tag_name(&text)?), start))
}

fn describe(tok: &Tok) -> String {
    match tok {
        Tok::LParen => "'('".to_string(),
        Tok::RParen => "')'".to_string(),
        Tok::Amp => "'&'".to_string(),
        Tok::Pipe => "'|'".to_string(),
        Tok::Bang => "'!'".to_string(),
        Tok::Comma => "','".to_string(),
        Tok::Tag(name) => format!("tag '{name}'"),
        Tok::Scope(scope) => format!("scope wildcard '{scope}::*'"),
        Tok::Named(name) => format!("named predicate '@{name}'"),
        Tok::Regex(pattern) => format!("regular expression '/{pattern}/'"),
    }
}

struct Parser {
    tokens: Vec<(Tok, usize)>,
    pos: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<(&Tok, usize)> {
        self.tokens.get(self.pos).map(|(t, p)| (t, *p))
    }

    fn next(&mut self) -> Option<(Tok, usize)> {
        let cur = self.tokens.get(self.pos).cloned()?;
        self.pos += 1;
        Some(cur)
    }

    fn parse_or(&mut self) -> Result<TagExpr, String> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some((Tok::Pipe, _))) {
            self.pos += 1;
            let right = self.parse_and()?;
            left = TagExpr::Any(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<TagExpr, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek(), Some((Tok::Amp, _))) {
            self.pos += 1;
            let right = self.parse_unary()?;
            left = TagExpr::All(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<TagExpr, String> {
        if self.depth >= 64 {
            return Err("expression exceeds nesting depth 64".into());
        }
        self.depth += 1;
        let result = self.parse_nested_unary();
        self.depth -= 1;
        result
    }

    fn parse_nested_unary(&mut self) -> Result<TagExpr, String> {
        if matches!(self.peek(), Some((Tok::Bang, _))) {
            self.pos += 1;
            let inner = self.parse_unary()?;
            return Ok(TagExpr::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<TagExpr, String> {
        let (tok, pos) = self.next().ok_or_else(|| {
            "unexpected end of expression: a tag, regex or '(' was expected".to_string()
        })?;
        match tok {
            Tok::LParen => {
                if matches!(self.peek(), Some((Tok::RParen, _))) {
                    return Err(format!("empty group at position {pos}"));
                }
                let inner = self.parse_or()?;
                match self.next() {
                    Some((Tok::RParen, _)) => Ok(inner),
                    _ => Err(format!("missing ')' for '(' at position {pos}")),
                }
            }
            Tok::Tag(name) if name == "mutex" && matches!(self.peek(), Some((Tok::LParen, _))) => {
                self.pos += 1;
                let mut args = vec![self.parse_or()?];
                while matches!(self.peek(), Some((Tok::Comma, _))) {
                    self.pos += 1;
                    args.push(self.parse_or()?);
                }
                if args.len() < 2 {
                    return Err(format!(
                        "mutex requires at least two operands at position {pos}"
                    ));
                }
                match self.next() {
                    Some((Tok::RParen, _)) => Ok(TagExpr::Mutex(args)),
                    _ => Err(format!("missing ')' for mutex at position {pos}")),
                }
            }
            Tok::Tag(name) => Ok(TagExpr::Atom(TagAtom::Tag(name))),
            Tok::Scope(scope) => Ok(TagExpr::Atom(TagAtom::Scope(scope))),
            Tok::Named(name) => Ok(TagExpr::Atom(TagAtom::Named(name))),
            Tok::Regex(pattern) => {
                let re = regex::Regex::new(&pattern).map_err(|e| {
                    format!("invalid regular expression '/{pattern}/' at position {pos}: {e}")
                })?;
                Ok(TagExpr::Atom(TagAtom::Regex(Box::new(re))))
            }
            other => Err(format!(
                "unexpected {} at position {pos}: a tag, regex or '(' was expected",
                describe(&other)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &str, tags: &[&str]) -> bool {
        let owned: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
        parse(expr)
            .unwrap_or_else(|e| panic!("parse '{expr}' failed: {e}"))
            .eval(&owned)
    }

    /// Evaluate two expressions over every sample tag set and require identical results —
    /// the precedence checks (which used tree equality before regex atoms carried compiled
    /// engines with no Eq).
    fn eval_equivalent(a: &str, b: &str) {
        let samples: Vec<Vec<&str>> = vec![
            vec![],
            vec!["a"],
            vec!["b"],
            vec!["c"],
            vec!["d"],
            vec!["a", "b"],
            vec!["b", "c"],
            vec!["a", "b", "c"],
            vec!["a", "d"],
        ];
        for tags in &samples {
            let owned: Vec<String> = tags.iter().map(|s| s.to_string()).collect();
            let left = parse(a)
                .unwrap_or_else(|e| panic!("parse '{a}' failed: {e}"))
                .eval(&owned);
            let right = parse(b)
                .unwrap_or_else(|e| panic!("parse '{b}' failed: {e}"))
                .eval(&owned);
            assert_eq!(left, right, "mismatch on tags {tags:?}");
        }
    }

    #[test]
    fn operators_and_precedence() {
        // The user-facing example: both a and b, or c alone
        assert!(eval("(a&b)|c", &["a", "b"]));
        assert!(eval("(a&b)|c", &["c"]));
        assert!(!eval("(a&b)|c", &["a"]));
        // & binds tighter than |
        eval_equivalent("a|b&c", "a|(b&c)");
        eval_equivalent("a&b|c&d", "(a&b)|(c&d)");
        // ! binds tighter than &
        eval_equivalent("!a&b", "(!a)&b");
        assert!(eval("!a&b", &["b"]));
        assert!(!eval("!a&b", &["a", "b"]));
        // Double-character aliases spell the same operators
        eval_equivalent("a&&b||c", "(a&b)|c");
        // Chains flatten into left-associative trees
        eval_equivalent("a&b&c", "(a&b)&c");
        assert!(eval("!!a", &["a"]));
    }

    /// An exact name matches itself and nothing else: a scoped name does not widen to its scope,
    /// and a bare name never matches a scoped one.
    #[test]
    fn exact_names_match_only_themselves() {
        assert!(eval("rust", &["rust"]));
        assert!(!eval("rust", &["rusty"]));
        assert!(eval("a::b", &["a::b"]));
        assert!(!eval("a::b", &["a::c"]));
        assert!(!eval("a::b", &["a"]));
        assert!(!eval("a", &["a::b"]));
        // A bare name never matches the value half of a scoped name
        assert!(!eval("a", &["b::a"]));
        // !a::b is decided on that exact name
        assert!(eval("!a::b", &["a::c"]));
        assert!(!eval("!a::b", &["a::b"]));
        assert!(eval("a::b&!a::c", &["a::b"]));
    }

    /// `scope::*` matches every name in that scope: not the bare scope tag, not another scope,
    /// and not a longer name that merely starts with the same letters.
    #[test]
    fn scope_wildcards_match_one_scope() {
        assert!(eval("a::*", &["a::b"]));
        assert!(eval("a::*", &["a::c", "x"]));
        assert!(!eval("a::*", &["b::a"]));
        assert!(!eval("a::*", &["a"]));
        assert!(!eval("a::*", &["ab::c"]));
        assert!(!eval("a::*", &[]));
        // Negation covers the whole scope at once
        assert!(eval("!a::*", &["a"]));
        assert!(eval("!a::*", &["b::a"]));
        assert!(!eval("!a::*", &["a::b"]));
        // Composes with the algebra like any other operand
        assert!(eval("a::*&!a::skip", &["a::keep", "b::skip"]));
        assert!(!eval("a::*&!a::skip", &["a::skip"]));
        assert!(eval("a::*|b::x", &["b::x"]));
    }

    /// Quoting escapes the expression operators; the quoted text is still one tag name, so it is
    /// normalized like any other and never spells a wildcard.
    #[test]
    fn quoted_names_are_names_and_never_wildcards() {
        assert!(eval("'rust'|x", &["rust"]));
        assert!(eval("\"rust\"", &["rust"]));
        assert!(eval("'a::b'", &["a::b"]));
        assert!(eval("中文|rust", &["中文"]));
        assert!(eval("'项目A.1'", &["项目A.1"]));
        // Surrounding whitespace trims away, as for any tag name
        assert!(eval("' rust '", &["rust"]));
        // A quoted wildcard is an invalid name, not a wildcard
        assert!(parse("'a::*'").is_err());
        // Names are identifier words: quoting cannot smuggle in a delimiter or an operator
        assert!(parse("'rust & life'").is_err());
        assert!(parse("'a/b'").is_err());
        assert!(parse("'a:b'").is_err());
        // Unterminated or empty quotes are syntax errors
        assert!(parse("'rust").is_err());
        assert!(parse("''").is_err());
        assert!(parse("'  '").is_err());
    }

    /// Scoped names are the only place a ':' may appear, and '*' only where it is the wildcard.
    #[test]
    fn scoped_name_syntax_errors() {
        let err = parse("a:b").unwrap_err();
        assert!(
            err.contains("at position 2") && err.contains("'scope::value'"),
            "got: {err}"
        );
        let err = parse("a::b::c").unwrap_err();
        assert!(
            err.contains("at most one '::'") && err.contains("at position 5"),
            "got: {err}"
        );
        let err = parse("a::").unwrap_err();
        assert!(
            err.contains("missing value after '::'") && err.contains("at position 2"),
            "got: {err}"
        );
        let err = parse("::*").unwrap_err();
        assert!(
            err.contains("missing scope name before '::'") && err.contains("at position 1"),
            "got: {err}"
        );
        for input in ["*", "a*", "a::*b", "a::b*"] {
            let err = parse(input).unwrap_err();
            assert!(
                err.contains("'*' is the value of a scope wildcard"),
                "{input}: got {err}"
            );
        }
        // '=' belongs to no part of the grammar any more
        let err = parse("a=b").unwrap_err();
        assert!(
            err.contains("at position 2") && err.contains("'scope::value'"),
            "got: {err}"
        );
        assert!(parse("=x").unwrap_err().contains("at position 1"));
        // A scoped name is an identifier word on both sides
        assert!(parse("a::-b").is_err());
        assert!(parse("-a::b").is_err());
    }

    /// `@name` stays a predicate reference: resolved before evaluation, never a tag name.
    #[test]
    fn named_references_resolve_before_evaluation() {
        let expr = parse("@Archived|x").unwrap();
        assert_eq!(expr.tag_names(), vec!["x"]);
        let expr = parse("@Archived").unwrap();
        assert!(matches!(expr, TagExpr::Atom(TagAtom::Named(name)) if name == "Archived"));
        assert!(parse("@").is_err());
        assert!(parse("@a b").is_err());
    }

    /// Regex atoms: pass when ANY of the memory's tag names matches; a slash opens a regex only
    /// at a token boundary, so a slash inside a word stays ordinary.
    #[test]
    fn regex_atoms() {
        assert!(eval("/^proj/", &["proj-alpha"]));
        assert!(!eval("/^proj/", &["web"]));
        // Any-tag OR: one matching name suffices
        assert!(eval("/^proj/", &["web", "proj-beta"]));
        // Scoped names match through the separator like any other text
        assert!(eval("/^system::/", &["system::pc1"]));
        assert!(!eval("/^system::/", &["lang::rust"]));
        // Composes with the algebra like any other operand
        assert!(eval("/^proj/&!misc", &["proj-alpha"]));
        assert!(!eval("/^proj/&!misc", &["proj-alpha", "misc"]));
        assert!(eval("a|/b+/", &["bbb"]));
        assert!(eval("x&/^项目/", &["x", "项目管理"]));
        // Escaped delimiter matches a literal slash; other backslash sequences reach the engine
        assert!(eval("/a\\/b/", &["a/b"]));
        assert!(eval("/\\d/", &["v2-x"]));
        assert!(!eval("/\\d/", &["no-digits"]));
        // Case-sensitive by default; inline flags opt into case folding
        assert!(!eval("/rust/", &["RUST"]));
        assert!(eval("/(?i)rust/", &["RUST"]));
        // A slash inside a word stays one name, which the name rule then rejects
        assert!(parse("proj/alpha").is_err());
        // Unterminated, empty and invalid regexes carry positions
        let err = parse("/abc").unwrap_err();
        assert!(
            err.contains("unterminated regular expression starting at position 1"),
            "got: {err}"
        );
        let err = parse("a&//").unwrap_err();
        assert!(
            err.contains("empty regular expression at position 3"),
            "got: {err}"
        );
        let err = parse("/([/").unwrap_err();
        assert!(
            err.contains("invalid regular expression '/([/' at position 1"),
            "got: {err}"
        );
    }

    #[test]
    fn syntax_errors_carry_positions() {
        let err = parse("(a|b").unwrap_err();
        assert!(
            err.contains("missing ')' for '(' at position 1"),
            "got: {err}"
        );
        let err = parse("a|").unwrap_err();
        assert!(err.contains("end of expression"), "got: {err}");
        let err = parse("a b").unwrap_err();
        assert!(err.contains("tag 'b' at position 3"), "got: {err}");
        let err = parse("()").unwrap_err();
        assert!(err.contains("empty group"), "got: {err}");
        let err = parse("&a").unwrap_err();
        assert!(err.contains("unexpected '&'"), "got: {err}");
        let err = parse("(a)b").unwrap_err();
        assert!(err.contains("tag 'b' at position 4"), "got: {err}");
        assert_eq!(parse("").unwrap_err(), "empty expression");
        assert_eq!(parse("   ").unwrap_err(), "empty expression");
    }

    #[test]
    fn leaf_names_are_unique_in_order() {
        let expr = parse("b&(a|b)&!c").unwrap();
        assert_eq!(expr.tag_names(), vec!["b", "a", "c"]);
        // A scoped name is one exact leaf
        let expr = parse("a::b&(a|a::b)").unwrap();
        assert_eq!(expr.tag_names(), vec!["a::b", "a"]);
        // Wildcards and regex atoms match dynamically: no literal leaves
        assert_eq!(parse("a::*&/^x/&b").unwrap().tag_names(), vec!["b"]);
    }

    /// Scope wildcards report the scopes the store must check, in first-appearance order.
    #[test]
    fn scope_names_are_unique_in_order() {
        let expr = parse("b::*&(a::*|b::*)&!c").unwrap();
        assert_eq!(expr.scope_names(), vec!["b", "a"]);
        assert_eq!(expr.tag_names(), vec!["c"]);
        // An exact scoped name is a full name, not a scope
        let expr = parse("a::b|!c::*").unwrap();
        assert_eq!(expr.tag_names(), vec!["a::b"]);
        assert_eq!(expr.scope_names(), vec!["c"]);
        assert!(parse("/^x/&y").unwrap().scope_names().is_empty());
    }

    #[test]
    fn eval_negation_over_tags() {
        // !a means "does not carry a" — the complement within one memory's tag set
        assert!(eval("!a", &[]));
        assert!(eval("!(a|b)", &["c"]));
        assert!(!eval("!(a|b)", &["b"]));
    }

    #[test]
    fn mutex_supports_zero_one_and_compound_operands() {
        eval_equivalent("mutex(a,b,c)", "!((a&b)|(a&c)|(b&c))");
        assert!(eval("mutex(a,b,c)", &[]));
        assert!(!eval("mutex(a,b,c)", &["a", "c"]));
        assert!(eval("mutex(a&b,c)", &["a", "c"]));
        assert!(!eval("mutex(a&b,c)", &["a", "b", "c"]));
        assert_eq!(
            parse("mutex(a&b,c)").unwrap().tag_names(),
            vec!["a", "b", "c"]
        );
        // A scope wildcard is one operand like any other
        assert!(eval("mutex(a::*,b)", &["b"]));
        assert!(!eval("mutex(a::*,b)", &["a::x", "b"]));
        assert!(eval("mutex", &["mutex"]));
        for input in ["mutex()", "mutex(a)", "mutex(a,)", "mutex(a,b", "a,b"] {
            assert!(parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn parser_bounds_untrusted_expression_size_and_depth() {
        assert!(parse(&"a".repeat(4097)).is_err());
        assert!(parse(&format!("{}a", "!".repeat(65))).is_err());
        assert!(parse(&format!("{}a{}", "(".repeat(65), ")".repeat(65))).is_err());
        assert!(parse(&vec!["a"; 130].join("|")).is_err());
        assert!(parse(&format!("{}a", "!".repeat(32))).is_ok());
    }
}
