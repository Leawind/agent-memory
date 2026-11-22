<template>
  <div class="am-tags-sidebar">
    <!-- Header: label + count + quiet icon tools (tag predicates, scopes, create tag). -->
    <div class="sb-head">
      <span class="sb-title">{{ t('tags.title') }}</span>
      <span class="sb-count">{{ rows.length }}</span>
      <div class="sb-tools">
        <el-tooltip :content="t('predicates.title')" placement="bottom" :enterable="false">
          <button type="button" class="sb-tool" :aria-label="t('predicates.title')" @click="emit('predicates')">
            <el-icon :size="14"><Operation /></el-icon>
          </button>
        </el-tooltip>
        <el-tooltip :content="t('scopes.create')" placement="bottom" :enterable="false">
          <button type="button" class="sb-tool" :aria-label="t('scopes.create')" @click="emit('createScope')">
            <el-icon :size="14"><FolderAdd /></el-icon>
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
      <!-- Bare tags first: their own name is the filter expression -->
      <button
        v-for="tag in bareTags"
        :key="tag.name"
        type="button"
        class="sb-item"
        :class="{ 'is-active': isActive(tag.name) }"
        @click="emit('select', tag.name)"
      >
        <span class="sb-item-top">
          <span class="sb-twist sb-twist-empty" aria-hidden="true"></span>
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
      <!-- Then one section per scope: the header filters by the scope wildcard, its values
           (prefix stripped) nest below, collapsed by default -->
      <template v-for="group in scopeGroups" :key="group.scope.name">
        <button
          type="button"
          class="sb-item sb-scope"
          :class="{ 'is-active': isActive(group.expr) }"
          @click="emit('select', group.expr)"
        >
          <span class="sb-item-top">
            <span
              v-if="group.values.length > 0"
              class="sb-twist"
              :class="{ 'is-open': isExpanded(group.scope.name) }"
              role="button"
              :aria-label="t('tags.toggleValues')"
              :aria-expanded="isExpanded(group.scope.name)"
              tabindex="0"
              @click.stop="toggleExpanded(group.scope.name)"
              @keydown.enter.stop.prevent="toggleExpanded(group.scope.name)"
              @keydown.space.stop.prevent="toggleExpanded(group.scope.name)"
            >
              <el-icon :size="12"><CaretRight /></el-icon>
            </span>
            <span v-else class="sb-twist sb-twist-empty" aria-hidden="true"></span>
            <span class="sb-name">{{ group.scope.name }}</span>
            <el-tooltip
              v-if="group.scope.required"
              :content="t('scopes.requiredHint')"
              placement="top"
              :enterable="false"
            >
              <el-tag size="small" type="info" class="sb-required">{{ t('scopes.required') }}</el-tag>
            </el-tooltip>
            <span class="sb-item-count" :title="t('tags.title')">{{ group.scope.count }}</span>
            <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
              <span
                class="sb-edit"
                role="button"
                :aria-label="t('common.edit')"
                @click.stop="emit('editScope', group.scope)"
                @keydown.enter.stop.prevent="emit('editScope', group.scope)"
              >
                <el-icon><Edit /></el-icon>
              </span>
            </el-tooltip>
          </span>
          <span v-if="group.scope.description" class="sb-desc">{{ group.scope.description }}</span>
        </button>
        <!-- Value rows: the scope's tags (`scope::value`) nested under their scope, shown only
             while it is expanded. The prefix is dropped (the nesting already says it); the
             title keeps the full name on hover. Clicking one filters by the tag itself; the
             edit affordance opens the tag dialog on the full name. -->
        <template v-if="group.values.length > 0 && isExpanded(group.scope.name)">
          <button
            v-for="value in group.values"
            :key="value.name"
            type="button"
            class="sb-item sb-value"
            :class="{ 'is-active': isActive(value.name) }"
            :title="value.name"
            @click="emit('select', value.name)"
          >
            <span class="sb-item-top">
              <span class="sb-name">{{ valueLabel(group.scope.name, value.name) }}</span>
              <span class="sb-item-count" :title="t('tags.title')">{{ value.count }}</span>
              <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
                <span
                  class="sb-edit"
                  role="button"
                  :aria-label="t('common.edit')"
                  @click.stop="emit('edit', value)"
                  @keydown.enter.stop.prevent="emit('edit', value)"
                >
                  <el-icon><Edit /></el-icon>
                </span>
              </el-tooltip>
            </span>
          </button>
        </template>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { CaretRight, Collection, Edit, FolderAdd, Operation, Plus } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { toastError } from '../toast'
import { useTags } from '../composables/useTags'
import type { ScopeView, TagView } from '../types'

/** One scope section: the scope plus the tags living under it (`scope::value` full names). */
interface ScopeGroup {
  scope: ScopeView
  /** The scope wildcard atom, the expression its header filters by */
  expr: string
  values: TagView[]
}

/** Sidebar listing the whole vocabulary vertically (Modrinth discover-style): bare tags as their
 *  own entries, then one section per scope — the header filters by the `scope::*` wildcard and
 *  the scope's values nest below it, collapsed by default, prefix stripped. Clicking an entry
 *  asks the parent to filter the list by it; editing (rename/description/delete) goes through
 *  the hover-revealed edit affordance (a scope's opens the scope dialog, a tag's the tag
 *  dialog). The parent feeds the current expression back as `active` to highlight the filtered
 *  row. */
const props = defineProps<{
  /** The panel's current tag expression; a row is active when the expression matches exactly */
  active?: string
}>()

const emit = defineEmits<{
  /** select carries the exact filter expression: a tag's own name, or a scope's `scope::*` */
  select: [expr: string]
  edit: [tag: TagView]
  editScope: [scope: ScopeView]
  create: []
  createScope: []
  predicates: []
  /** The "all memories" row: clear the tag expression */
  clear: []
}>()

const { rows, scopes, loading, reload } = useTags()

// Most-used first, ties broken by name — a stable order independent of server-side ordering
const byUse = (a: TagView, b: TagView) => b.count - a.count || a.name.localeCompare(b.name)

const bareTags = computed(() => rows.value.filter((tag) => !tag.scope).sort(byUse))

const scopeGroups = computed<ScopeGroup[]>(() =>
  scopes.value
    .map((scope) => ({ scope, byCount: scope.count, byName: scope.name }))
    .sort((a, b) => b.byCount - a.byCount || a.byName.localeCompare(b.byName))
    .map(({ scope }) => ({
      scope,
      expr: `${scope.name}::*`,
      values: rows.value.filter((tag) => tag.scope === scope.name).sort(byUse),
    })),
)

/** The nesting already carries the scope: the value row shows the bare value half */
function valueLabel(scope: string, name: string): string {
  return name.startsWith(`${scope}::`) ? name.slice(scope.length + 2) : name
}

// Value lists start collapsed: the sidebar stays scannable, values are a drill-down
const expandedScopes = ref<Set<string>>(new Set())

function isExpanded(scope: string): boolean {
  return expandedScopes.value.has(scope)
}

function toggleExpanded(scope: string): void {
  const next = new Set(expandedScopes.value)
  if (!next.delete(scope)) next.add(scope)
  expandedScopes.value = next
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

/* Tag entry: twist slot + name + badges + count on the first line, description below. A plain
   button styled as a quiet list row (Modrinth filter-entry look). */
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
/* Expand toggle (scopes with values) / its empty placeholder (the rest): fixed width so the
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
.sb-reserved,
.sb-required {
  flex-shrink: 0;
}
.sb-item-count {
  margin-left: auto;
  flex-shrink: 0;
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--el-text-color-secondary);
}
/* Hover-revealed edit affordance (opens the tag/scope dialog); keeps its slot so the count
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

/* Value row: the NAME nests one full indent step (twist slot + gap) deeper than its scope's
   name — the scope name sits at 10 + 14 + 6 = 30px, so 30px here would align values with
   scopes. The count stays right-aligned (margin-left auto) and does not move. */
.sb-value {
  padding-left: 50px;
}
.sb-value .sb-name {
  font-weight: 400;
  color: var(--el-text-color-regular);
}
</style>
