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

<!-- Typography styles live in src/styles/markdown.css (global .md-body namespace, shipped with the library entry) -->
