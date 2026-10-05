<template>
  <div ref="rootRef" class="am-panel">
    <el-config-provider :locale="elementPlusLocale">
      <div v-if="showHeader" class="am-panel-header">
        <div class="am-heading">
          <h2 class="am-panel-title">{{ props.title ?? t('memories.title') }}</h2>
          <el-tooltip :content="props.subtitle ?? t('memories.subtitle')" placement="top">
            <el-icon class="am-info"><InfoFilled /></el-icon>
          </el-tooltip>
        </div>
        <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('memories.create') }}</el-button>
      </div>

      <div class="am-toolbar">
        <el-input
          v-model="query"
          :placeholder="t('memories.searchPlaceholder')"
          clearable
          class="search"
          @keyup.enter="run(onSearch)"
          @clear="run(onSearch)"
        >
          <template #append>
            <el-button :icon="Search" @click="run(onSearch)" />
          </template>
        </el-input>
        <el-select
          v-model="tagFilter"
          :placeholder="t('memories.tagFilter')"
          clearable
          filterable
          class="tag-filter"
          @change="run(onSearch)"
        >
          <el-option v-for="tag in tagOptions" :key="tag" :label="tag" :value="tag" />
        </el-select>
        <el-select v-model="sortChoice" class="sort-select">
          <el-option :label="t('memories.sortUpdatedDesc')" value="updated_at:desc" />
          <el-option :label="t('memories.sortUpdatedAsc')" value="updated_at:asc" />
          <el-option :label="t('memories.sortIdAsc')" value="id:asc" />
          <el-option :label="t('memories.sortIdDesc')" value="id:desc" />
        </el-select>
        <el-select v-model="mode" class="mode-select" @change="run(onSearch)">
          <el-option :label="t('memories.modeAuto')" value="auto" />
          <el-option :label="t('memories.modeKeyword')" value="keyword" />
          <el-option :label="t('memories.modeHybrid')" value="hybrid" />
        </el-select>
      </div>

      <el-alert v-if="note" :title="note" type="warning" show-icon :closable="false" />

      <!-- Card list (Modrinth discover-style): the summary is the entry's title, tags and the
           search snippet live on their own lines below it. The whole card opens the memory
           dialog; delete/edit live inside that dialog. -->
      <div v-loading="loading" class="am-list">
        <el-alert v-if="emptyNote" :title="emptyNote" type="info" show-icon :closable="false" />
        <button v-for="card in cards" :key="card.id" type="button" class="am-memory-card" @click="openEdit(card.id)">
          <span class="card-main">
            <span class="card-summary">{{ card.summary }}</span>
            <!-- Snippets are plain text (server sends them unescaped); text interpolation, never v-html -->
            <span v-if="card.snippet" class="card-snippet">{{ card.snippet }}</span>
            <span class="card-tags">
              <el-tag v-for="tag in card.tags" :key="tag" size="small">{{ tag }}</el-tag>
            </span>
          </span>
          <span class="card-meta">
            <span v-if="card.score !== undefined" class="card-score" :title="t('memories.colScore')">
              {{ card.score }}
            </span>
            <span class="card-time">{{ card.updated }}</span>
            <span class="card-id">{{ card.id }}</span>
          </span>
        </button>
      </div>

      <div class="am-pager" v-if="!searching">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next"
          @current-change="run(reload)"
          @size-change="run(reload)"
        />
      </div>
      <div class="am-pager" v-else>
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[10, 20, 50]"
          layout="total, sizes, prev, pager, next"
          @current-change="run(reload)"
          @size-change="run(reload)"
        />
      </div>

      <!-- Create / edit / delete (delete lives in the dialog's title row) -->
      <MemoryEditorDialog
        v-model:visible="editorVisible"
        :memory-id="editingId"
        :tag-options="tagOptions"
        :width="narrow ? '96%' : '960px'"
        @saved="onMutated"
        @deleted="onMutated"
      />
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { InfoFilled, Plus, Search } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { toastError } from '../toast'
import { useMemories } from '../composables/useMemories'
import { useContainerWidth } from '../composables/useContainerWidth'
import MemoryEditorDialog from './MemoryEditorDialog.vue'

const props = withDefaults(
  defineProps<{
    /** Hide the title/subtitle area (when embedded in a host that already has a page title, only the toolbar + list are wanted) */
    showHeader?: boolean
    /** Override the default title */
    title?: string
    /** Override the default subtitle */
    subtitle?: string
  }>(),
  { showHeader: true },
)

const emit = defineEmits<{
  /** A memory was created, updated or deleted: hosts (the workspace) refresh tag-derived views */
  changed: []
}>()

const {
  query,
  tagFilter,
  mode,
  sort,
  order,
  page,
  pageSize,
  rows,
  searchResults,
  note,
  total,
  loading,
  tagOptions,
  searching,
  onSearch,
  reload,
  loadTagOptions,
} = useMemories()

// Unified card shape across list rows and search hits (search adds snippet + score)
interface MemoryCard {
  id: string
  summary: string
  tags: string[]
  updated: string
  snippet?: string
  score?: number
}
const cards = computed<MemoryCard[]>(() =>
  searching.value
    ? searchResults.value.map((r) => ({
        id: r.id,
        summary: r.summary,
        tags: r.tags,
        updated: r.updated,
        snippet: r.snippet,
        score: r.score,
      }))
    : rows.value.map((r) => ({ id: r.id, summary: r.summary, tags: r.tags, updated: r.updated })),
)

// The empty-result notice is generated locally (the server's note targets agents and is English
// contract text, not passed through to the UI):
// list mode = the tag has no linked memories; search mode = no hits
const emptyNote = computed(() => {
  if (loading.value || total.value !== 0) return ''
  if (searching.value) return t('memories.searchEmpty')
  const tag = tagFilter.value.trim()
  return tag ? t('memories.tagEmpty', { tag }) : ''
})

// Sorting: one select over the (field, direction) pair the list query takes. The old table
// headers were sortable; the card list carries the same capability in the toolbar instead.
const sortChoice = computed({
  get: () => `${sort.value}:${order.value}`,
  set: (value: string) => {
    const [nextSort, nextOrder] = value.split(':')
    sort.value = nextSort as 'updated_at' | 'id'
    order.value = nextOrder as 'asc' | 'desc'
    run(reload)
  },
})

const rootRef = ref<HTMLElement | null>(null)
// <720px switches to narrow mode (dialogs narrow); the card list is flexible-width by nature
const { narrow } = useContainerWidth(rootRef)

const editorVisible = ref(false)
const editingId = ref<string | null>(null)

// Unified action wrapper: toast on failure (error semantics live in the data layer, presentation here)
function run(action: () => Promise<unknown>) {
  return action().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e)))
}

function openCreate() {
  editingId.value = null
  editorVisible.value = true
}

function openEdit(id: string) {
  editingId.value = id
  editorVisible.value = true
}

function onMutated() {
  run(reload)
  loadTagOptions()
  emit('changed')
}

// A permanently mounted panel cannot sense visibility itself: the host calls refresh when
// switching back to this view to pull the latest data (tag options too: renames elsewhere
// must reach the filter dropdown)
defineExpose({
  refresh: () => {
    run(reload)
    loadTagOptions()
  },
})
</script>

<style scoped>
/* Search box takes a full row (flex-basis 100%), the selects wrap to the next line */
.search {
  flex: 1 1 100%;
}
.tag-filter {
  width: 200px;
}
.sort-select {
  width: 140px;
}
.mode-select {
  width: 150px;
}

/* Scrollable card area between toolbar and pager */
.am-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
  /* Room for the loading mask and card focus rings */
  padding: 2px;
}

/* Memory card: quiet bordered row, surface fill on hover — reads as one clickable entry */
.am-memory-card {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 12px 14px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: var(--am-radius-lg);
  background: var(--el-bg-color);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}
.am-memory-card:hover {
  border-color: var(--el-border-color);
  background: var(--el-fill-color-lighter);
}
.card-main {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
  flex: 1;
}
/* Summary first, alone on its line — never sharing a row with the tags */
.card-summary {
  font-weight: 500;
  font-size: 15px;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}
.card-snippet {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  word-break: break-word;
}
.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
/* Right meta column: score (search only), updated time, id */
.card-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 3px;
  flex-shrink: 0;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}
.card-score {
  font-weight: 600;
  color: var(--el-text-color-regular);
}
.card-id {
  font-family: var(--el-font-family-mono, ui-monospace, monospace);
  color: var(--el-text-color-placeholder);
}
</style>
