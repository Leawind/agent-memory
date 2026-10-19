<template>
  <div class="am-tags-sidebar">
    <!-- Header: label + count + quiet icon tools (tag predicates, create). The advanced
         predicate editor used to be a full-width button that read like a page tab. -->
    <div class="sb-head">
      <span class="sb-title">{{ t('tags.title') }}</span>
      <span class="sb-count">{{ rows.length }}</span>
      <div class="sb-tools">
        <el-tooltip :content="t('predicates.title')" placement="bottom" :enterable="false">
          <button type="button" class="sb-tool" :aria-label="t('predicates.title')" @click="emit('predicates')">
            <el-icon :size="14"><Operation /></el-icon>
          </button>
        </el-tooltip>
        <el-tooltip :content="t('tags.create')" placement="bottom" :enterable="false">
          <button type="button" class="sb-tool sb-add" :aria-label="t('tags.create')" @click="emit('create')">
            <el-icon :size="14"><Plus /></el-icon>
          </button>
        </el-tooltip>
      </div>
    </div>
    <div v-loading="loading" class="sb-list">
      <!-- Always-visible reset: clears the tag expression and returns to the full list -->
      <button type="button" class="sb-all" :class="{ 'is-active': !isFiltering }" @click="emit('clear')">
        <el-icon class="sb-all-icon" :size="14"><Collection /></el-icon>
        <span class="sb-all-name">{{ t('tags.all') }}</span>
      </button>
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
          <!-- Keyed tags (name carries ':') are attribute values: the server owns their lifecycle
               through the key registry, so the rename/delete affordance is hidden (the row click
               still filters). Edit lives on a hover-revealed affordance for the rest: the row's
               own click is the filter toggle, so renaming/deleting gets a smaller, separate
               target (span + role so a real button is never nested inside this row button) -->
          <el-tooltip v-if="!tag.name.includes(':')" :content="t('common.edit')" placement="top" :enterable="false">
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
import { Collection, Edit, Operation, Plus } from '@element-plus/icons-vue'
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
  predicates: []
  /** The "all memories" row: clear the tag expression */
  clear: []
}>()

const { rows, loading, reload } = useTags()

// Most-used first, ties broken by name — a stable order independent of server-side ordering
const sorted = computed(() => [...rows.value].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name)))

const isFiltering = computed(() => (props.active ?? '').trim() !== '')

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
  gap: 10px;
  height: 100%;
  min-height: 0;
}
.sb-head {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 2px;
  min-height: 28px;
}
.sb-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-secondary);
  letter-spacing: 0.02em;
}
.sb-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 999px;
  background: var(--el-fill-color);
  color: var(--el-text-color-secondary);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}
.sb-tools {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: auto;
}
.sb-tool {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--el-text-color-secondary);
  cursor: pointer;
}
.sb-tool:hover {
  background: var(--el-fill-color);
  color: var(--el-text-color-primary);
}
.sb-tool:focus-visible,
.sb-all:focus-visible,
.sb-item:focus-visible {
  outline: 2px solid var(--el-color-primary-light-5);
  outline-offset: 1px;
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

/* Reset row: the whole list, highlighted whenever no tag expression is applied */
.sb-all {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--el-text-color-regular);
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.sb-all:hover {
  background: var(--el-fill-color);
}
.sb-all.is-active {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}
.sb-all-icon {
  flex-shrink: 0;
  color: var(--el-text-color-placeholder);
}
.sb-all.is-active .sb-all-icon {
  color: var(--el-color-primary);
}
.sb-all-name {
  font-weight: 500;
}

/* Tag entry: name + reserved badge + count on the first line, description below.
   A plain button styled as a quiet list row (Modrinth filter-entry look). */
.sb-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 7px 10px;
  border: none;
  border-radius: 8px;
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
  justify-content: center;
  width: 14px;
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
