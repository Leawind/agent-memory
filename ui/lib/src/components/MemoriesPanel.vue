<template>
  <div ref="rootRef" class="am-panel" :class="{ 'is-tight': width > 0 && width < 880, 'is-narrow': narrow }">
    <el-config-provider :locale="elementPlusLocale">
      <div v-if="showHeader" class="am-panel-header">
        <div class="am-heading">
          <div class="am-heading-text">
            <div class="am-title-line">
              <h2 class="am-panel-title">{{ props.title ?? t('memories.title') }}</h2>
              <span v-if="!loading" class="am-total">{{ total }}</span>
            </div>
            <p class="am-panel-subtitle">{{ props.subtitle ?? t('memories.subtitle') }}</p>
          </div>
        </div>
        <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('memories.create') }}</el-button>
      </div>

      <div class="am-toolbar">
        <el-input
          :model-value="query"
          :placeholder="semanticReady ? t('memories.searchPlaceholderSemantic') : t('memories.searchPlaceholder')"
          clearable
          class="search"
          @update:model-value="onQueryInput"
          @keyup.enter="applySearchNow"
          @clear="applySearchNow"
        >
          <template #prefix>
            <el-icon class="search-icon"><Search /></el-icon>
          </template>
        </el-input>
        <TagExprInput
          v-model="tagExpr"
          :tags="tagOptions"
          :placeholder="t('memories.tagExprPlaceholder')"
          :help="t('memories.tagExprHelp')"
          class="tag-filter"
          @apply="onExprApply"
          @update:model-value="onExprChanged"
        />
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
        <div v-if="emptyKind" class="am-empty" role="status">
          <div class="am-empty-icon">
            <el-icon :size="26"><Collection /></el-icon>
          </div>
          <h3 class="am-empty-title">
            {{ emptyKind === 'none' ? t('memories.emptyTitle') : t('memories.noMatchTitle') }}
          </h3>
          <p class="am-empty-hint">
            {{
              emptyKind === 'search'
                ? t('memories.searchEmpty')
                : emptyKind === 'filter'
                  ? t('memories.exprEmpty')
                  : t('memories.emptyHint')
            }}
          </p>
          <el-button v-if="emptyKind === 'none'" type="primary" :icon="Plus" @click="openCreate">
            {{ t('memories.emptyCreate') }}
          </el-button>
          <el-button v-else @click="clearFilters">{{ t('memories.clearFilters') }}</el-button>
        </div>
        <button v-for="card in cards" :key="card.id" type="button" class="am-memory-card" @click="openEdit(card.id)">
          <span class="card-main">
            <span class="card-summary">{{ card.summary }}</span>
            <!-- Snippets are plain text (server sends them unescaped); text interpolation, never v-html -->
            <span v-if="card.snippet" class="card-snippet">{{ card.snippet }}</span>
            <span class="card-tags">
              <!-- A tag chip shows its description on hover (only when one exists) -->
              <el-tooltip
                v-for="tag in card.tags"
                :key="tag"
                :content="tagDescriptions[tag]"
                :disabled="!tagDescriptions[tag]"
                placement="top"
                :enterable="false"
              >
                <el-tag size="small">{{ tag }}</el-tag>
              </el-tooltip>
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

      <div v-if="total > 0" class="am-pager">
        <!-- Hybrid searches report how many hits are literal keyword matches: the rest entered
             via the semantic channel and are looser hits worth eyeing with suspicion -->
        <span v-if="showSearchResults && keywordMatches !== undefined" class="search-km">
          {{ t('memories.keywordHitsFmt', { k: keywordMatches, n: total }) }}
        </span>
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="showSearchResults ? [10, 20, 50] : [20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next"
          @current-change="run(reloadPage)"
          @size-change="run(reloadPage)"
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
import { Collection, Plus, Search } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { toastError } from '../toast'
import { tagExprReady } from '../tagExprGuard'
import { useMemories } from '../composables/useMemories'
import { useContainerWidth } from '../composables/useContainerWidth'
import MemoryEditorDialog from './MemoryEditorDialog.vue'
import TagExprInput from './TagExprInput.vue'

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
  tagExpr,
  mode,
  sort,
  order,
  page,
  pageSize,
  rows,
  searchResults,
  showSearchResults,
  note,
  total,
  keywordMatches,
  loading,
  tagOptions,
  tagDescriptions,
  searching,
  semanticReady,
  onSearch,
  reload,
  reloadPage,
  loadTagOptions,
} = useMemories()

// Unified card shape across list rows and search hits (search adds snippet + score).
// The view follows the settled mode: while a search intent has no data yet (typing, or the
// request in flight) the last settled cards stay on screen instead of flashing empty.
interface MemoryCard {
  id: string
  summary: string
  tags: string[]
  updated: string
  snippet?: string
  score?: number
}
const cards = computed<MemoryCard[]>(() =>
  searching.value && showSearchResults.value
    ? searchResults.value.map((r) => ({
        id: r.id,
        summary: r.summary,
        tags: r.tags,
        updated: r.updated,
        snippet: r.snippet,
        score: r.score,
      }))
    : rows.value.map((r) => ({
        id: r.id,
        summary: r.summary,
        tags: r.tags,
        updated: r.updated,
      })),
)

// Which empty state to show (null while loading or when there is something to list).
// list mode with no filter = the store is empty; a filter or search explains the mismatch and
// offers a way back.
const emptyKind = computed<'none' | 'filter' | 'search' | null>(() => {
  if (loading.value || cards.value.length > 0 || total.value > 0) return null
  if (searching.value && showSearchResults.value) return 'search'
  if (tagExpr.value.trim()) return 'filter'
  return 'none'
})

// Typing applies through a debounce (a search term is composed over several keystrokes), the
// same contract as the tag expression box; Enter / clear / the magnifier apply immediately.
let searchTimer: ReturnType<typeof setTimeout> | undefined
function onQueryInput(v: string) {
  query.value = v
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    void run(onSearch)
  }, 400)
}
function applySearchNow() {
  clearTimeout(searchTimer)
  void run(onSearch)
}

// Typing applies through a debounce (an expression is composed over several keystrokes);
// Enter / clear / suggestion selection apply immediately. The debounced path is gated on
// local completeness (tagExprReady): a half-typed expression — unterminated quote, dangling
// operator, trailing word that is not a known tag yet — would only draw an unknown-tag error
// toast per keystroke, so it waits. The apply path always sends: a deliberately applied
// expression deserves the server's precise did-you-mean error.
let exprTimer: ReturnType<typeof setTimeout> | undefined
function onExprChanged() {
  clearTimeout(exprTimer)
  exprTimer = setTimeout(() => {
    if (tagExprReady(tagExpr.value, tagOptions.value)) void run(onSearch)
  }, 400)
}
function onExprApply() {
  clearTimeout(exprTimer)
  void run(onSearch)
}

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
// <880px: the search field gets its own row; <720px: each text field gets a row and the
// selects split the last one. Kept separate from `compact` (960) because the four toolbar
// controls happen to fit side by side down to roughly 880px.
const { narrow, width } = useContainerWidth(rootRef)

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

// Sidebar tag click lands here (through the workspace): the tag becomes the whole expression —
// a click is a deliberate single-tag view, not an edit of whatever expression was typed. Clicking
// the already-active tag clears the filter. Applied immediately: the tag is known to exist.
// Keyed tags (name carries ':') pass verbatim: the expression tokenizer keeps ':' inside bare
// words, so the name is a valid exact-tag atom.
function toggleTagFilter(name: string) {
  tagExpr.value = tagExpr.value.trim() === name ? '' : name
  run(onSearch)
}

// Empty-state action: drop both the keyword and tag filters, then reload the whole list
function clearFilters() {
  clearTimeout(searchTimer)
  clearTimeout(exprTimer)
  query.value = ''
  tagExpr.value = ''
  void run(onSearch)
}

// Sidebar "all memories" row: drop just the tag expression (a keyword search stays as typed)
function clearTagFilter() {
  if (!tagExpr.value) return
  clearTimeout(exprTimer)
  tagExpr.value = ''
  void run(onSearch)
}

// A permanently mounted panel cannot sense visibility itself: the host calls refresh when
// switching back to this view to pull the latest data (tag options too: renames elsewhere
// must reach the expression suggestions)
defineExpose({
  refresh: () => {
    run(reload)
    loadTagOptions()
  },
  toggleTagFilter,
  clearTagFilter,
  // Read by the workspace so the sidebar can highlight the row matching the expression
  tagExpr,
})
</script>

<style scoped>
/* Header: title + count badge on one line, the explanation visible underneath (it used to be
   hidden behind an info tooltip) */
.am-heading-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.am-title-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}
.am-total {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 20px;
  padding: 0 7px;
  border-radius: 999px;
  background: var(--el-fill-color);
  color: var(--el-text-color-secondary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}
.am-panel-subtitle {
  margin: 0;
  overflow: hidden;
  font-size: 13px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Toolbar: search and the tag expression share the free space; the selects stay compact */
.search {
  flex: 2 1 260px;
}
.tag-filter {
  flex: 2 1 240px;
}
.sort-select {
  flex: 0 0 auto;
  width: 132px;
}
.mode-select {
  flex: 0 0 auto;
  width: 142px;
}
.search-icon {
  color: var(--el-text-color-placeholder);
}
/* Tight panel: the search field takes a full row; the tag expression and the two selects
   share the next one (the tag expression grows into the leftover space). */
.am-panel.is-tight .search {
  flex-basis: 100%;
}
/* Narrow panel: each text field gets its own row, the selects split the last row */
.am-panel.is-narrow .search,
.am-panel.is-narrow .tag-filter {
  flex-basis: 100%;
}
.am-panel.is-narrow .sort-select,
.am-panel.is-narrow .mode-select {
  flex: 1 1 0;
  width: auto;
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

/* Empty state: centered in the remaining list height, with the one action that makes sense */
.am-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin: auto;
  padding: 40px 24px;
  max-width: 420px;
  text-align: center;
}
.am-empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border-radius: 20px;
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}
.am-empty-title {
  margin: 16px 0 6px;
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.am-empty-hint {
  margin: 0 0 18px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
}

/* Memory card: quiet bordered row, surface fill on hover — reads as one clickable entry */
.am-memory-card {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 13px 15px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: var(--am-radius-lg);
  background: var(--el-bg-color);
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background 0.15s ease,
    box-shadow 0.15s ease;
}
.am-memory-card:hover {
  border-color: var(--el-border-color);
  background: var(--el-fill-color-lighter);
  box-shadow: var(--el-box-shadow-light);
}
.am-memory-card:focus-visible {
  outline: 2px solid var(--el-color-primary-light-5);
  outline-offset: 2px;
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
  font-weight: 600;
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
.am-pager {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  min-height: 32px;
}
.search-km {
  margin-right: auto;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
</style>
