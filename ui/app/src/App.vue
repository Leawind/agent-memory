<template>
  <MemoryAdmin layout="sidebar" title="agent-memory">
    <template #footer>
      <el-segmented v-model="theme" :options="themeOptions" size="small" />
    </template>
  </MemoryAdmin>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { MemoryAdmin, provideMemoryUI } from '@agent-memory/ui'

// 同源部署：API 走默认配置（baseUrl = ''）
provideMemoryUI({})

type ThemeChoice = 'light' | 'dark' | 'system'
const themeOptions = [
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
  { label: '跟随系统', value: 'system' },
]
const STORAGE_KEY = 'agent-memory-theme'

const theme = ref<ThemeChoice>(readStoredTheme())
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)')

function readStoredTheme(): ThemeChoice {
  try {
    const v = localStorage.getItem(STORAGE_KEY)
    return v === 'light' || v === 'dark' || v === 'system' ? v : 'system'
  } catch {
    return 'system'
  }
}

function applyTheme() {
  const dark = theme.value === 'dark' || (theme.value === 'system' && prefersDark.matches)
  document.documentElement.classList.toggle('dark', dark)
}

watch(theme, () => {
  try {
    localStorage.setItem(STORAGE_KEY, theme.value)
  } catch {
    /* 隐私模式等场景存不了就算了 */
  }
  applyTheme()
})

onMounted(() => {
  applyTheme()
  // 跟随系统时监听系统主题变化
  prefersDark.addEventListener?.('change', () => {
    if (theme.value === 'system') applyTheme()
  })
})
</script>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
}
</style>
