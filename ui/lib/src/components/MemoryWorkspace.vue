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

/* The drag strip: nearly invisible until hovered, then a rounded handle line */
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
  background: transparent;
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

/* Narrow containers: stack — the sidebar becomes a capped block above the list */
.am-workspace.is-stacked {
  flex-direction: column;
  gap: 12px;
}
.is-stacked .amws-aside {
  flex: 0 0 auto;
  max-height: 240px;
  overflow-y: auto;
}
.is-stacked .amws-main {
  min-height: 0;
}
</style>
