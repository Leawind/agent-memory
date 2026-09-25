<template>
  <el-container class="layout">
    <el-aside width="200px" class="aside">
      <div class="brand">
        <el-icon size="22"><Collection /></el-icon>
        <span>agent-memory</span>
      </div>
      <el-menu :default-active="active" class="menu" @select="active = $event">
        <el-menu-item index="memories">
          <el-icon><Notebook /></el-icon>
          <span>记忆管理</span>
        </el-menu-item>
        <el-menu-item index="tags">
          <el-icon><PriceTag /></el-icon>
          <span>标签管理</span>
        </el-menu-item>
        <el-menu-item index="ops">
          <el-icon><Odometer /></el-icon>
          <span>运维</span>
        </el-menu-item>
      </el-menu>
      <div class="aside-footer">
        <el-text size="small" type="info">自托管 · 多 agent 共享</el-text>
      </div>
    </el-aside>
    <el-main class="main">
      <MemoriesView v-show="active === 'memories'" />
      <TagsView v-show="active === 'tags'" />
      <OpsView v-show="active === 'ops'" />
    </el-main>
  </el-container>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import MemoriesView from './views/MemoriesView.vue'
import TagsView from './views/TagsView.vue'
import OpsView from './views/OpsView.vue'

const active = ref<'memories' | 'tags' | 'ops'>('memories')
</script>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
}
.layout {
  height: 100%;
}
.aside {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--el-border-color-light);
  background: var(--el-bg-color);
}
.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 16px;
  padding: 18px 20px;
  color: var(--el-color-primary);
}
.menu {
  border-right: none;
  flex: 1;
}
.aside-footer {
  padding: 14px 20px;
}
.main {
  background: var(--el-fill-color-lighter);
}
</style>
