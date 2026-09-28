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
        <el-select v-model="mode" class="mode-select" @change="run(onSearch)">
          <el-option :label="t('memories.modeAuto')" value="auto" />
          <el-option :label="t('memories.modeKeyword')" value="keyword" />
          <el-option :label="t('memories.modeHybrid')" value="hybrid" />
        </el-select>
      </div>

      <el-alert v-if="note" :title="note" type="warning" show-icon :closable="false" />
      <el-alert v-if="emptyNote" :title="emptyNote" type="info" show-icon :closable="false" />

      <!-- Search mode: show matched snippets and scores -->
      <el-table v-if="searching" :data="searchResults" v-loading="loading">
        <el-table-column prop="id" :label="t('memories.colId')" width="80" />
        <el-table-column :label="t('memories.colSummary')">
          <template #default="{ row }">
            <div class="am-summary">{{ row.summary }}</div>
            <div class="am-snippet" v-html="row.snippet" />
          </template>
        </el-table-column>
        <el-table-column :label="t('memories.colTags')" min-width="150">
          <template #default="{ row }">
            <el-tag v-for="tag in row.tags" :key="tag" size="small" class="am-tag">{{ tag }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="score" :label="t('memories.colScore')" width="80" sortable />
        <el-table-column :label="t('memories.colUpdatedAt')" width="170">
          <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
        </el-table-column>
        <el-table-column :label="t('memories.colActions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
              <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="openEdit(row.id)" />
            </el-tooltip>
            <el-tooltip :content="t('common.delete')" placement="top" :enterable="false">
              <el-button link type="danger" :icon="Delete" :aria-label="t('common.delete')" @click="remove(row)" />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>

      <!-- List mode: click the created/updated column headers to sort (convention: direction shown on the right of the active header, click to toggle) -->
      <el-table
        v-else
        :data="rows"
        v-loading="loading"
        :default-sort="{ prop: sort, order: order === 'asc' ? 'ascending' : 'descending' }"
        @sort-change="onSortChange"
      >
        <el-table-column prop="id" :label="t('memories.colId')" width="80" sortable="custom" />
        <el-table-column prop="summary" :label="t('memories.colSummary')" min-width="180" show-overflow-tooltip />
        <el-table-column :label="t('memories.colTags')" min-width="150">
          <template #default="{ row }">
            <el-tag v-for="tag in row.tags" :key="tag" size="small" class="am-tag">{{ tag }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column
          v-if="!compact"
          prop="created_at"
          :label="t('memories.colCreatedAt')"
          width="170"
          sortable="custom"
        >
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column prop="updated_at" :label="t('memories.colUpdatedAt')" width="170" sortable="custom">
          <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
        </el-table-column>
        <el-table-column :label="t('memories.colActions')" width="100" fixed="right">
          <template #default="{ row }">
            <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
              <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="openEdit(row.id)" />
            </el-tooltip>
            <el-tooltip :content="t('common.delete')" placement="top" :enterable="false">
              <el-button link type="danger" :icon="Delete" :aria-label="t('common.delete')" @click="remove(row)" />
            </el-tooltip>
          </template>
        </el-table-column>
      </el-table>

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

      <!-- Create / edit -->
      <MemoryEditorDialog
        v-model:visible="editorVisible"
        :memory-id="editingId"
        :tag-options="tagOptions"
        :width="narrow ? '96%' : '640px'"
        @saved="onSaved"
      />
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { ElMessageBox } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { Delete, Edit, InfoFilled, Plus, Search } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { t } from '../i18n'
import { useMemories } from '../composables/useMemories'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { MemorySummary } from '../types'
import MemoryEditorDialog from './MemoryEditorDialog.vue'

const props = withDefaults(
  defineProps<{
    /** Hide the title/subtitle area (when embedded in a host that already has a page title, only the toolbar + table are wanted) */
    showHeader?: boolean
    /** Override the default title */
    title?: string
    /** Override the default subtitle */
    subtitle?: string
  }>(),
  { showHeader: true },
)

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
  removeMemory,
} = useMemories()

// The empty-result notice is generated locally (the server's note targets agents and is English
// contract text, not passed through to the UI):
// list mode = the tag has no linked memories; search mode = no hits
const emptyNote = computed(() => {
  if (loading.value || total.value !== 0) return ''
  if (searching.value) return t('memories.searchEmpty')
  const tag = tagFilter.value.trim()
  return tag ? t('memories.tagEmpty', { tag }) : ''
})

const rootRef = ref<HTMLElement | null>(null)
// Container <960px switches to compact (hides the created-at column so the updated-at/actions
// columns are not pushed out of the viewport);
// <720px switches to narrow (dialogs narrow)
const { compact, narrow } = useContainerWidth(rootRef)

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

function onSaved() {
  run(reload)
  loadTagOptions()
}

/** Header sorting: clicking a column head toggles asc/desc, a third click cancels and returns to the default (most recently updated first). */
function onSortChange(payload: { prop: string; order: 'ascending' | 'descending' | null }) {
  // The parameter must not be named order — it would shadow the outer order ref and the
  // assignment would land on the parameter (strict mode throws a TypeError outright)
  const { prop, order: nextOrder } = payload
  if (nextOrder && (prop === 'updated_at' || prop === 'created_at' || prop === 'id')) {
    sort.value = prop
    order.value = nextOrder === 'ascending' ? 'asc' : 'desc'
  } else {
    sort.value = 'updated_at'
    order.value = 'desc'
  }
  run(reload)
}

async function remove(row: MemorySummary) {
  try {
    await ElMessageBox.confirm(t('memories.deleteConfirm', { id: row.id }), t('memories.deleteTitle'), {
      type: 'warning',
    })
  } catch {
    return
  }
  try {
    await removeMemory(row.id)
    toastSuccess(t('memories.deleted'))
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  }
}

// A permanently mounted panel cannot sense visibility itself: the host calls refresh when switching back to this panel to pull the latest data
defineExpose({ refresh: () => run(reload) })
</script>

<style scoped>
/* Search box takes a full row (flex-basis 100%), tag filter wraps to the next line */
.search {
  flex: 1 1 100%;
}
.tag-filter {
  width: 240px;
}
.mode-select {
  width: 150px;
}
</style>
