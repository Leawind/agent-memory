<template>
  <div class="memory-ui">
    <el-config-provider :locale="elementPlusLocale">
      <div class="console">
        <div class="console-header">
          <!-- Click the title to open the server overview (OpsDialog) -->
          <button type="button" class="brand-btn" @click="opsVisible = true">
            <span class="brand-mark">
              <el-icon :size="16"><Collection /></el-icon>
            </span>
            <span>{{ title }}</span>
          </button>
        </div>
        <MemoryWorkspace class="console-body" />
      </div>
      <OpsDialog :visible="opsVisible" @update:visible="opsVisible = $event" />
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { Collection } from '@element-plus/icons-vue'
import MemoryWorkspace from './MemoryWorkspace.vue'
import OpsDialog from './OpsDialog.vue'

/** Root admin console: brand title on top, the workspace (tag sidebar + memory list) below.
 * Height fills the viewport on a standalone site; when embedded, override with
 * --memory-admin-height (e.g. 480px). */
withDefaults(defineProps<{ title?: string }>(), { title: 'Agent Memory' })

// Service overview dialog: opens by clicking the brand title
const opsVisible = ref(false)
</script>

<style scoped>
/* Fill the viewport on the standalone site; when embedded, override with --memory-admin-height (e.g. 480px / auto) */
.memory-ui {
  height: var(--memory-admin-height, 100%);
}
.console {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.console-header {
  display: flex;
  align-items: center;
  padding: 14px 20px 10px;
  flex-shrink: 0;
}
/* Title button: opens the server overview; restore plain-text look (negative margin cancels button padding), surface color on hover */
.brand-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: -4px -8px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--el-border-radius-base);
  background: transparent;
  color: var(--el-text-color-primary);
  font: inherit;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: -0.01em;
  cursor: pointer;
}
.brand-btn:hover {
  background: var(--el-fill-color);
}
/* Brand icon: rounded block with the brand-green gradient (Modrinth's --brand-gradient-bg) */
.brand-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  color: var(--el-color-primary);
  background: linear-gradient(135deg, rgba(68, 182, 138, 0.25) 0%, rgba(58, 250, 112, 0.18) 100%);
}
.console-body {
  flex: 1;
  min-height: 0;
  padding: 0 20px 20px;
  box-sizing: border-box;
}
</style>
