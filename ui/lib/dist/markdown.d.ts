/** Parse markdown into sanitized HTML (synchronous). */
export declare function renderMarkdown(source: string): string;
/**
 * Sanitize an already-HTML fragment (e.g. server-generated search snippets,
 * whose <mark> highlights must survive). Every v-html entry point must go
 * through this or renderMarkdown — no exceptions.
 */
export declare function sanitizeHtml(html: string): string;
