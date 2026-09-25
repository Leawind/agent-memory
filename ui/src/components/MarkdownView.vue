<template>
  <!-- Rendered locally from the user-authored markdown source. -->
  <div class="md-body" v-html="html" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '../markdown'

const props = defineProps<{ source: string }>()
const html = computed(() => renderMarkdown(props.source))
</script>

<!--
  Styles are global but namespaced under .md-body: v-html children carry no
  scope attribute, so scoped styles cannot reach them.
-->
<style>
.md-body {
  font-size: 14px;
  line-height: 1.7;
  word-break: break-word;
}
.md-body > :first-child {
  margin-top: 0;
}
.md-body > :last-child {
  margin-bottom: 0;
}
.md-body h1,
.md-body h2,
.md-body h3,
.md-body h4,
.md-body h5,
.md-body h6 {
  margin: 1.2em 0 0.5em;
  font-weight: 600;
  line-height: 1.4;
}
.md-body h1 {
  font-size: 1.5em;
}
.md-body h2 {
  font-size: 1.3em;
}
.md-body h3 {
  font-size: 1.15em;
}
.md-body h4,
.md-body h5,
.md-body h6 {
  font-size: 1em;
}
.md-body p {
  margin: 0.6em 0;
}
.md-body ul,
.md-body ol {
  margin: 0.6em 0;
  padding-left: 1.6em;
}
.md-body li {
  margin: 0.2em 0;
}
.md-body li > input[type='checkbox'] {
  margin-right: 0.4em;
}
.md-body code {
  font-family: var(--el-font-family-mono, ui-monospace, Menlo, Consolas, monospace);
  font-size: 0.9em;
  background: var(--el-fill-color-light);
  border-radius: 4px;
  padding: 0.15em 0.4em;
}
.md-body pre {
  background: var(--el-fill-color-light);
  border-radius: 6px;
  padding: 10px 12px;
  overflow-x: auto;
}
.md-body pre code {
  background: none;
  padding: 0;
  font-size: 13px;
}
.md-body blockquote {
  margin: 0.6em 0;
  padding: 0.2em 0 0.2em 12px;
  border-left: 3px solid var(--el-border-color);
  color: var(--el-text-color-secondary);
}
.md-body table {
  border-collapse: collapse;
  margin: 0.8em 0;
}
.md-body th,
.md-body td {
  border: 1px solid var(--el-border-color-lighter);
  padding: 5px 12px;
}
.md-body th {
  background: var(--el-fill-color-light);
  font-weight: 600;
}
.md-body a {
  color: var(--el-color-primary);
}
.md-body img {
  max-width: 100%;
}
.md-body hr {
  border: none;
  border-top: 1px solid var(--el-border-color-lighter);
  margin: 1.2em 0;
}
</style>
