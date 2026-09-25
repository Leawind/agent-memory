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

<!-- 排版样式在 src/styles/markdown.css（全局 .md-body 命名空间，随库出口分发） -->
