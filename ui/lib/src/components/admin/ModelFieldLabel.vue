<template>
  <span class="field-label">
    <span><span v-if="required" class="field-required" aria-hidden="true">* </span>{{ label }}</span>
    <el-tooltip v-if="hint" :content="hint" placement="top" :trigger="['hover', 'focus']">
      <el-icon class="field-help" tabindex="0" :aria-label="`${label}: ${hint}`"><InfoFilled /></el-icon>
    </el-tooltip>
  </span>
</template>

<script setup lang="ts">
import { InfoFilled } from '@element-plus/icons-vue'

defineProps<{ label: string; hint?: string; required?: boolean }>()
</script>

<style scoped>
.field-label {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.field-help {
  cursor: help;
  color: var(--el-text-color-placeholder);
}
.field-required {
  color: var(--el-color-danger);
}
.field-help:focus-visible {
  outline: 2px solid var(--el-color-primary);
  outline-offset: 2px;
  border-radius: 50%;
}
</style>
