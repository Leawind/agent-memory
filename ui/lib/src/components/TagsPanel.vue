<template>
  <div ref="rootRef" class="am-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div class="am-heading">
        <h2 class="am-panel-title">{{ props.title ?? t('tags.title') }}</h2>
        <el-tooltip :content="props.subtitle ?? t('tags.subtitle')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
      </div>
      <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('tags.create') }}</el-button>
    </div>

    <!-- 标签名正则过滤（服务端执行；非法正则由服务端报错并 toast） -->
    <el-input
      v-model="filter"
      class="am-tag-filter"
      :placeholder="t('tags.filterPlaceholder')"
      clearable
      :prefix-icon="Search"
      @input="applyFilterDebounced"
      @clear="applyFilterDebounced"
    />

    <el-table :data="rows" v-loading="loading">
      <el-table-column prop="name" :label="t('tags.colName')" min-width="140" sortable>
        <template #default="{ row }">
          <el-tag>{{ row.name }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" :label="t('tags.colDescription')" min-width="150" show-overflow-tooltip>
        <template #default="{ row }">{{ row.description || '—' }}</template>
      </el-table-column>
      <el-table-column prop="memory_count" :label="t('tags.colMemoryCount')" width="90" sortable />
      <el-table-column
        prop="last_used_at"
        :label="t('tags.colLastUsed')"
        width="170"
        sortable
        :sort-method="byTimeField('last_used_at')"
      >
        <template #default="{ row }">{{ formatTime(row.last_used_at) }}</template>
      </el-table-column>
      <el-table-column
        prop="created_at"
        :label="t('tags.colCreatedAt')"
        width="170"
        sortable
        :sort-method="byTimeField('created_at')"
      >
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column :label="t('memories.colActions')" width="150" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openEdit(row)">{{ t('common.edit') }}</el-button>
          <el-button link type="danger" @click="openDelete(row)">{{ t('common.delete') }}</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog
      v-model="dialogVisible"
      :title="form.oldName ? t('tags.editTitle') : t('tags.createTitle')"
      :width="narrow ? '96%' : '480px'"
    >
      <el-form label-position="top">
        <!-- 编辑时预填当前名：微小修改直接改，不再要求重输全名；与服务端约定同名提交 = 不改名 -->
        <el-form-item :label="t('tags.nameLabel')">
          <el-input v-model="form.name" maxlength="100" show-word-limit :placeholder="t('tags.namePlaceholder')" />
        </el-form-item>
        <el-form-item :label="t('tags.descLabel')">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            maxlength="512"
            show-word-limit
            :placeholder="t('tags.descPlaceholder')"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="saving" @click="save">{{ t('common.save') }}</el-button>
      </template>
    </el-dialog>

    <!-- 删除方式选择：富文本结构（标签名/计数）无法整句插值，拆为三段固定语序 -->
    <el-dialog v-model="deleteVisible" :title="t('tags.deleteTitle')" :width="narrow ? '96%' : '480px'">
      <p class="delete-body">
        {{ t('tags.deleteBefore') }}
        <el-tag>{{ target?.name }}</el-tag>
        {{ t('tags.deleteMiddle') }}
        <b>{{ target?.memory_count }}</b>
        {{ t('tags.deleteAfter') }}
      </p>
      <el-radio-group v-model="deleteMode">
        <el-radio value="detach">{{ t('tags.detach') }}</el-radio>
        <el-radio value="purge">{{ t('tags.purge') }}</el-radio>
      </el-radio-group>
      <template #footer>
        <el-button @click="deleteVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="danger" :loading="saving" @click="doDelete">{{ t('common.delete') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { InfoFilled, Plus, Search } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { t } from '../i18n'
import { useTags } from '../composables/useTags'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { TagView } from '../types'

const props = withDefaults(
  defineProps<{
    /** 隐藏标题/副标题区（嵌入宿主已有页面标题时只要表格） */
    showHeader?: boolean
    /** 覆盖默认标题 */
    title?: string
    /** 覆盖默认副标题 */
    subtitle?: string
  }>(),
  { showHeader: true },
)

const { rows, loading, filter, reload, create, rename, remove } = useTags()

// 正则过滤输入 → 防抖后重新拉取；非法正则的错误由 reload 的异常路径 toast
let filterTimer: ReturnType<typeof setTimeout> | null = null
function applyFilterDebounced() {
  if (filterTimer) clearTimeout(filterTimer)
  filterTimer = setTimeout(() => {
    filterTimer = null
    reload().catch((e) => toastError(e instanceof Error ? e.message : String(e)))
  }, 300)
}

const rootRef = ref<HTMLElement | null>(null)
// narrow（<720px）时弹层收窄；表格列均为可伸缩宽度，窄容器不丢列
const { narrow } = useContainerWidth(rootRef)

const saving = ref(false)
const dialogVisible = ref(false)
const form = reactive({ oldName: null as string | null, name: '', description: '' })
const deleteVisible = ref(false)
const deleteMode = ref<'detach' | 'purge'>('detach')
const target = ref<TagView | null>(null)

/** 时间列排序（last_used_at 可为 null，按 0 参与；纯数字比较避免默认字典序） */
function byTimeField(field: 'last_used_at' | 'created_at') {
  return (a: TagView, b: TagView) => (a[field] ?? 0) - (b[field] ?? 0)
}

function openCreate() {
  Object.assign(form, { oldName: null, name: '', description: '' })
  dialogVisible.value = true
}

function openEdit(row: TagView) {
  Object.assign(form, { oldName: row.name, name: row.name, description: row.description ?? '' })
  dialogVisible.value = true
}

async function save() {
  saving.value = true
  try {
    if (form.oldName) {
      // 名称与原名相同（或空白）时 api 层不下发 new_name，即只更新描述
      await rename(form.oldName, form.name, form.description)
      toastSuccess(t('tags.saved'))
    } else {
      await create(form.name, form.description)
      toastSuccess(t('tags.created'))
    }
    dialogVisible.value = false
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

function openDelete(row: TagView) {
  target.value = row
  deleteMode.value = 'detach'
  deleteVisible.value = true
}

async function doDelete() {
  if (deleteMode.value === 'purge') {
    try {
      await ElMessageBox.confirm(
        t('tags.purgeConfirm', { name: target.value?.name, count: target.value?.memory_count ?? 0 }),
        t('tags.purgeConfirmTitle'),
        { type: 'error', confirmButtonText: t('tags.purgeButton') },
      )
    } catch {
      return
    }
  }
  if (!target.value) return
  saving.value = true
  try {
    await remove(target.value.name, deleteMode.value)
    toastSuccess(t('tags.deleted'))
    deleteVisible.value = false
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

// 面板常驻挂载时无法自行感知可见性：宿主切回此面板时调 refresh 拉最新数据
defineExpose({
  refresh: () => reload().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e))),
})
</script>

<style scoped>
.am-tag-filter {
  margin-bottom: 12px;
  width: 100%;
}
.delete-body {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  margin: 0 0 16px;
}
</style>
