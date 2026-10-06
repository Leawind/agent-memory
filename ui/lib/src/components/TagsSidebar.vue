<template>
  <div class="am-tags-sidebar">
    <!-- Header: label + count + the create entry (the tag dialog handles the actual form) -->
    <div class="sb-head">
      <span class="sb-title">{{ t('tags.title') }}</span>
      <span class="sb-count">{{ rows.length }}</span>
      <el-tooltip :content="t('tags.create')" placement="top" :enterable="false">
        <el-button
          class="sb-add"
          :icon="Plus"
          circle
          size="small"
          :aria-label="t('tags.create')"
          @click="emit('create')"
        />
      </el-tooltip>
    </div>

    <div v-loading="loading" class="sb-list">
      <p v-if="!loading && rows.length === 0" class="sb-none">{{ t('tags.none') }}</p>
      <button
        v-for="tag in sorted"
        :key="tag.name"
        type="button"
        class="sb-item"
        :class="{ 'is-active': isActive(tag) }"
        @click="emit('select', tag)"
      >
        <span class="sb-item-top">
          <span class="sb-name">{{ tag.name }}</span>
          <el-tooltip v-if="tag.reserved" :content="t('tags.reservedHint')" placement="top" :enterable="false">
            <el-tag size="small" type="warning" class="sb-reserved">{{ t('tags.reserved') }}</el-tag>
          </el-tooltip>
          <span class="sb-item-count" :title="t('tags.title')">{{ tag.count }}</span>
          <!-- Edit lives on a hover-revealed affordance: the row's own click is the filter
               toggle, so renaming/deleting gets a smaller, separate target (span + role so a
               real button is never nested inside this row button) -->
          <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
            <span
              class="sb-edit"
              role="button"
              :aria-label="t('common.edit')"
              @click.stop="emit('edit', tag)"
              @keydown.enter.stop.prevent="emit('edit', tag)"
            >
              <el-icon><Edit /></el-icon>
            </span>
          </el-tooltip>
        </span>
        <span v-if="tag.description" class="sb-desc">{{ tag.description }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Edit, Plus } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { toastError } from '../toast'
import { useTags } from '../composables/useTags'
import type { TagView } from '../types'

/** Sidebar listing every tag vertically (Modrinth discover-style): name + description +
 * memory count. Clicking an entry asks the parent to filter the list by that tag; editing
 * (rename/description/delete) goes through the hover-revealed edit affordance. The parent
 * feeds the current expression back as `active` to highlight the filtered row. */
const props = defineProps<{
  /** The panel's current tag expression; a row is active when the expression is exactly its name */
  active?: string
}>()

const emit = defineEmits<{
  select: [tag: TagView]
  edit: [tag: TagView]
  create: []
}>()

const { rows, loading, reload } = useTags()

// Most-used first, ties broken by name — a stable order independent of server-side ordering
const sorted = computed(() => [...rows.value].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name)))

function isActive(tag: TagView): boolean {
  return (props.active ?? '').trim() === tag.name
}

defineExpose({
  refresh: () => reload().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e))),
})
</script>

<style scoped>
.am-tags-sidebar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
}
.sb-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 4px;
}
.sb-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  letter-spacing: 0.02em;
}
.sb-count {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}
.sb-add {
  margin-left: auto;
}

.sb-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
  /* Room for the loading mask / focus rings */
  padding: 2px;
}
.sb-none {
  margin: 8px 4px;
  font-size: 13px;
  color: var(--el-text-color-placeholder);
}

/* Tag entry: name + reserved badge + count on the first line, description below.
   A plain button styled as a quiet list row (Modrinth filter-entry look). */
.sb-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 7px 10px;
  border: none;
  border-radius: var(--el-border-radius-base);
  background: transparent;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.sb-item:hover {
  background: var(--el-fill-color);
}
/* The row whose tag is the active expression filter */
.sb-item.is-active {
  background: var(--el-color-primary-light-9);
}
.sb-item.is-active .sb-name {
  color: var(--el-color-primary);
}
.sb-item-top {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}
.sb-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.sb-reserved {
  flex-shrink: 0;
}
.sb-item-count {
  margin-left: auto;
  flex-shrink: 0;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--el-text-color-secondary);
}
/* Hover-revealed edit affordance (opens the tag dialog); keeps its slot so the count
   never shifts when it fades in */
.sb-edit {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  font-size: 13px;
  color: var(--el-text-color-placeholder);
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.sb-item:hover .sb-edit,
.sb-item:focus-visible .sb-edit,
.sb-edit:focus {
  opacity: 1;
}
.sb-edit:hover {
  color: var(--el-text-color-regular);
}
.sb-desc {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  font-size: 12px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  word-break: break-word;
}
</style>
