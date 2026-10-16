// Tag-name legality, mirrored from the server (src/model.rs): a name or key is one word of
// Unicode identifier characters (XID_Continue — letters, digits and `_` in every script) with
// `-` and `.` allowed after the first character. Validating chips the moment they exist keeps
// illegal input from surfacing only as a server error on save.

export const MAX_TAG_NAME_CHARS = 100

const IDENT_RE = /^[\p{XID_Continue}][\p{XID_Continue}\-.]*$/u

/** A bare tag name or the key half of `key:value`; surrounding whitespace is trimmed first, like the server does */
export function validTagName(raw: string): boolean {
  const name = raw.trim()
  if (name.length === 0 || [...name].length > MAX_TAG_NAME_CHARS) return false
  return IDENT_RE.test(name)
}

/** The value half of `key:value`: an identifier word, or anything the editor's number conversion
 * (Number(), finite) turns into a JSON number — hex and leading-zero spellings included, since
 * that conversion is what actually happens on save. */
export function validTagValue(raw: string): boolean {
  const value = raw.trim()
  if (value === '') return false
  if (Number.isFinite(Number(value))) return true
  return validTagName(value)
}
