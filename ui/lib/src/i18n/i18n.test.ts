// i18n: one-to-one zh/en dictionary keys, interpolation and language switching
import { describe, expect, it } from 'vitest'
import { memoryUIi18n, setMemoryUILocale, currentMemoryUILocale, t } from './index'
import { elementPlusLocale } from './elementPlus'
import zh from './zh'
import en from './en'

type Node = Record<string, unknown>

function flatten(obj: Node, prefix = ''): string[] {
  return Object.entries(obj).flatMap(([k, v]) =>
    v != null && typeof v === 'object' ? flatten(v as Node, `${prefix}${k}.`) : [`${prefix}${k}`],
  )
}

describe('i18n messages', () => {
  it('zh and en dictionaries have identical key sets', () => {
    expect(flatten(en as Node).sort()).toEqual(flatten(zh as Node).sort())
  })

  it('no message is an empty string', () => {
    for (const key of flatten(zh as Node)) {
      expect(String(t(key)).trim()).not.toBe('')
    }
  })

  it('renders all tag-rule operators and the full derivation restrictions', () => {
    for (const locale of ['zh', 'en'] as const) {
      setMemoryUILocale(locale)
      expect(t('access.tagRulesHint')).toContain('!child|parent')
      expect(t('access.tagRulesHint')).toContain('100')
      expect(t('access.derivationsHint')).toContain('|')
      expect(t('access.derivationsHint')).toContain('convention')
    }
    setMemoryUILocale('zh')
  })
})

describe('locale switching', () => {
  it('setMemoryUILocale switches t() output and is reactive in both languages', () => {
    setMemoryUILocale('zh')
    expect(currentMemoryUILocale()).toBe('zh')
    expect(t('memories.title')).toBe('记忆')
    setMemoryUILocale('en')
    expect(t('memories.title')).toBe('Memories')
    setMemoryUILocale('zh')
  })

  it('t() interpolates named params', () => {
    expect(t('memories.deleteConfirm', { id: 'm7' })).toBe('确定永久删除记忆 m7？')
    expect(t('editor.titleFmt', { id: 'm7' })).toBe('记忆 #m7')
    expect(t('tags.titleFmt', { name: 'rust' })).toBe('标签 #rust')
    expect(t('access.doctorFail', { count: 3, failed: 2, total: 10 })).toBe('发现 3 个问题（2/10 项检查未通过）')
  })

  it('the shared composer exposes t for use outside component setup', () => {
    // pure TS modules such as format.ts / client.ts depend on this shape
    expect(typeof memoryUIi18n.global.t).toBe('function')
  })

  it('the Element Plus locale follows the library locale', () => {
    setMemoryUILocale('zh')
    expect(elementPlusLocale.value.name).toBe('zh-cn')
    setMemoryUILocale('en')
    expect(elementPlusLocale.value.name).toBe('en')
    setMemoryUILocale('zh')
  })
})
