<template>
  <div ref="rootRef" class="am-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div>
        <h2 class="am-panel-title">{{ props.title ?? t('memories.title') }}</h2>
        <p class="am-panel-subtitle">{{ props.subtitle ?? t('memories.subtitle') }}</p>
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
      <el-select v-model="sort" class="sort" @change="run(reload)">
        <el-option :label="t('memories.sortUpdated')" value="updated_at" />
        <el-option :label="t('memories.sortCreated')" value="created_at" />
      </el-select>
      <el-select v-model="order" class="order" @change="run(reload)">
        <el-option :label="t('memories.orderDesc')" value="desc" />
        <el-option :label="t('memories.orderAsc')" value="asc" />
      </el-select>
    </div>

    <el-alert v-if="note" :title="note" type="info" show-icon :closable="false" />

    <!-- 搜索模式：显示匹配片段与评分 -->
    <el-table v-if="searching" :data="searchResults" v-loading="loading">
      <el-table-column prop="id" :label="t('memories.colId')" width="80" />
      <el-table-column :label="t('memories.colSummary')">
        <template #default="{ row }">
          <div class="am-summary">{{ row.summary }}</div>
          <div class="am-snippet" v-html="row.snippet" />
        </template>
      </el-table-column>
      <el-table-column :label="t('memories.colTags')" width="220">
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

    <!-- 列表模式 -->
    <el-table v-else :data="rows" v-loading="loading">
      <el-table-column prop="id" :label="t('memories.colId')" width="80" />
      <el-table-column prop="summary" :label="t('memories.colSummary')" min-width="300" show-overflow-tooltip />
      <el-table-column :label="t('memories.colTags')" width="220">
        <template #default="{ row }">
          <el-tag v-for="tag in row.tags" :key="tag" size="small" class="am-tag">{{ tag }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column v-if="!compact" prop="created_at" :label="t('memories.colCreatedAt')" width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
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
      :width="compact ? '96%' : '640px'"
      @saved="onSaved"
    />

    <!-- 全文详情 -->
    <MemoryDetailDrawer v-model:visible="detailVisible" :memory-id="detailId" :size="compact ? '100%' : '45%'" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Search } from '@element-plus/icons-vue'
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
  sort,
  order,
  page,
  pageSize,
  rows,
  searchResults,
  total,
  note,
  loading,
  tagOptions,
  searching,
  onSearch,
  reload,
  loadTagOptions,
  removeMemory,
} = useMemories()

const rootRef = ref<HTMLElement | null>(null)
// 窄容器（宿主侧栏、卡片等）自动切 compact：隐藏低优先级列、收窄弹层
const { compact } = useContainerWidth(rootRef)

const editorVisible = ref(false)
const editingId = ref<string | null>(null)
const detailVisible = ref(false)
const detailId = ref<string | null>(null)

// 动作统一包装：失败 toast（错误语义在数据层，展示在这里）
function run(action: () => Promise<unknown>) {
  return action().catch((e: unknown) => ElMessage.error(e instanceof Error ? e.message : String(e)))
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
    ElMessage.success(t('memories.deleted'))
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  }
}
</script>

<style scoped>
.search {
  width: 360px;
  max-width: 100%;
}
.tag-filter {
  width: 160px;
}
.sort,
.order {
  width: 130px;
}
</style>
