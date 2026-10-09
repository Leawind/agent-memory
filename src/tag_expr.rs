//! Tag set algebra: parse and evaluate boolean expressions over tag names.
//!
//! Every tag is a set of memories, so a tag expression is a set expression: `(a&b)|c` selects
//! memories carrying both `a` and `b`, or carrying `c`. Grammar (precedence `!` > `&` > `|`,
//! parentheses group, `&&` / `||` accepted as aliases):
//!
//! ```text
//! expr    := or
//! or      := and ("|" | "||" and)*
//! and     := unary ("&" | "&&" unary)*
//! unary   := ("!" unary) | primary
//! primary := "(" expr ")" | "mutex(" expr ("," expr)+ ")" | tag | regex
//! tag     := quoted | bare
//! quoted  := '"' (escape | char)* '"' | "'" (escape | char)* "'"
//! bare    := 1+ chars outside whitespace, parentheses, "&|!", quotes
//! regex   := "/" (escape | char)* "/"
//! ```
//!
//! A regex atom passes when ANY of the memory's tag names matches (Rust regex syntax, case
//! sensitive by default — inline flags like `(?i)` work). A `/` opens a regex only at a token
//! boundary; inside a bare word it stays literal, so `proj/alpha` is still one tag name. Inside
//! a regex, `\/` escapes the delimiter and every other backslash sequence passes through to the
//! regex engine untouched (`\d`, `\\`, ...).
//!
//! Parsing is pure syntax; leaf names are validated against the store by the caller (unknown
//! names are an error with a did-you-mean hint, mirroring tag linking on writes). Regex atoms
//! carry no names and match dynamically.

use crate::predicate::Predicate;

pub type TagExpr = Predicate<TagAtom>;

/// Query atoms use names; the rule compiler binds literal atoms to stable ids.
#[derive(Debug, Clone)]
pub enum TagAtom {
    Tag(String),
    Named(String),
    Regex(Box<regex::Regex>),
}

impl TagExpr {
    /// Evaluate against one memory's tag set: a leaf passes when the set carries the name, a
    /// regex when any name in the set matches.
    pub fn eval(&self, tags: &[String]) -> bool {
        self.eval_with(&|atom| match atom {
            TagAtom::Tag(name) => tags.iter().any(|t| t == name),
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
                if name.is_empty() {
                    return Err(format!("empty quoted tag name at position {pos}"));
                }
                out.push((Tok::Tag(name), pos));
            }
            c if c.is_whitespace() => {}
            '/' => {
                // A slash at a token boundary opens a regex atom; inside a bare word (handled
                // below) it stays literal, so 'proj/alpha' still parses as one tag name
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
                let mut name = String::new();
                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_whitespace() || "(),&|!\"'".contains(ch) {
                        break;
                    }
                    name.push(ch);
                    i += 1;
                }
                i -= 1;
                out.push((Tok::Tag(name), pos));
            }
        }
        i += 1;
    }
    Ok(out)
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
            Tok::Tag(name) if name.starts_with('@') => {
                let reference = &name[1..];
                let (scope, raw) = reference
                    .split_once("::")
                    .map_or((None, reference), |(scope, name)| (Some(scope), name));
                if scope.is_some_and(|scope| scope != "global" && scope != "user") {
                    return Err("predicate scope must be global or user".into());
                }
                crate::model::normalize_tag_name(raw)?;
                Ok(TagExpr::Atom(TagAtom::Named(reference.to_string())))
            }
            Tok::Tag(name) => Ok(TagExpr::Atom(TagAtom::Tag(name))),
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

    #[test]
    fn quoted_and_unicode_names() {
        // Names containing operators, whitespace or quotes need quoting; escapes keep literals
        assert!(eval("'rust & life'|x", &["rust & life"]));
        assert!(eval("\"项目 管理\"|x", &["项目 管理"]));
        assert!(eval("'it\\'s'|x", &["it's"]));
        assert!(eval("'a\\\\b'|x", &["a\\b"]));
        assert!(eval("中文|rust", &["中文"]));
        // Unterminated or empty quotes are syntax errors
        assert!(parse("'rust").is_err());
        assert!(parse("''").is_err());
    }

    /// Regex atoms: pass when ANY of the memory's tag names matches; a slash opens a regex only
    /// at a token boundary, so slashes inside bare words stay literal tag characters.
    #[test]
    fn regex_atoms() {
        assert!(eval("/^proj/", &["proj/alpha"]));
        assert!(!eval("/^proj/", &["web"]));
        // Any-tag OR: one matching name suffices
        assert!(eval("/^proj/", &["web", "proj/beta"]));
        // Composes with the algebra like any other operand
        assert!(eval("/^proj/&!misc", &["proj/alpha"]));
        assert!(!eval("/^proj/&!misc", &["proj/alpha", "misc"]));
        assert!(eval("a|/b+/", &["bbb"]));
        assert!(eval("x&/^项目/", &["x", "项目 管理"]));
        // Escaped delimiter matches a literal slash; other backslash sequences reach the engine
        assert!(eval("/a\\/b/", &["a/b"]));
        assert!(eval("/\\d/", &["v2-x"]));
        assert!(!eval("/\\d/", &["no-digits"]));
        // Case-sensitive by default; inline flags opt into case folding
        assert!(!eval("/rust/", &["RUST"]));
        assert!(eval("/(?i)rust/", &["RUST"]));
        // A slash inside a bare word stays a tag character (one leaf, not a regex)
        let expr = parse("proj/alpha").unwrap();
        assert_eq!(expr.tag_names(), vec!["proj/alpha"]);
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
        // Regex atoms match dynamically: no literal leaves
        assert_eq!(parse("/^x/&a").unwrap().tag_names(), vec!["a"]);
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
