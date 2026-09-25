<template>
  <div ref="rootRef" class="am-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div>
        <h2 class="am-panel-title">{{ props.title ?? t('tags.title') }}</h2>
        <p class="am-panel-subtitle">{{ props.subtitle ?? t('tags.subtitle') }}</p>
      </div>
      <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('tags.create') }}</el-button>
    </div>

    <el-table :data="rows" v-loading="loading">
      <el-table-column prop="name" :label="t('tags.colName')" min-width="160">
        <template #default="{ row }">
          <el-tag>{{ row.name }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" :label="t('tags.colDescription')" min-width="300" show-overflow-tooltip>
        <template #default="{ row }">{{ row.description || '—' }}</template>
      </el-table-column>
      <el-table-column prop="memory_count" :label="t('tags.colMemoryCount')" width="100" sortable />
      <el-table-column :label="t('tags.colLastUsed')" width="170">
        <template #default="{ row }">{{ formatTime(row.last_used_at) }}</template>
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
      :width="compact ? '96%' : '480px'"
    >
      <el-form label-position="top">
        <el-form-item :label="t('tags.nameLabel')">
          <el-input
            v-model="form.name"
            :disabled="!!form.oldName"
            maxlength="100"
            show-word-limit
            :placeholder="t('tags.namePlaceholder')"
          />
        </el-form-item>
        <template v-if="form.oldName">
          <el-form-item :label="t('tags.renameLabel')">
            <el-input
              v-model="form.newName"
              maxlength="100"
              show-word-limit
              :placeholder="t('tags.renamePlaceholder')"
            />
          </el-form-item>
        </template>
        <el-form-item :label="t('tags.descLabel')">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            maxlength="500"
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
    <el-dialog v-model="deleteVisible" :title="t('tags.deleteTitle')" :width="compact ? '96%' : '480px'">
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
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
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

const { rows, loading, reload, create, rename, remove } = useTags()

const rootRef = ref<HTMLElement | null>(null)
const { compact } = useContainerWidth(rootRef)

const saving = ref(false)
const dialogVisible = ref(false)
const form = reactive({ oldName: null as string | null, name: '', newName: '', description: '' })
const deleteVisible = ref(false)
const deleteMode = ref<'detach' | 'purge'>('detach')
const target = ref<TagView | null>(null)

function openCreate() {
  Object.assign(form, { oldName: null, name: '', newName: '', description: '' })
  dialogVisible.value = true
}

function openEdit(row: TagView) {
  Object.assign(form, { oldName: row.name, name: row.name, newName: '', description: row.description ?? '' })
  dialogVisible.value = true
}

async function save() {
  saving.value = true
  try {
    if (form.oldName) {
      await rename(form.oldName, form.newName, form.description)
      ElMessage.success(t('tags.saved'))
    } else {
      await create(form.name, form.description)
      ElMessage.success(t('tags.created'))
    }
    dialogVisible.value = false
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
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
    ElMessage.success(t('tags.deleted'))
    deleteVisible.value = false
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.delete-body {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  margin: 0 0 16px;
}
</style>
