// Tag-name legality, mirrored from the server (src/model.rs): a name is one word of Unicode
// identifier characters (XID_Continue — letters, digits and `_` in every script) with `-` and `.`
// allowed after the first character. A tag is bare (`rust`) or scoped (`system::pc1sw0`), where
// both halves are identifier words and the scope must exist as its own entity. Validating chips
// the moment they exist keeps illegal input from surfacing only as a server error on save.

export const MAX_TAG_NAME_CHARS = 100

const IDENT_RE = /^[\p{XID_Continue}][\p{XID_Continue}\-.]*$/u

/** A bare tag name or a scope name; surrounding whitespace is trimmed first, like the server does */
export function validTagName(raw: string): boolean {
  const name = raw.trim()
  if (name.length === 0 || [...name].length > MAX_TAG_NAME_CHARS) return false
  return IDENT_RE.test(name)
}

/** Split `scope::value` into its halves; null for bare names and malformed splits (the separator
 *  may appear at most once — a scope name is a single identifier word). */
export function parseScopedName(raw: string): { scope: string; value: string } | null {
  const name = raw.trim()
  const i = name.indexOf('::')
  if (i <= 0) return null
  const rest = name.slice(i + 2)
  if (rest.includes('::')) return null
  const scope = name.slice(0, i).trim()
  const value = rest.trim()
  if (value === '') return null
  return { scope, value }
}

/** A full tag name as typed into the tag dialog: a bare identifier word, or `scope::value` with
 *  identifier-word halves. */
export function validFullTagName(raw: string): boolean {
  const name = raw.trim()
  if (name === '') return false
  const scoped = parseScopedName(name)
  if (!scoped) return validTagName(name)
  return validTagName(scoped.scope) && validTagName(scoped.value)
}
