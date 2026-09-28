<template>
  <div class="memory-ui">
    <el-container class="layout">
      <el-aside v-if="layout === 'sidebar'" width="200px" class="aside">
        <div class="brand">
          <!-- Click the title to open the server overview (OpsDialog) -->
          <button type="button" class="brand-btn" @click="opsVisible = true">
            <span class="brand-mark">
              <el-icon :size="16"><Collection /></el-icon>
            </span>
            <span>{{ title }}</span>
          </button>
        </div>
        <el-menu :default-active="active" class="menu" @select="active = $event">
          <el-menu-item index="memories">
            <el-icon><Notebook /></el-icon>
            <span>{{ t('nav.memories') }}</span>
          </el-menu-item>
          <el-menu-item index="tags">
            <el-icon><PriceTag /></el-icon>
            <span>{{ t('nav.tags') }}</span>
          </el-menu-item>
        </el-menu>
        <div class="aside-footer">
          <slot name="footer" />
        </div>
      </el-aside>

      <el-main class="main">
        <el-tabs v-if="layout === 'tabs'" v-model="active" class="tabs-bar">
          <el-tab-pane :label="t('nav.memories')" name="memories" />
          <el-tab-pane :label="t('nav.tags')" name="tags" />
        </el-tabs>

        <!-- Panels stay mounted: switching tabs keeps state and avoids refetching -->
        <MemoriesPanel v-show="active === 'memories'" />
        <TagsPanel v-show="active === 'tags'" />
      </el-main>
    </el-container>
    <OpsDialog :visible="opsVisible" @update:visible="opsVisible = $event" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { t } from '../i18n'
import { Collection, Notebook, PriceTag } from '@element-plus/icons-vue'
import MemoriesPanel from './MemoriesPanel.vue'
import TagsPanel from './TagsPanel.vue'
import OpsDialog from './OpsDialog.vue'

/**
 * Root admin console component. layout:
 * - 'sidebar': fixed left-side navigation, suited to a standalone full-screen site (default)
 * - 'tabs': top tab navigation, suited to embedding in host system pages
 */
withDefaults(defineProps<{ layout?: 'sidebar' | 'tabs'; title?: string }>(), {
  layout: 'sidebar',
  title: 'Agent Memory',
})

type AdminTab = 'memories' | 'tags'
const active = ref<AdminTab>('memories')
// Service overview dialog: in sidebar layout it opens by clicking the sidebar title; in tabs
// layout there is no title area, so the host mounts OpsDialog itself
const opsVisible = ref(false)
</script>

<style scoped>
/* Fill the viewport on the standalone site; when embedded, override with --memory-admin-height (e.g. 480px / auto) */
.memory-ui {
  height: var(--memory-admin-height, 100%);
}
.layout {
  height: 100%;
}
.aside {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--el-border-color-lighter);
  background: var(--el-bg-color);
}
.brand {
  display: flex;
  align-items: center;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: -0.01em;
  padding: 20px 20px 14px;
  color: var(--el-text-color-primary);
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
  color: inherit;
  font: inherit;
  letter-spacing: inherit;
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
.menu {
  border-right: none;
  flex: 1;
  padding: 4px 12px;
  background: transparent;
}
/* Nav items: pill-shaped, surface color on hover, soft brand-green fill when active */
.menu :deep(.el-menu-item) {
  height: 40px;
  line-height: 40px;
  margin: 2px 0;
  border-radius: var(--el-border-radius-base);
  color: var(--el-text-color-regular);
}
.menu :deep(.el-menu-item:hover) {
  background: var(--el-fill-color-light);
}
.menu :deep(.el-menu-item.is-active) {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  font-weight: 600;
}
.aside-footer {
  padding: 14px 20px;
}
.tabs-bar {
  margin-bottom: 4px;
}
.main {
  background: var(--el-bg-color-page);
  padding: 24px 28px;
}
</style>
