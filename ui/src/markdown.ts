/**
 * Client-side markdown rendering for memory content (GFM: tables,
 * strikethrough, task lists). Output is sanitized before it reaches
 * v-html: memories are written by many agents, and the admin UI renders
 * whatever is stored, so script/handler/javascript-URL stripping is not
 * optional here.
 */
import { Marked } from 'marked'
import DOMPurify from 'dompurify'

const marked = new Marked()

/** Parse markdown into sanitized HTML (synchronous). */
export function renderMarkdown(source: string): string {
  const raw = marked.parse(source, { async: false }) as string
  return DOMPurify.sanitize(raw)
}
