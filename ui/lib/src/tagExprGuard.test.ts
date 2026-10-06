// Unit tests for the debounced-typing gate: an expression under composition must not hit the
// server (it would 400 on every keystroke), while complete expressions pass. The explicit
// apply path bypasses this gate, so "not ready" is a soft wait, never an error.
import { describe, expect, it } from 'vitest'
import { tagExprReady } from './tagExprGuard.js'

const TAGS = ['rust', 'web', 'notes', 'conventions', 'rust & life']

describe('tagExprReady', () => {
  it('accepts complete expressions over known tags', () => {
    expect(tagExprReady('rust', TAGS)).toBe(true)
    expect(tagExprReady('(rust&web)|notes', TAGS)).toBe(true)
    expect(tagExprReady('!rust& (web||notes)', TAGS)).toBe(true)
    expect(tagExprReady("'rust & life'&!web", TAGS)).toBe(true)
  })

  it('waits for expressions still being composed', () => {
    // Trailing bare word that is not (yet) a known tag
    expect(tagExprReady('rust&web|n', TAGS)).toBe(false)
    expect(tagExprReady('rust&web|', TAGS)).toBe(false)
    expect(tagExprReady('zzz', TAGS)).toBe(false)
    expect(tagExprReady('rust&zz', TAGS)).toBe(false)
    // Unterminated quote or group
    expect(tagExprReady("'rust", TAGS)).toBe(false)
    expect(tagExprReady('(rust|web', TAGS)).toBe(false)
    // Dangling operators and empty groups
    expect(tagExprReady('rust&', TAGS)).toBe(false)
    expect(tagExprReady('|rust', TAGS)).toBe(false)
    expect(tagExprReady('rust&|web', TAGS)).toBe(false)
    expect(tagExprReady('()', TAGS)).toBe(false)
    expect(tagExprReady('rust)', TAGS)).toBe(false)
  })

  it('a trailing bare word applies once it becomes a known tag', () => {
    // Typing "rust&web|notes" char by char: ready exactly when the word completes
    expect(tagExprReady('rust&web|note', TAGS)).toBe(false)
    expect(tagExprReady('rust&web|notes', TAGS)).toBe(true)
    // Same for the first word
    expect(tagExprReady('ru', TAGS)).toBe(false)
    expect(tagExprReady('rust', TAGS)).toBe(true)
  })

  it('empty or whitespace-only expressions are always sendable', () => {
    expect(tagExprReady('', TAGS)).toBe(true)
    expect(tagExprReady('   ', TAGS)).toBe(true)
  })

  it('quoted unknown names wait; escaped characters inside quotes are decoded for the check', () => {
    expect(tagExprReady("'nope'", TAGS)).toBe(false)
    expect(tagExprReady("'rust & life'", TAGS)).toBe(true)
  })

  it('regex atoms: unterminated waits, closed passes regardless of matching names', () => {
    // Still typing the pattern
    expect(tagExprReady('rust&/pro', TAGS)).toBe(false)
    // Closed regex is dynamic — no known-name requirement
    expect(tagExprReady('rust&/proj/', TAGS)).toBe(true)
    expect(tagExprReady('/^proj/', TAGS)).toBe(true)
    expect(tagExprReady('/^proj/&!web', TAGS)).toBe(true)
    expect(tagExprReady('rust|/zzz-nothing-matches/', TAGS)).toBe(true)
    // Escaped delimiter does not close the atom; a slash inside a bare word is literal
    expect(tagExprReady('/a\\/b', TAGS)).toBe(false)
    expect(tagExprReady('/a\\/b/', TAGS)).toBe(true)
    expect(tagExprReady('proj/alpha', TAGS)).toBe(false) // unknown tag
  })
})
