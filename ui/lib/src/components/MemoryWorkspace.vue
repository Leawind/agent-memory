<template>
  <div ref="rootRef" class="am-workspace" :class="{ 'is-stacked': narrow, 'is-dragging': dragging }">
    <aside class="amws-aside" :style="narrow ? undefined : { width: `${sidebarWidth}px` }">
      <TagsSidebar
        ref="sidebar"
        :active="panelExpr"
        @select="toggleFilter"
        @edit="openTag"
        @create="createTag"
        @predicates="predicateVisible = true"
        @clear="clearFilter"
      />
    </aside>

    <!-- Drag handle between sidebar and list: adjusts the sidebar width within bounds.
         Pointer capture keeps the drag alive even when the cursor leaves the 6px strip. -->
    <div
      v-show="!narrow"
      class="amws-resizer"
      role="separator"
      aria-orientation="vertical"
      :aria-valuenow="sidebarWidth"
      @pointerdown="onDragStart"
      @pointermove="onDragMove"
      @pointerup="onDragEnd"
      @pointercancel="onDragEnd"
    />

    <div class="amws-main">
      <MemoriesPanel ref="panel" @changed="refreshSidebar" />
    </div>

    <!-- Tag dialog: opened from a sidebar entry's edit affordance (the row's plain click
         filters the list instead) or the sidebar's plus button (create); every change
         renames/deletes tags across memories, so both sides refresh afterwards -->
    <TagDialog
      :visible="tagDialogVisible"
      :tag="editingTag"
      @update:visible="tagDialogVisible = $event"
      @saved="onTagChanged"
      @deleted="onTagChanged"
    />
    <PredicateDialog
      v-model:visible="predicateVisible"
      @changed="onTagChanged"
      @apply="panel?.toggleTagFilter($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { TagView } from '../types'
import MemoriesPanel from './MemoriesPanel.vue'
import TagsSidebar from './TagsSidebar.vue'
import TagDialog from './TagDialog.vue'
import PredicateDialog from './PredicateDialog.vue'
const predicateVisible = ref(false)

const rootRef = ref<HTMLElement | null>(null)

/** The main working area: tag sidebar on the left, memory list on the right, resizable
 * between them. Coordinates refreshes: tag edits reshape memory rows (and vice versa). */
const { narrow } = useContainerWidth(rootRef)

// ---- Sidebar width: draggable, persisted per browser ----
const WIDTH_KEY = 'agent-memory-sidebar-width'
const WIDTH_MIN = 180
const WIDTH_MAX = 420
const WIDTH_DEFAULT = 260

function readStoredWidth(): number {
  try {
    const raw = localStorage.getItem(WIDTH_KEY)
    // getItem returns null when unset and Number(null) is 0 — only trust actually stored values
    if (raw !== null) {
      const v = Number(raw)
      if (Number.isFinite(v) && v > 0) return Math.min(WIDTH_MAX, Math.max(WIDTH_MIN, v))
    }
  } catch {
    /* Storage unavailable (e.g. private mode): fall through to the default */
  }
  return WIDTH_DEFAULT
}

const sidebarWidth = ref(readStoredWidth())
const dragging = ref(false)

let dragStartX = 0
let dragStartWidth = 0

function onDragStart(e: PointerEvent): void {
  dragging.value = true
  dragStartX = e.clientX
  dragStartWidth = sidebarWidth.value
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}

function onDragMove(e: PointerEvent): void {
  if (!dragging.value) return
  sidebarWidth.value = Math.min(WIDTH_MAX, Math.max(WIDTH_MIN, dragStartWidth + e.clientX - dragStartX))
}

function onDragEnd(e: PointerEvent): void {
  if (!dragging.value) return
  dragging.value = false
  ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
  try {
    localStorage.setItem(WIDTH_KEY, String(sidebarWidth.value))
  } catch {
    /* Storage unavailable: the width just resets on reload */
  }
}

// ---- Tag dialog state ----
const tagDialogVisible = ref(false)
const editingTag = ref<TagView | null>(null)

function openTag(tag: TagView): void {
  editingTag.value = tag
  tagDialogVisible.value = true
}

function createTag(): void {
  editingTag.value = null
  tagDialogVisible.value = true
}

// ---- Sidebar click = filter: clicking a tag narrows the list to it (clicking the active
// one clears); the panel owns the expression, the sidebar only mirrors it for highlighting ----
function toggleFilter(tag: TagView): void {
  panel.value?.toggleTagFilter(tag.name)
}

// Sidebar "all memories" row: drop the tag expression, keep any keyword search
function clearFilter(): void {
  panel.value?.clearTagFilter()
}

const panelExpr = computed(() => panel.value?.tagExpr ?? '')

// ---- Cross-side refresh: panel and sidebar stay in sync through each other's changes ----
const sidebar = ref<InstanceType<typeof TagsSidebar> | null>(null)
const panel = ref<InstanceType<typeof MemoriesPanel> | null>(null)

function refreshSidebar(): void {
  sidebar.value?.refresh()
}

function onTagChanged(): void {
  editingTag.value = null
  refreshSidebar()
  panel.value?.refresh()
}

// A permanently mounted workspace cannot sense visibility itself: the host calls refresh
// when the identity changes to pull the latest data on both sides
defineExpose({
  refresh: () => {
    predicateVisible.value = false
    refreshSidebar()
    panel.value?.refresh()
  },
})
</script>

<style scoped>
.am-workspace {
  display: flex;
  align-items: stretch;
  height: 100%;
  min-height: 0;
}
/* While dragging: keep the grab even across text, and show the resize cursor everywhere */
.am-workspace.is-dragging {
  cursor: col-resize;
  user-select: none;
}

.amws-aside {
  flex-shrink: 0;
  min-height: 0;
}

/* The drag strip: a hairline that thickens into a handle on hover. The always-on line gives the
   sidebar a visible edge without the weight of a real border. */
.amws-resizer {
  flex-shrink: 0;
  width: 8px;
  margin: 0 3px;
  border-radius: 999px;
  cursor: col-resize;
  touch-action: none;
  position: relative;
}
.amws-resizer::after {
  content: '';
  position: absolute;
  inset: 12px 3px;
  border-radius: 999px;
  background: var(--el-border-color-light);
  transition: background 0.15s ease;
}
.amws-resizer:hover::after,
.is-dragging .amws-resizer::after {
  background: var(--el-border-color);
}

.amws-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
}
.amws-main :deep(.am-panel) {
  flex: 1;
  min-height: 0;
}

/* Narrow containers: the sidebar turns into a compact horizontal filter strip above the list.
   A full vertical tag column would eat most of a phone screen before the memories start. */
.am-workspace.is-stacked {
  flex-direction: column;
  gap: 10px;
}
.is-stacked .amws-aside {
  flex: 0 0 auto;
  width: 100%;
  max-height: none;
  overflow: visible;
}
.is-stacked .amws-main {
  min-height: 0;
}
.is-stacked :deep(.am-tags-sidebar) {
  height: auto;
  gap: 6px;
}
.is-stacked :deep(.sb-head) {
  padding: 0;
}
.is-stacked :deep(.sb-list) {
  flex-direction: row;
  flex-wrap: nowrap;
  gap: 6px;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 2px 2px 6px;
  scrollbar-width: thin;
}
.is-stacked :deep(.sb-all),
.is-stacked :deep(.sb-item) {
  flex: 0 0 auto;
  flex-direction: row;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 999px;
  background: var(--el-fill-color);
}
.is-stacked :deep(.sb-all.is-active),
.is-stacked :deep(.sb-item.is-active) {
  background: var(--el-color-primary-light-9);
}
.is-stacked :deep(.sb-item-top) {
  gap: 6px;
}
.is-stacked :deep(.sb-name),
.is-stacked :deep(.sb-all-name) {
  max-width: 160px;
}
.is-stacked :deep(.sb-desc) {
  display: none;
}
/* No hover on touch: keep the edit affordance visible in the strip */
.is-stacked :deep(.sb-edit) {
  opacity: 0.65;
}
</style>
