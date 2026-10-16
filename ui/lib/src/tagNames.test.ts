import { describe, expect, it } from 'vitest'
import { validTagName, validTagValue } from './tagNames'

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
    expect(validTagName('"quoted"')).toBe(false)
    expect(validTagName('-lead')).toBe(false) // '-' is not an identifier start
    expect(validTagName('')).toBe(false)
    expect(validTagName('   ')).toBe(false)
    expect(validTagName('x'.repeat(101))).toBe(false)
  })

  it('value halves: identifier words or anything the number conversion accepts', () => {
    expect(validTagValue('rust')).toBe(true)
    expect(validTagValue('1')).toBe(true)
    expect(validTagValue('1.5')).toBe(true)
    expect(validTagValue('-2')).toBe(true)
    expect(validTagValue('1e5')).toBe(true)
    // Number() converts these to legal JSON numbers on save, so they are not illegal
    expect(validTagValue('0x10')).toBe(true)
    expect(validTagValue('01')).toBe(true)
    expect(validTagValue('va(lid')).toBe(false)
    expect(validTagValue('1+2')).toBe(false)
    expect(validTagValue('1 2')).toBe(false)
    expect(validTagValue('')).toBe(false)
    expect(validTagValue('   ')).toBe(false)
    expect(validTagValue('a'.repeat(101))).toBe(false)
  })
})
