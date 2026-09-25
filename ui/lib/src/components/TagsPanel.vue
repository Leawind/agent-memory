<template>
  <div ref="rootRef" class="am-panel">
    <div class="am-panel-header">
      <div>
        <h2 class="am-panel-title">标签管理</h2>
        <p class="am-panel-subtitle">标签是 agent 自主维护的分类体系；改名会同步更新所有引用它的记忆</p>
      </div>
      <el-button type="primary" :icon="Plus" @click="openCreate">新建标签</el-button>
    </div>

    <el-table :data="rows" v-loading="loading">
      <el-table-column prop="name" label="名称" min-width="160">
        <template #default="{ row }">
          <el-tag>{{ row.name }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" label="描述" min-width="300" show-overflow-tooltip>
        <template #default="{ row }">{{ row.description || '—' }}</template>
      </el-table-column>
      <el-table-column prop="memory_count" label="记忆数" width="100" sortable />
      <el-table-column label="最近使用" width="170">
        <template #default="{ row }">{{ formatTime(row.last_used_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="150" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openEdit(row)">编辑</el-button>
          <el-button link type="danger" @click="openDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog
      v-model="dialogVisible"
      :title="form.oldName ? '编辑标签' : '新建标签'"
      :width="compact ? '96%' : '480px'"
    >
      <el-form label-position="top">
        <el-form-item label="名称（唯一，≤100 字符）">
          <el-input
            v-model="form.name"
            :disabled="!!form.oldName"
            maxlength="100"
            show-word-limit
            placeholder="如 rust、项目、工作流"
          />
        </el-form-item>
        <template v-if="form.oldName">
          <el-form-item label="改为新名称（留空表示不改名）">
            <el-input
              v-model="form.newName"
              maxlength="100"
              show-word-limit
              placeholder="仅大小写改名也可用于合并拼写偏差"
            />
          </el-form-item>
        </template>
        <el-form-item label="描述（可选，≤500 字符）">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            maxlength="500"
            show-word-limit
            placeholder="这个标签用来组织什么内容"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="deleteVisible" title="删除标签" :width="compact ? '96%' : '480px'">
      <p>
        标签 <el-tag>{{ target?.name }}</el-tag> 当前被 <b>{{ target?.memory_count }}</b> 条记忆使用。请选择删除方式：
      </p>
      <el-radio-group v-model="deleteMode">
        <el-radio value="detach">仅摘除引用（保留全部记忆）</el-radio>
        <el-radio value="purge">连带删除记忆（不可恢复）</el-radio>
      </el-radio-group>
      <template #footer>
        <el-button @click="deleteVisible = false">取消</el-button>
        <el-button type="danger" :loading="saving" @click="doDelete">删除</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { useTags } from '../composables/useTags'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { TagView } from '../types'

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
      ElMessage.success('已保存')
    } else {
      await create(form.name, form.description)
      ElMessage.success('已创建')
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
        `将永久删除标签「${target.value?.name}」及其关联的 ${target.value?.memory_count ?? 0} 条记忆，且不可恢复！`,
        '高危操作确认',
        { type: 'error', confirmButtonText: '永久删除' },
      )
    } catch {
      return
    }
  }
  if (!target.value) return
  saving.value = true
  try {
    await remove(target.value.name, deleteMode.value)
    ElMessage.success('已删除')
    deleteVisible.value = false
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}
</script>
