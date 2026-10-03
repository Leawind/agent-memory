<template>
  <el-dialog
    :model-value="visible"
    :title="form.id ? t('editor.editTitle') : t('editor.createTitle')"
    :width="width"
    align-center
    append-to-body
    class="memory-editor-dialog"
    @update:model-value="emit('update:visible', $event)"
  >
    <el-form label-position="top">
      <el-form-item :label="t('editor.summaryLabel')">
        <el-input
          v-model="form.summary"
          maxlength="512"
          show-word-limit
          :placeholder="t('editor.summaryPlaceholder')"
        />
      </el-form-item>
      <el-form-item :label="t('editor.tagsLabel')">
        <el-select
          v-model="form.tags"
          multiple
          filterable
          allow-create
          default-first-option
          :placeholder="t('editor.tagsPlaceholder')"
          class="tags-select"
        >
          <el-option v-for="tag in tagOptions" :key="tag" :label="tag" :value="tag" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <template #label>
          <div class="content-label">
            <span>{{ t('editor.contentLabel') }}</span>
            <MarkdownModeToggle v-model="contentTab" />
          </div>
        </template>
        <el-input
          v-if="contentTab === 'edit'"
          v-model="form.content"
          type="textarea"
          :rows="12"
          maxlength="262144"
          show-word-limit
          :placeholder="t('editor.contentPlaceholder')"
        />
        <MarkdownView v-else class="content-preview" :source="form.content" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="emit('update:visible', false)">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="saving" @click="save">{{ t('common.save') }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createMemory, getMemory, updateMemory } from '../api/memories'
import type { MemoryDraft } from '../composables/useMemories'
import MarkdownView from './MarkdownView.vue'
import MarkdownModeToggle from './MarkdownModeToggle.vue'
import { t } from '../i18n'

const props = defineProps<{
  visible: boolean
  /** The memory id being edited; null means creating new */
  memoryId: string | null
  tagOptions: string[]
  /** Dialog width; the panel passes a narrower value in narrow containers */
  width?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  /** Saved successfully (create or update finished) */
  saved: []
}>()

const client = useApiClient()

const saving = ref(false)
const contentTab = ref<'edit' | 'preview'>('preview')
// Content editor view: edit the source or preview the rendered Markdown
const form = ref<MemoryDraft>({ id: null, summary: '', content: '', tags: [] })
// Original tag set while editing, converted to add/remove on save
let originalTags: string[] = []

watch(
  () => props.visible,
  async (open) => {
    if (!open) return
    // Editing an existing memory opens rendered; creating starts in the source editor
    contentTab.value = props.memoryId ? 'preview' : 'edit'
    if (props.memoryId) {
      try {
        const full = await getMemory(client, props.memoryId)
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: [...full.tags] }
        originalTags = [...full.tags]
      } catch (e: unknown) {
        toastError(e instanceof Error ? e.message : String(e))
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
      // Editing: compute add_tags / remove_tags against the original tag set as baseline
      const before = new Set(originalTags)
      const after = new Set(form.value.tags)
      await updateMemory(client, form.value.id, {
        summary: form.value.summary,
        content: form.value.content,
        add_tags: [...after].filter((t) => !before.has(t)),
        remove_tags: [...before].filter((t) => !after.has(t)),
      })
      toastSuccess(t('editor.updated'))
    } else {
      await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        tags: form.value.tags,
      })
      toastSuccess(t('editor.created'))
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
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
.content-label > :last-child {
  margin-left: auto;
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

<!-- Unscoped: with append-to-body the dialog element lives outside this component tree -->
<style>
.memory-editor-dialog {
  resize: both;
  overflow: auto;
  max-width: 95vw;
  max-height: 90vh;
}
.memory-editor-dialog .el-dialog__body {
  max-height: 65vh;
  overflow-y: auto;
}
</style>
