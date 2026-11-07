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
      <template v-for="group in groups" :key="group.tag.name">
        <button
          type="button"
          class="sb-item"
          :class="{ 'is-active': isActive(group.expr) }"
          @click="emit('select', group.tag, group.expr)"
        >
          <span class="sb-item-top">
            <!-- Expand toggle for keys with values: a span + role (a real button can never nest
                 inside this row button); the twist slot keeps every row's name aligned -->
            <span
              v-if="group.values.length > 0"
              class="sb-twist"
              :class="{ 'is-open': isExpanded(group.tag.name) }"
              role="button"
              :aria-label="t('tags.toggleValues')"
              :aria-expanded="isExpanded(group.tag.name)"
              tabindex="0"
              @click.stop="toggleExpanded(group.tag.name)"
              @keydown.enter.stop.prevent="toggleExpanded(group.tag.name)"
              @keydown.space.stop.prevent="toggleExpanded(group.tag.name)"
            >
              <el-icon :size="12"><CaretRight /></el-icon>
            </span>
            <span v-else class="sb-twist sb-twist-empty" aria-hidden="true"></span>
            <span class="sb-name">{{ group.tag.name }}</span>
            <el-tooltip v-if="group.tag.reserved" :content="t('tags.reservedHint')" placement="top" :enterable="false">
              <el-tag size="small" type="warning" class="sb-reserved">{{ t('tags.reserved') }}</el-tag>
            </el-tooltip>
            <span class="sb-item-count" :title="t('tags.title')">{{ group.tag.count }}</span>
            <!-- Edit lives on a hover-revealed affordance: the row's own click is the filter
                 toggle, so renaming/deleting gets a smaller, separate target (span + role so a
                 real button is never nested inside this row button) -->
            <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
              <span
                class="sb-edit"
                role="button"
                :aria-label="t('common.edit')"
                @click.stop="emit('edit', group.tag)"
                @keydown.enter.stop.prevent="emit('edit', group.tag)"
              >
                <el-icon><Edit /></el-icon>
              </span>
            </el-tooltip>
          </span>
          <span v-if="group.tag.description" class="sb-desc">{{ group.tag.description }}</span>
        </button>
        <!-- Value rows: the attribute values (`key:value` edges) nested under their key,
             shown only while the key is expanded. The key prefix is dropped (the nesting
             already says it); the title keeps the full form on hover. Clicking one filters
             by the edge itself (`key=value`); their lifecycle belongs to the edges, so no
             edit affordance. -->
        <template v-if="group.values.length > 0 && isExpanded(group.tag.name)">
          <button
            v-for="value in group.values"
            :key="value.expr"
            type="button"
            class="sb-item sb-value"
            :class="{ 'is-active': isActive(value.expr) }"
            :title="`${group.tag.name}:${value.tag.name}`"
            @click="emit('select', value.tag, value.expr)"
          >
            <span class="sb-item-top">
              <span class="sb-name">{{ value.tag.name }}</span>
              <span class="sb-item-count" :title="t('tags.title')">{{ value.tag.count }}</span>
            </span>
          </button>
        </template>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { CaretRight, Collection, Edit, Operation, Plus } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { toastError } from '../toast'
import { useTags } from '../composables/useTags'
import type { TagView } from '../types'

/** One attribute value nested under its key: the value tag plus the `key=value` expression
 *  that filters by the displayed `key:value` edge (a bare name cannot carry the key). */
interface ValueRow {
  tag: TagView
  expr: string
}

/** A sidebar entry: the tag itself, the expression its click filters by, and the values
 *  nested under it when edges point from it at other tags. */
interface GroupRow {
  tag: TagView
  expr: string
  values: ValueRow[]
}

/** Sidebar listing every tag vertically (Modrinth discover-style): name + description +
 * memory count. Clicking an entry asks the parent to filter the list by it; editing
 * (rename/description/delete) goes through the hover-revealed edit affordance. The parent
 * feeds the current expression back as `active` to highlight the filtered row. Tags that
 * other tags point at (attribute values) are not their own entries: they nest, collapsed
 * by default, under each key that points at them (TagView.keys). */
const props = defineProps<{
  /** The panel's current tag expression; a row is active when the expression matches exactly */
  active?: string
}>()

const emit = defineEmits<{
  /** select carries the tag and the exact filter expression (a value's is `key=value`) */
  select: [tag: TagView, expr: string]
  edit: [tag: TagView]
  create: []
  predicates: []
  /** The "all memories" row: clear the tag expression */
  clear: []
}>()

const { rows, loading, reload } = useTags()

// Most-used first, ties broken by name — a stable order independent of server-side ordering
const sorted = computed(() => [...rows.value].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name)))

// The tree: top-level entries are the tags with a presence of their own (bare or edge
// source); a value-only tag lives inside its keys' value lists. A tag both pointed at and
// organizing appears top-level and as a value.
const groups = computed<GroupRow[]>(() =>
  sorted.value
    .filter((tag) => !tag.target_only)
    .map((tag) => ({
      tag,
      expr: tag.name,
      values: sorted.value
        .filter((v) => v.keys?.includes(tag.name))
        .map((v) => ({ tag: v, expr: `${tag.name}=${v.name}` })),
    })),
)

// Value lists start collapsed: the sidebar stays scannable, values are a drill-down
const expandedKeys = ref<Set<string>>(new Set())

function isExpanded(key: string): boolean {
  return expandedKeys.value.has(key)
}

function toggleExpanded(key: string): void {
  const next = new Set(expandedKeys.value)
  if (!next.delete(key)) next.add(key)
  expandedKeys.value = next
}

const isFiltering = computed(() => (props.active ?? '').trim() !== '')

function isActive(expr: string): boolean {
  return (props.active ?? '').trim() === expr
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
.sb-twist:focus-visible,
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

/* Tag entry: twist slot + name + reserved badge + count on the first line, description
   below. A plain button styled as a quiet list row (Modrinth filter-entry look). */
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
/* The row whose expression is the active filter */
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
/* Expand toggle (keys with values) / its empty placeholder (the rest): fixed width so the
   names line up across rows; the caret turns down while expanded */
.sb-twist {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  border-radius: 4px;
  color: var(--el-text-color-placeholder);
  cursor: pointer;
  transition: transform 0.15s ease;
}
.sb-twist:hover {
  color: var(--el-text-color-regular);
}
.sb-twist.is-open {
  transform: rotate(90deg);
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

/* Value row: the NAME nests one full indent step (twist slot + gap) deeper than its key's
   name — the key name sits at 10 + 14 + 6 = 30px, so 30px here would align values with keys.
   The count stays right-aligned (margin-left auto) and does not move. */
.sb-value {
  padding-left: 50px;
}
.sb-value .sb-name {
  font-weight: 400;
  color: var(--el-text-color-regular);
}
</style>
