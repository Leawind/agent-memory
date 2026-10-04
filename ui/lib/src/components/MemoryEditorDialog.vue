<template>
  <FormDialog
    :visible="visible"
    :title="memoryId ? t('editor.editTitle') : t('editor.createTitle')"
    :width="width"
    align-center
    append-to-body
    :saving="saving"
    :submit-disabled="loading"
    @update:visible="emit('update:visible', $event)"
    @submit="save"
  >
    <div v-loading="loading">
      <el-form-item :label="t('editor.summaryLabel')">
        <el-input
          ref="summaryInput"
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
    </div>
  </FormDialog>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { InputInstance } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createMemory, getMemory, updateMemory } from '../api/memories'
import type { MemoryDraft } from '../composables/useMemories'
import FormDialog from './FormDialog.vue'
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
const summaryInput = ref<InputInstance | null>(null)
// Fetching the full memory on open; while true the (still empty) form must not be savable
const loading = ref(false)
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
    // Reset optimistically so a slow fetch never shows the previously edited memory's data
    form.value = { id: null, summary: '', content: '', tags: [] }
    originalTags = []
    if (props.memoryId) {
      loading.value = true
      try {
        const full = await getMemory(client, props.memoryId)
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: [...full.tags] }
        originalTags = [...full.tags]
      } catch (e: unknown) {
        toastError(e instanceof Error ? e.message : String(e))
        emit('update:visible', false)
        // Keep loading=true: the dialog is closing and its (empty) form must stay unsavable
        return
      }
      loading.value = false
    }
    // Keyboard entry starts at the first field, once any loading mask is gone
    await nextTick()
    summaryInput.value?.focus()
  },
)

async function save() {
  if (loading.value) return
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
        create_missing_tags: true,
      })
      toastSuccess(t('editor.updated'))
    } else {
      await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        tags: form.value.tags,
        create_missing_tags: true,
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
