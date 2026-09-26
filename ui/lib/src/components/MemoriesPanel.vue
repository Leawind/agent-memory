<template>
  <div ref="rootRef" class="am-panel">
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

    <el-alert v-if="emptyNote" :title="emptyNote" type="info" show-icon :closable="false" />

    <!-- 搜索模式：显示匹配片段与评分 -->
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
      <el-table-column :label="t('memories.colActions')" width="190" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openDetail(row.id)">{{ t('common.detail') }}</el-button>
          <el-button link type="primary" @click="openEdit(row.id)">{{ t('common.edit') }}</el-button>
          <el-button link type="danger" @click="remove(row)">{{ t('common.delete') }}</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 列表模式：创建/更新时间列点击表头排序（惯例：当前序列头右侧显示方向，点击切换） -->
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
      <el-table-column :label="t('memories.colActions')" width="190" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openDetail(row.id)">{{ t('common.detail') }}</el-button>
          <el-button link type="primary" @click="openEdit(row.id)">{{ t('common.edit') }}</el-button>
          <el-button link type="danger" @click="remove(row)">{{ t('common.delete') }}</el-button>
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

    <!-- 新建 / 编辑 -->
    <MemoryEditorDialog
      v-model:visible="editorVisible"
      :memory-id="editingId"
      :tag-options="tagOptions"
      :width="narrow ? '96%' : '640px'"
      @saved="onSaved"
    />

    <!-- 全文详情 -->
    <MemoryDetailDrawer v-model:visible="detailVisible" :memory-id="detailId" :size="narrow ? '100%' : '45%'" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { InfoFilled, Plus, Search } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { t } from '../i18n'
import { useMemories } from '../composables/useMemories'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { MemorySummary } from '../types'
import MemoryEditorDialog from './MemoryEditorDialog.vue'
import MemoryDetailDrawer from './MemoryDetailDrawer.vue'

const props = withDefaults(
  defineProps<{
    /** 隐藏标题/副标题区（嵌入宿主已有页面标题时只要工具栏+表格） */
    showHeader?: boolean
    /** 覆盖默认标题 */
    title?: string
    /** 覆盖默认副标题 */
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
  total,
  loading,
  tagOptions,
  searching,
  onSearch,
  reload,
  loadTagOptions,
  removeMemory,
} = useMemories()

// 空结果提示本地生成（服务端 note 面向 agent，是英文契约文本，不在界面透传）：
// 列表模式 = 标签无关联记忆；搜索模式 = 无命中
const emptyNote = computed(() => {
  if (loading.value || total.value !== 0) return ''
  if (searching.value) return t('memories.searchEmpty')
  const tag = tagFilter.value.trim()
  return tag ? t('memories.tagEmpty', { tag }) : ''
})

const rootRef = ref<HTMLElement | null>(null)
// 容器 <960px 切 compact（隐藏创建时间列，保证更新时间/操作列不被挤出视口）；
// <720px 切 narrow（弹层收窄）
const { compact, narrow } = useContainerWidth(rootRef)

const editorVisible = ref(false)
const editingId = ref<string | null>(null)
const detailVisible = ref(false)
const detailId = ref<string | null>(null)

// 动作统一包装：失败 toast（错误语义在数据层，展示在这里）
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

function openDetail(id: string) {
  detailId.value = id
  detailVisible.value = true
}

function onSaved() {
  run(reload)
  loadTagOptions()
}

/** 表头排序：点击列头切换升降序，第三次点击取消并回到默认（最近更新在前）。 */
function onSortChange(payload: { prop: string; order: 'ascending' | 'descending' | null }) {
  // 参数名不可叫 order——会遮蔽外部的 order ref，赋值落回参数上（严格模式直接抛 TypeError）
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

// 面板常驻挂载时无法自行感知可见性：宿主切回此面板时调 refresh 拉最新数据
defineExpose({ refresh: () => run(reload) })
</script>

<style scoped>
/* 搜索框独占一行（flex-basis 100%），标签过滤自然换行到下一行 */
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
