<template>
  <div class="memory-ui">
    <el-container class="layout">
      <el-aside v-if="layout === 'sidebar'" width="200px" class="aside">
        <div class="brand">
          <span class="brand-mark">
            <el-icon :size="16"><Collection /></el-icon>
          </span>
          <span>{{ title }}</span>
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
          <el-menu-item index="ops">
            <el-icon><Odometer /></el-icon>
            <span>{{ t('nav.ops') }}</span>
          </el-menu-item>
        </el-menu>
        <div class="aside-footer">
          <slot name="footer">
            <el-text size="small" type="info">{{ t('admin.hint') }}</el-text>
          </slot>
        </div>
      </el-aside>

      <el-main class="main">
        <el-tabs v-if="layout === 'tabs'" v-model="active" class="tabs-bar">
          <el-tab-pane :label="t('nav.memories')" name="memories" />
          <el-tab-pane :label="t('nav.tags')" name="tags" />
          <el-tab-pane :label="t('nav.ops')" name="ops" />
        </el-tabs>

        <!-- 面板常驻挂载：切换导航不销毁、不重新请求 -->
        <MemoriesPanel v-show="active === 'memories'" />
        <TagsPanel v-show="active === 'tags'" />
        <OpsPanel v-show="active === 'ops'" />
      </el-main>
    </el-container>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { t } from '../i18n'
import { Collection, Notebook, Odometer, PriceTag } from '@element-plus/icons-vue'
import MemoriesPanel from './MemoriesPanel.vue'
import TagsPanel from './TagsPanel.vue'
import OpsPanel from './OpsPanel.vue'

/**
 * 管理台根组件。layout:
 * - 'sidebar'：左侧固定导航，适合独立全屏站点（默认）
 * - 'tabs'：顶部页签导航，适合嵌入宿主系统页面
 */
withDefaults(defineProps<{ layout?: 'sidebar' | 'tabs'; title?: string }>(), {
  layout: 'sidebar',
  title: 'agent-memory',
})

type AdminTab = 'memories' | 'tags' | 'ops'
const active = ref<AdminTab>('memories')
</script>

<style scoped>
/* 独立站撑满视口；嵌入宿主时可用 --memory-admin-height 覆盖（如 480px / auto） */
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
  gap: 10px;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: -0.01em;
  padding: 20px 20px 14px;
  color: var(--el-text-color-primary);
}
/* 品牌图标：品牌绿渐变圆角块（Modrinth 的 --brand-gradient-bg） */
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
/* 导航项：胶囊形，悬浮换表面色，选中铺品牌绿软底 */
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
