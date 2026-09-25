<template>
  <el-dialog
    :model-value="visible"
    :title="form.id ? '编辑记忆' : '新建记忆'"
    :width="width"
    @update:model-value="emit('update:visible', $event)"
  >
    <el-form label-position="top">
      <el-form-item label="摘要（列表与搜索展示的一行简介）">
        <el-input v-model="form.summary" maxlength="512" show-word-limit placeholder="精确、自洽的一句话" />
      </el-form-item>
      <el-form-item>
        <template #label>
          <div class="content-label">
            <span>正文（Markdown）</span>
            <el-radio-group v-model="contentTab" size="small">
              <el-radio-button value="edit">编辑</el-radio-button>
              <el-radio-button value="preview">预览</el-radio-button>
            </el-radio-group>
          </div>
        </template>
        <el-input
          v-if="contentTab === 'edit'"
          v-model="form.content"
          type="textarea"
          :rows="12"
          maxlength="200000"
          show-word-limit
          placeholder="支持 Markdown：标题、列表、代码块、表格……"
        />
        <MarkdownView v-else class="content-preview" :source="form.content" />
      </el-form-item>
      <el-form-item label="标签（回车添加，可新建）">
        <el-select
          v-model="form.tags"
          multiple
          filterable
          allow-create
          default-first-option
          placeholder="选择或输入标签"
          class="tags-select"
        >
          <el-option v-for="t in tagOptions" :key="t" :label="t" :value="t" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="emit('update:visible', false)">取消</el-button>
      <el-button type="primary" :loading="saving" @click="save">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useApiClient } from '../api/client'
import { createMemory, getMemory, updateMemory } from '../api/memories'
import type { MemoryDraft } from '../composables/useMemories'
import MarkdownView from './MarkdownView.vue'

const props = defineProps<{
  visible: boolean
  /** 编辑目标的记忆 id；null 表示新建 */
  memoryId: string | null
  tagOptions: string[]
  /** 弹层宽度；窄容器由面板传入收窄值 */
  width?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  /** 保存成功（新建或更新完成） */
  saved: []
}>()

const client = useApiClient()

const saving = ref(false)
const contentTab = ref<'edit' | 'preview'>('edit')
// 正文编辑器的视图：编辑源码或预览 Markdown 渲染结果
const form = ref<MemoryDraft>({ id: null, summary: '', content: '', tags: [] })
// 编辑时的原始标签集，保存时换算成 add/remove
let originalTags: string[] = []

watch(
  () => props.visible,
  async (open) => {
    if (!open) return
    contentTab.value = 'edit'
    if (props.memoryId) {
      try {
        const full = await getMemory(client, props.memoryId)
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: [...full.tags] }
        originalTags = [...full.tags]
      } catch (e: unknown) {
        ElMessage.error(e instanceof Error ? e.message : String(e))
        emit('update:visible', false)
      }
    } else {
      form.value = { id: null, summary: '', content: '', tags: [] }
      originalTags = []
    }
  },
)

async function save() {
  saving.value = true
  try {
    if (form.value.id) {
      // 编辑：以原始标签为基线换算 add_tags / remove_tags
      const before = new Set(originalTags)
      const after = new Set(form.value.tags)
      await updateMemory(client, form.value.id, {
        summary: form.value.summary,
        content: form.value.content,
        add_tags: [...after].filter((t) => !before.has(t)),
        remove_tags: [...before].filter((t) => !after.has(t)),
      })
      ElMessage.success('已更新')
    } else {
      await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        tags: form.value.tags,
      })
      ElMessage.success('已创建')
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.content-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}
.tags-select {
  width: 100%;
}
.content-preview {
  width: 100%;
  box-sizing: border-box;
  /* Mirror the textarea frame so switching views keeps the field outline. */
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  padding: 5px 11px;
  min-height: 260px;
}
</style>
