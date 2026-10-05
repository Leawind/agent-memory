<template>
  <el-dialog
    :model-value="visible"
    :width="width ?? '780px'"
    align-center
    append-to-body
    :close-on-click-modal="!saving"
    :close-on-press-escape="!saving"
    :before-close="blockWhileSaving"
    class="am-form-dialog am-form-dialog--floating"
    @update:model-value="emit('update:visible', $event)"
  >
    <!-- Title row per the workspace spec: 标签 #<标识> + delete + save; save stays disabled
         until something changed. Tags carry no server-visible id (the name is the only
         external identifier), so the title anchors on the name. -->
    <template #header>
      <div class="am-dialog-head">
        <h3 class="am-dialog-title">
          {{ isCreate ? t('tags.createTitle') : t('tags.titleFmt', { name: original.name }) }}
        </h3>
        <div class="am-dialog-actions">
          <!-- Reserved tags can be neither renamed nor deleted server-side; hide the dead control -->
          <el-tooltip v-if="!isCreate && !reserved" :content="t('common.delete')" placement="top" :enterable="false">
            <el-button
              type="danger"
              circle
              :icon="Delete"
              :disabled="saving"
              :aria-label="t('common.delete')"
              @click="openDelete"
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

    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('tags.nameLabel')">
        <!-- Prefill the current name when editing: small tweaks are direct edits, no need to
             retype the full name; the server treats an unchanged-name submit as a no-rename -->
        <el-input
          v-model="form.name"
          maxlength="100"
          show-word-limit
          :disabled="reserved"
          :placeholder="t('tags.namePlaceholder')"
        />
      </el-form-item>
      <el-form-item :label="t('tags.descLabel')">
        <el-input
          v-model="form.description"
          type="textarea"
          :rows="6"
          maxlength="512"
          show-word-limit
          :placeholder="t('tags.descPlaceholder')"
        />
      </el-form-item>
    </el-form>

    <!-- Delete-mode choice: the rich structure (tag name / count) can't be interpolated as one
         sentence, so it's split into three fixed-order segments -->
    <FormDialog
      v-model:visible="deleteVisible"
      :title="t('tags.deleteTitle')"
      :saving="deleting"
      submit-type="danger"
      :submit-text="t('common.delete')"
      append-to-body
      @submit="doDelete"
    >
      <p class="delete-body">
        {{ t('tags.deleteBefore') }}
        <el-tag>{{ original.name }}</el-tag>
        {{ t('tags.deleteMiddle') }}
        <b>{{ original.count }}</b>
        {{ t('tags.deleteAfter') }}
      </p>
      <el-radio-group v-model="deleteMode">
        <el-radio value="detach">{{ t('tags.detach') }}</el-radio>
        <el-radio value="purge">{{ t('tags.purge') }}</el-radio>
      </el-radio-group>
    </FormDialog>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { Delete } from '@element-plus/icons-vue'
import { ElMessageBox } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createTag, deleteTag, updateTag } from '../api/tags'
import type { TagDeleteMode } from '../api/tags'
import FormDialog from './FormDialog.vue'
import { t } from '../i18n'
import { useDialogEdgeResize } from '../composables/useDialogEdgeResize'
import type { TagView } from '../types'

const props = defineProps<{
  visible: boolean
  /** The tag being edited; null means creating new */
  tag: TagView | null
  /** Initial dialog width; defaults to 520px */
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

const isCreate = computed(() => props.tag === null)
const reserved = computed(() => props.tag?.reserved === true)

const saving = ref(false)
const form = reactive({ name: '', description: '' })
// Snapshot of the opened tag: identity for the title row, baseline for the dirty check
const original = reactive({ name: '', description: '', count: 0 })

const dirty = computed(() => form.name.trim() !== original.name || form.description !== original.description)
const canSave = computed(() => !saving.value && form.name.trim() !== '' && dirty.value)

watch(
  () => props.visible,
  (open) => {
    if (!open) return
    original.name = props.tag?.name ?? ''
    original.description = props.tag?.description ?? ''
    original.count = props.tag?.count ?? 0
    form.name = props.tag?.name ?? ''
    form.description = props.tag?.description ?? ''
  },
)

async function save() {
  if (!canSave.value) return
  saving.value = true
  try {
    if (isCreate.value) {
      await createTag(client, form.name.trim(), form.description)
      toastSuccess(t('tags.created'))
    } else {
      // When the name equals the original (or is blank) the api layer omits new_name,
      // i.e. only the description is updated
      await updateTag(client, original.name, form.name, form.description)
      toastSuccess(t('tags.saved'))
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

// ---- Delete: mode choice first (detach vs purge), an extra confirm on the destructive one ----
const deleteVisible = ref(false)
const deleting = ref(false)
const deleteMode = ref<TagDeleteMode>('detach')

function openDelete() {
  deleteMode.value = 'detach'
  deleteVisible.value = true
}

async function doDelete() {
  if (deleteMode.value === 'purge') {
    try {
      await ElMessageBox.confirm(
        t('tags.purgeConfirm', { name: original.name, count: original.count }),
        t('tags.purgeConfirmTitle'),
        { type: 'error', confirmButtonText: t('tags.purgeButton') },
      )
    } catch {
      return
    }
  }
  deleting.value = true
  try {
    await deleteTag(client, original.name, deleteMode.value)
    toastSuccess(t('tags.deleted'))
    deleteVisible.value = false
    emit('update:visible', false)
    emit('deleted')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    deleting.value = false
  }
}

// before-close fires for the X button (overlay click / ESC are already off while saving)
function blockWhileSaving(done: () => void): void {
  if (!saving.value) done()
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
