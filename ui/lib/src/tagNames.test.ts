import { describe, expect, it } from 'vitest'
import { validFullTagName, validTagName, parseScopedName } from './tagNames'

describe('tag name rules (mirroring src/model.rs)', () => {
  it('accepts identifier words: ascii, CJK, with - or . after the first character', () => {
    expect(validTagName('rust')).toBe(true)
    expect(validTagName('am-debug')).toBe(true)
    expect(validTagName('a.b')).toBe(true)
    expect(validTagName('架构')).toBe(true)
    expect(validTagName('_x')).toBe(true)
    expect(validTagName(' rust ')).toBe(true) // trimmed first, like the server
    expect(validTagName('构'.repeat(100))).toBe(true)
  })

  it('rejects operators, spaces, colons, empty and overlong names', () => {
    expect(validTagName('am-debug)invalid')).toBe(false)
    expect(validTagName('a b')).toBe(false)
    expect(validTagName('a:b')).toBe(false)
    expect(validTagName('a::b')).toBe(false)
    expect(validTagName('"quoted"')).toBe(false)
    expect(validTagName('-lead')).toBe(false) // '-' is not an identifier start
    expect(validTagName('')).toBe(false)
    expect(validTagName('   ')).toBe(false)
    expect(validTagName('x'.repeat(101))).toBe(false)
  })
})

describe('scoped names', () => {
  it('splits scope::value into identifier-word halves', () => {
    expect(parseScopedName('system::pc1sw0')).toEqual({ scope: 'system', value: 'pc1sw0' })
    expect(parseScopedName('phase::design')).toEqual({ scope: 'phase', value: 'design' })
    expect(parseScopedName(' rust ::x ')).toEqual({ scope: 'rust', value: 'x' })
  })

  it('rejects bare names, empty halves and repeated separators', () => {
    expect(parseScopedName('rust')).toBeNull()
    expect(parseScopedName('system::')).toBeNull()
    expect(parseScopedName('::pc1sw0')).toBeNull()
    expect(parseScopedName('a::b::c')).toBeNull()
    expect(parseScopedName('')).toBeNull()
  })

  it('full names: a bare identifier word or scope::value with legal halves', () => {
    expect(validFullTagName('rust')).toBe(true)
    expect(validFullTagName('system::pc1sw0')).toBe(true)
    expect(validFullTagName('score::3')).toBe(true)
    expect(validFullTagName('a::b::c')).toBe(false)
    expect(validFullTagName('am-debug)invalid')).toBe(false)
    expect(validFullTagName('bad scope::x')).toBe(false)
    expect(validFullTagName('system::va(lid')).toBe(false)
    expect(validFullTagName('')).toBe(false)
    expect(validFullTagName('   ')).toBe(false)
  })
})
