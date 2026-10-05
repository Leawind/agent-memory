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
//! primary := "(" expr ")" | tag
//! tag     := quoted | bare
//! quoted  := '"' (escape | char)* '"' | "'" (escape | char)* "'"
//! bare    := 1+ chars outside whitespace, parentheses, "&|!", quotes
//! ```
//!
//! Parsing is pure syntax; leaf names are validated against the store by the caller (unknown
//! names are an error with a did-you-mean hint, mirroring tag linking on writes).

/// A parsed expression: one node per operator, leaves are tag names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagExpr {
    Tag(String),
    Not(Box<TagExpr>),
    All(Box<TagExpr>, Box<TagExpr>),
    Any(Box<TagExpr>, Box<TagExpr>),
}

impl TagExpr {
    /// Evaluate against one memory's tag set: `has` answers "does this memory carry the tag?".
    pub fn eval(&self, has: &impl Fn(&str) -> bool) -> bool {
        match self {
            TagExpr::Tag(name) => has(name),
            TagExpr::Not(inner) => !inner.eval(has),
            TagExpr::All(a, b) => a.eval(has) && b.eval(has),
            TagExpr::Any(a, b) => a.eval(has) || b.eval(has),
        }
    }

    /// Leaf tag names in first-appearance order, deduplicated (for store-side existence checks).
    pub fn tag_names(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        self.collect_names(&mut out);
        out
    }

    fn collect_names<'a>(&'a self, out: &mut Vec<&'a str>) {
        match self {
            TagExpr::Tag(name) => {
                if !out.contains(&name.as_str()) {
                    out.push(name);
                }
            }
            TagExpr::Not(inner) => inner.collect_names(out),
            TagExpr::All(a, b) | TagExpr::Any(a, b) => {
                a.collect_names(out);
                b.collect_names(out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    LParen,
    RParen,
    Amp,
    Pipe,
    Bang,
    Tag(String),
}

/// Parse an expression; errors carry the char position (1-based) of the offending token.
pub fn parse(input: &str) -> Result<TagExpr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("empty expression".to_string());
    }
    let mut p = Parser { tokens, pos: 0 };
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
            _ => {
                let mut name = String::new();
                while i < chars.len() {
                    let ch = chars[i];
                    if ch.is_whitespace() || "()&|!\"'".contains(ch) {
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
        Tok::Tag(name) => format!("tag '{name}'"),
    }
}

struct Parser {
    tokens: Vec<(Tok, usize)>,
    pos: usize,
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
        if matches!(self.peek(), Some((Tok::Bang, _))) {
            self.pos += 1;
            let inner = self.parse_unary()?;
            return Ok(TagExpr::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<TagExpr, String> {
        let (tok, pos) = self
            .next()
            .ok_or_else(|| "unexpected end of expression: a tag or '(' was expected".to_string())?;
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
            Tok::Tag(name) => Ok(TagExpr::Tag(name)),
            other => Err(format!(
                "unexpected {} at position {pos}: a tag or '(' was expected",
                describe(&other)
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(expr: &str, tags: &[&str]) -> bool {
        parse(expr)
            .unwrap_or_else(|e| panic!("parse '{expr}' failed: {e}"))
            .eval(&|n| tags.contains(&n))
    }

    #[test]
    fn operators_and_precedence() {
        // The user-facing example: both a and b, or c alone
        assert!(eval("(a&b)|c", &["a", "b"]));
        assert!(eval("(a&b)|c", &["c"]));
        assert!(!eval("(a&b)|c", &["a"]));
        // & binds tighter than |
        assert_eq!(parse("a|b&c").unwrap(), parse("a|(b&c)").unwrap());
        assert_eq!(parse("a&b|c&d").unwrap(), parse("(a&b)|(c&d)").unwrap());
        // ! binds tighter than &
        assert_eq!(parse("!a&b").unwrap(), parse("(!a)&b").unwrap());
        assert!(eval("!a&b", &["b"]));
        assert!(!eval("!a&b", &["a", "b"]));
        // Double-character aliases spell the same operators
        assert_eq!(parse("a&&b||c").unwrap(), parse("(a&b)|c").unwrap());
        // Chains flatten into left-associative trees
        assert_eq!(parse("a&b&c").unwrap(), parse("(a&b)&c").unwrap());
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
    }

    #[test]
    fn eval_negation_over_tags() {
        // !a means "does not carry a" — the complement within one memory's tag set
        assert!(eval("!a", &[]));
        assert!(eval("!(a|b)", &["c"]));
        assert!(!eval("!(a|b)", &["b"]));
    }
}
