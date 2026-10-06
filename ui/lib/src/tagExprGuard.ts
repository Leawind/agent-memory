// Local completeness gate for tag expressions (the debounced-typing path only).
//
// While an expression is being composed — unbalanced quotes or parentheses, dangling
// operators, or a trailing bare word that is not a known tag — sending it would only draw a
// server-side unknown-tag error toast on every keystroke. The gate mirrors the server's
// tag_expr tokenizer (src/tag_expr.rs) at "send or wait" granularity: it never rewrites the
// expression, and unknown names are not an error here, just a reason to wait.
//
// The explicit-apply path (Enter / suggestion pick / clear) bypasses this gate entirely: an
// expression the user deliberately applied deserves the server's precise did-you-mean error.

export function tagExprReady(expr: string, knownTags: string[]): boolean {
  const chars = [...expr]
  const known = new Set(knownTags)
  let balance = 0
  // true = the next token must be an operand (start of expression / after an operator / after '(')
  let expectOperand = true
  let sawToken = false
  let i = 0
  while (i < chars.length) {
    const c = chars[i]!
    if (/\s/.test(c)) {
      i++
      continue
    }
    sawToken = true
    if (c === '(') {
      if (!expectOperand) return false
      balance++
      i++
      continue
    }
    if (c === ')') {
      if (expectOperand || balance === 0) return false
      balance--
      i++
      continue
    }
    if (c === '&' || c === '|') {
      if (expectOperand) return false
      if (chars[i + 1] === c) i++ // && / || spell the same operators
      expectOperand = true
      i++
      continue
    }
    if (c === '!') {
      if (!expectOperand) return false
      i++
      continue // still expecting an operand
    }
    if (c === '"' || c === "'") {
      if (!expectOperand) return false
      const quote = c
      i++
      let name = ''
      let closed = false
      while (i < chars.length) {
        const ch = chars[i]!
        if (ch === '\\' && chars[i + 1] !== undefined) {
          name += chars[i + 1]!
          i += 2
          continue
        }
        if (ch === quote) {
          closed = true
          i++
          break
        }
        name += ch
        i++
      }
      // Unterminated quote = still typing
      if (!closed) return false
      if (!known.has(name)) return false
      expectOperand = false
      continue
    }
    if (c === '/') {
      // A slash at a token boundary opens a regex atom (mirrors the server tokenizer); scan to
      // the closing delimiter, letting every backslash pair pass through — unterminated = still
      // typing. Regexes match dynamically, so no known-name check applies.
      if (!expectOperand) return false
      i++
      let closed = false
      while (i < chars.length) {
        const ch = chars[i]!
        if (ch === '\\') {
          if (chars[i + 1] !== undefined) i += 2
          else i++
          continue
        }
        if (ch === '/') {
          closed = true
          i++
          break
        }
        i++
      }
      if (!closed) return false
      expectOperand = false
      continue
    }
    // Bare word: a run of non-delimiter characters
    if (!expectOperand) return false
    let name = ''
    while (i < chars.length && !/[\s()&|!"']/.test(chars[i]!)) {
      name += chars[i]!
      i++
    }
    // A trailing bare word may still be growing: only proceed when it is already a known tag
    // ("rust" applies on pause; "rust&web|n" waits). A complete word must be known too —
    // unknown names wait for the explicit apply, where the server explains the typo.
    if (!known.has(name)) return false
    expectOperand = false
  }
  // Empty / whitespace-only counts as "no expression" — always sendable
  return !sawToken || (balance === 0 && !expectOperand)
}
