<template>
  <el-dialog
    :model-value="visible"
    :width="width"
    align-center
    append-to-body
    :close-on-click-modal="!saving"
    :close-on-press-escape="!saving"
    :before-close="blockWhileSaving"
    class="am-form-dialog am-form-dialog--floating"
    @update:model-value="emit('update:visible', $event)"
  >
    <!-- Title row per the workspace spec: 记忆 #<id> + delete + save; the save button stays
         disabled until the form actually differs from the loaded memory (or, when creating,
         until summary and content are both present). The edge strips flanking the dialog
         resize its width, the dragged edge tracking the pointer. -->
    <template #header>
      <div class="am-dialog-head">
        <h3 class="am-dialog-title">
          {{ memoryId ? t('editor.titleFmt', { id: memoryId }) : t('editor.createTitle') }}
        </h3>
        <div class="am-dialog-actions">
          <el-tooltip v-if="memoryId" :content="t('common.delete')" placement="top" :enterable="false">
            <el-button
              type="danger"
              circle
              :icon="Delete"
              :disabled="saving"
              :aria-label="t('common.delete')"
              @click="askDelete"
            />
          </el-tooltip>
          <el-button type="primary" :loading="saving" :disabled="!canSave" @click="save">
            {{ t('common.save') }}
          </el-button>
        </div>
      </div>
      <div
        class="am-edge am-edge--left"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
      <div
        class="am-edge am-edge--right"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
    </template>

    <!-- label-position=top: each field's description sits on its own line above the input
         instead of squeezing the field to the right -->
    <el-form v-loading="loading" label-position="top" @submit.prevent>
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
    </el-form>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { InputInstance } from 'element-plus'
import { ElMessageBox } from 'element-plus'
import { Delete } from '@element-plus/icons-vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createMemory, deleteMemory, getMemory, updateMemory } from '../api/memories'
import type { MemoryDraft } from '../composables/useMemories'
import MarkdownView from './MarkdownView.vue'
import MarkdownModeToggle from './MarkdownModeToggle.vue'
import { useDialogEdgeResize } from '../composables/useDialogEdgeResize'
import { t } from '../i18n'

const props = defineProps<{
  visible: boolean
  /** The memory id being edited; null means creating new */
  memoryId: string | null
  tagOptions: string[]
  /** Initial dialog width; the panel passes a narrower value in narrow containers */
  width?: string
}>()

const { onPointerDown, onPointerMove, onPointerUp } = useDialogEdgeResize()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  /** Saved successfully (create or update finished) */
  saved: []
  /** Deleted successfully (edit mode only) */
  deleted: []
}>()

const client = useApiClient()

const saving = ref(false)
const contentTab = ref<'edit' | 'preview'>('preview')
const summaryInput = ref<InputInstance | null>(null)
// Fetching the full memory on open; while true the (still empty) form must not be savable
const loading = ref(false)
// Content editor view: edit the source or preview the rendered Markdown
const form = ref<MemoryDraft>({ id: null, summary: '', content: '', tags: [] })
// Snapshot of the loaded memory: the save button stays disabled until the form differs
const original = ref({ summary: '', content: '', tags: [] as string[] })

const dirty = computed(() => {
  if (!props.memoryId) return false
  const f = form.value
  const o = original.value
  if (f.summary !== o.summary || f.content !== o.content) return true
  if (f.tags.length !== o.tags.length) return true
  const before = new Set(o.tags)
  return f.tags.some((tag) => !before.has(tag))
})

const canSave = computed(() => {
  if (loading.value) return false
  if (!props.memoryId) return form.value.summary.trim() !== '' && form.value.content.trim() !== ''
  return dirty.value
})

watch(
  () => props.visible,
  async (open) => {
    if (!open) return
    // Editing an existing memory opens rendered; creating starts in the source editor
    contentTab.value = props.memoryId ? 'preview' : 'edit'
    // Reset optimistically so a slow fetch never shows the previously edited memory's data
    form.value = { id: null, summary: '', content: '', tags: [] }
    original.value = { summary: '', content: '', tags: [] }
    if (props.memoryId) {
      loading.value = true
      try {
        const full = await getMemory(client, props.memoryId)
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: [...full.tags] }
        original.value = { summary: full.summary, content: full.content, tags: [...full.tags] }
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
  if (!canSave.value) return
  saving.value = true
  try {
    if (form.value.id) {
      // Editing: compute add_tags / remove_tags against the original tag set as baseline
      const before = new Set(original.value.tags)
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

async function askDelete() {
  if (!props.memoryId || saving.value) return
  try {
    await ElMessageBox.confirm(t('memories.deleteConfirm', { id: props.memoryId }), t('memories.deleteTitle'), {
      type: 'warning',
    })
  } catch {
    return
  }
  saving.value = true
  try {
    await deleteMemory(client, props.memoryId)
    toastSuccess(t('memories.deleted'))
    emit('update:visible', false)
    emit('deleted')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

// before-close fires for the X button (overlay click / ESC are already off while saving)
function blockWhileSaving(done: () => void): void {
  if (!saving.value) done()
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
