// @vitest-environment jsdom
// DOMPurify mis-parses under happy-dom (strips common block tags), so the
// sanitize assertions run against jsdom, its officially supported test DOM.
import { describe, expect, it } from 'vitest'
import { renderMarkdown, sanitizeHtml } from './markdown'

describe('renderMarkdown', () => {
  it('renders headings, emphasis and lists', () => {
    expect(renderMarkdown('# 标题')).toContain('<h1>标题</h1>')
    expect(renderMarkdown('**bold**')).toContain('<strong>bold</strong>')
    expect(renderMarkdown('- a\n- b')).toContain('<ul>')
  })

  it('renders GFM tables', () => {
    const html = renderMarkdown('| a | b |\n| --- | --- |\n| 1 | 2 |')
    expect(html).toContain('<table>')
    expect(html).toContain('<th>a</th>')
  })

  it('renders fenced code blocks without executing them', () => {
    const html = renderMarkdown('```rust\nfn main() {}\n```')
    expect(html).toContain('<pre>')
    expect(html).toContain('<code')
  })

  it('strips script tags and event handlers from stored content', () => {
    const html = renderMarkdown('hi <script>alert(1)</script> <img src=x onerror=alert(1)>')
    expect(html).not.toContain('<script')
    expect(html).not.toContain('alert(1)')
    expect(html).not.toContain('onerror')
  })

  it('removes javascript: link targets', () => {
    const html = renderMarkdown('[x](javascript:alert(1))')
    expect(html).not.toContain('javascript:')
  })

  it('keeps plain text intact', () => {
    expect(renderMarkdown('纯文本记忆内容')).toContain('纯文本记忆内容')
  })
})

describe('sanitizeHtml', () => {
  it('keeps server-side <mark> search highlights', () => {
    expect(sanitizeHtml('命中 <mark>关键词</mark> 片段')).toContain('<mark>关键词</mark>')
  })

  it('strips scripts and handlers from HTML fragments', () => {
    const html = sanitizeHtml('<b>ok</b><script>alert(1)</script><img src=x onerror=alert(1)>')
    expect(html).toContain('<b>ok</b>')
    expect(html).not.toContain('<script')
    expect(html).not.toContain('onerror')
  })
})
