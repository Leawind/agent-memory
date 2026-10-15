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
         until a summary is present — content is optional, a summary-only memory is valid).
         The edge strips flanking the dialog resize its width, the dragged edge tracking the
         pointer. -->
    <template #header>
      <div class="am-dialog-head">
        <h3 class="am-dialog-title">
          {{ memoryId ? t('editor.titleFmt', { id: memoryId }) : t('editor.createTitle') }}
        </h3>
        <div class="am-dialog-actions">
          <el-tooltip v-if="memoryId && lifecycle" :content="t('lifecycle.title')" placement="top" :enterable="false">
            <el-button
              circle
              :icon="Clock"
              :disabled="saving"
              :aria-label="t('lifecycle.title')"
              @click="lifecycleVisible = true"
            />
          </el-tooltip>
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
          <el-tooltip v-if="memoryId" :content="t('editor.mergeAnotherTip')" placement="top" :enterable="false">
            <el-button
              circle
              :icon="Connection"
              :disabled="saving"
              :aria-label="t('editor.mergeAnotherTip')"
              @click="askMerge"
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
          <!-- Keyed tags (name carries ':') are managed through the attributes editor below, never picked as plain tags -->
          <el-option v-for="tag in plainTagOptions" :key="tag" :label="tag" :value="tag" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <template #label>
          <div class="content-label">
            <span>{{ t('editor.attributesLabel') }}</span>
          </div>
        </template>
        <div class="attr-rows">
          <div v-for="(row, index) in attrRows" :key="index" class="attr-row">
            <el-select
              v-model="row.key"
              filterable
              allow-create
              default-first-option
              :placeholder="t('editor.attrKeyPlaceholder')"
              class="attr-key"
              :aria-label="t('editor.attrKeyPlaceholder')"
            >
              <el-option v-for="k in keyRows" :key="k.name" :label="k.name" :value="k.name">
                <span>{{ k.name }}</span>
                <span class="attr-option-type">{{ t(`editor.attrType.${k.type}`) }}</span>
              </el-option>
            </el-select>
            <el-input
              v-model="row.values"
              :placeholder="attrValuePlaceholder(row)"
              class="attr-value"
              :aria-label="t('editor.attrValueLabel')"
            />
            <el-button
              circle
              :icon="Delete"
              :aria-label="t('editor.attrRemove')"
              :disabled="saving"
              @click="attrRows.splice(index, 1)"
            />
            <div v-if="attrHint(row)" class="attr-hint" :class="{ 'is-error': attrErrorFor(row) }">
              {{ attrHint(row) }}
            </div>
          </div>
          <div class="attr-footer">
            <el-button :disabled="saving" @click="addAttrRow">{{ t('editor.attrAdd') }}</el-button>
            <span v-if="attrFormError" class="attr-hint is-error">{{ attrFormError }}</span>
          </div>
        </div>
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
    <MemoryLifecycleDialog
      v-if="memoryId && lifecycle"
      v-model:visible="lifecycleVisible"
      :memory-id="memoryId"
      :metadata="lifecycle"
      @saved="onLifecycleSaved"
    />
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { InputInstance } from 'element-plus'
import { ElMessageBox } from 'element-plus'
import { Clock, Connection, Delete } from '@element-plus/icons-vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createMemory, deleteMemory, getMemory, mergeMemory, updateMemory } from '../api/memories'
import { listKeys } from '../api/keys'
import type { KeyView, LifecycleMetadata, MemoryAttributes, MemoryCreateResp } from '../types'
import type { MemoryDraft } from '../composables/useMemories'
import MarkdownView from './MarkdownView.vue'
import MemoryLifecycleDialog from './MemoryLifecycleDialog.vue'
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
const form = ref<MemoryDraft>({ id: null, summary: '', content: '', tags: [], attributes: {} })
// Snapshot of the loaded memory: the save button stays disabled until the form differs
const original = ref<{ summary: string; content: string; tags: string[]; attributes: MemoryAttributes }>({
  summary: '',
  content: '',
  tags: [],
  attributes: {},
})
// Attribute keys registered server-side (the schema contract); the fetch failing is non-fatal —
// unregistered keys still work via allow-create and lazy registration
const keyRows = ref<KeyView[]>([])
// Editing shape of the attributes: one row per key, values as display text (multi keys are
// comma-separated — the identifier/number charset contains no comma, so the split is unambiguous)
interface AttrRow {
  key: string
  values: string
}
const attrRows = ref<AttrRow[]>([])

// Keyed tags never belong in the plain-tag picker (the server rejects colon names there)
const plainTagOptions = computed(() => props.tagOptions.filter((name) => !name.includes(':')))

const lifecycle = ref<LifecycleMetadata | null>(null)
const lifecycleVisible = ref(false)

async function onLifecycleSaved(): Promise<void> {
  if (!props.memoryId) return
  try {
    lifecycle.value = (await getMemory(client, props.memoryId)).lifecycle
    emit('saved')
  } catch (error) {
    toastError(error instanceof Error ? error.message : String(error))
  }
}

function addAttrRow(): void {
  attrRows.value.push({ key: '', values: '' })
}

function keyOf(name: string): KeyView | undefined {
  return keyRows.value.find((k) => k.name === name)
}

function attrValuePlaceholder(row: AttrRow): string {
  const k = keyOf(row.key)
  if (k?.multi) return t('editor.attrMultiPlaceholder')
  if (k?.type === 'number') return t('editor.attrNumberPlaceholder')
  return t('editor.attrValuePlaceholder')
}

function attrErrorFor(row: AttrRow): string {
  const k = keyOf(row.key)
  const tokens = row.values
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s !== '')
  if (k && !k.multi && tokens.length > 1) return t('editor.attrSingleError')
  if (k?.type === 'number' && tokens.some((s) => !Number.isFinite(Number(s)))) return t('editor.attrNumberError')
  return ''
}

function attrHint(row: AttrRow): string {
  const error = attrErrorFor(row)
  if (error) return error
  const k = keyOf(row.key)
  if (!k && row.key.trim()) return t('editor.attrLazyHint')
  if (k?.multi && row.values.trim()) return t('editor.attrMultiHint')
  return ''
}

// Cross-row problems (duplicate keys, a key missing on a filled row, a filled key without values)
const attrFormError = computed(() => {
  const keys = attrRows.value.map((r) => r.key.trim()).filter((k) => k !== '')
  if (attrRows.value.some((r) => r.key.trim() === '' && r.values.trim() !== '')) return t('editor.attrKeyRequired')
  if (attrRows.value.some((r) => r.key.trim() !== '' && r.values.trim() === '')) return t('editor.attrValueRequired')
  const dup = keys.find((k, i) => keys.indexOf(k) !== i)
  return dup ? t('editor.attrDuplicateFmt', { key: dup }) : ''
})

/** Rows → the wire shape: values parsed per the key's registered type (numbers as JSON numbers,
 *  unknown keys inferred client-side from the token shape, mirroring the server's lazy inference).
 *  Reachable only when the attrErrorFor / attrFormError gates pass (canSave enforces them). */
function serializeAttributes(): MemoryAttributes {
  const out: MemoryAttributes = {}
  for (const row of attrRows.value) {
    const key = row.key.trim()
    const tokens = row.values
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s !== '')
    if (!key || tokens.length === 0) continue
    const k = keyOf(key)
    const values: Array<string | number> = tokens.map((s) => {
      const numeric = s !== '' && Number.isFinite(Number(s))
      if (k?.type === 'number' || (!k && numeric)) return Number(s)
      return s
    })
    out[key] = k?.multi || (!k && values.length > 1) ? values : values[0]
  }
  return out
}

function rowsFromAttributes(attributes: MemoryAttributes): AttrRow[] {
  return Object.entries(attributes).map(([key, value]) => ({
    key,
    values: Array.isArray(value) ? value.map(String).join(', ') : String(value),
  }))
}

function stableStringify(attributes: MemoryAttributes): string {
  const keys = Object.keys(attributes).sort()
  return JSON.stringify(keys.map((k) => [k, attributes[k]]))
}

const attributesDirty = computed(
  () => stableStringify(serializeAttributes() ?? {}) !== stableStringify(original.value.attributes),
)

const dirty = computed(() => {
  if (!props.memoryId) return false
  const f = form.value
  const o = original.value
  if (f.summary !== o.summary || f.content !== o.content) return true
  if (f.tags.length !== o.tags.length) return true
  const before = new Set(o.tags)
  if (f.tags.some((tag) => !before.has(tag))) return true
  return attributesDirty.value
})

const canSave = computed(() => {
  if (loading.value) return false
  if (attrFormError.value) return false
  if (attrRows.value.some((row) => attrErrorFor(row))) return false
  // Creating needs a summary only: content is optional (summary-only memories are valid)
  if (!props.memoryId) return form.value.summary.trim() !== ''
  return dirty.value
})

watch(
  () => props.visible,
  async (open) => {
    if (!open) return
    // Editing an existing memory opens rendered; creating starts in the source editor
    contentTab.value = props.memoryId ? 'preview' : 'edit'
    // Reset optimistically so a slow fetch never shows the previously edited memory's data
    form.value = { id: null, summary: '', content: '', tags: [], attributes: {} }
    original.value = { summary: '', content: '', tags: [], attributes: {} }
    attrRows.value = []
    keyRows.value = []
    lifecycle.value = null
    lifecycleVisible.value = false
    // The key registry is the schema contract for the attributes editor (types + multi flags)
    listKeys(client)
      .then((resp) => {
        keyRows.value = resp.keys ?? []
      })
      .catch(() => {})
    if (props.memoryId) {
      loading.value = true
      try {
        const full = await getMemory(client, props.memoryId)
        const attributes = full.attributes ?? {}
        // Keyed tags (name carries ':') never enter the plain-tag form: they are owned by the
        // attributes rows below, whose whole-set replacement is the only thing that may change them
        const plainTags = full.tags.filter((tag) => !tag.includes(':'))
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: plainTags, attributes }
        original.value = { summary: full.summary, content: full.content, tags: plainTags, attributes }
        attrRows.value = rowsFromAttributes(attributes)
        lifecycle.value = full.lifecycle
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
  const attributes = serializeAttributes()
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
        // The keyed set is replaced whole: omit when untouched, {} clears every key
        ...(attributesDirty.value ? { attributes } : {}),
        create_missing_tags: true,
      })
      toastSuccess(t('editor.updated'))
    } else {
      const resp = await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        tags: form.value.tags,
        ...(Object.keys(attributes).length > 0 ? { attributes } : {}),
        create_missing_tags: true,
      })
      toastSuccess(t('editor.created'))
      // The server's dedup hints (exact + semantic near-duplicates) are actionable here: offer
      // the merge right where the duplicate was born. Closing the dialog first — the confirm
      // box is append-to-body and outlives it. The saved refresh fires only after the merge
      // decision, so the list reflects the final state (merged away or kept).
      emit('update:visible', false)
      await offerMerge(resp)
      emit('saved')
      return
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

/** Exact (duplicate_of) and semantic (similar_to) duplicates the create surfaced: offer a
 * one-click merge into the best match — the just-created memory becomes the absorbed source,
 * so the duplicate it was modelled on keeps its id and creation time. */
async function offerMerge(resp: MemoryCreateResp): Promise<void> {
  const dupes = resp.duplicate_of ?? []
  const similar = resp.similar_to ?? []
  if (dupes.length === 0 && similar.length === 0) return
  const target = dupes[0] ?? similar[0].id
  const list = [
    ...dupes.map((id) => `${id} (${t('editor.mergeExact')})`),
    ...similar.map((s) => `${s.id} (${t('editor.mergeSimilarFmt', { sim: s.similarity })})`),
  ].join(', ')
  try {
    await ElMessageBox.confirm(t('editor.mergeConfirmFmt', { list }), t('editor.mergeConfirmTitle'), {
      confirmButtonText: t('editor.mergeInto'),
      cancelButtonText: t('editor.mergeKeepBoth'),
      type: 'warning',
    })
  } catch {
    return // keep both
  }
  try {
    await mergeMemory(client, target, resp.id)
    toastSuccess(t('editor.mergedToast', { source: resp.id, target }))
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  }
}

/** Manual merge entry (MCP memory_merge parity): absorb another memory into the one being
 * edited. The source is picked by id — hint lists and the list panel are where ids come from. */
async function askMerge() {
  if (!props.memoryId || saving.value) return
  let source: string
  try {
    const res = await ElMessageBox.prompt(
      t('editor.mergePromptMessage', { id: props.memoryId }),
      t('editor.mergePromptTitle'),
      {
        inputPattern: /^m\d+$/,
        inputErrorMessage: t('editor.mergePromptError'),
        inputPlaceholder: t('editor.mergePromptPlaceholder'),
        confirmButtonText: t('editor.mergeInto'),
        cancelButtonText: t('common.cancel'),
        type: 'warning',
      },
    )
    source = res.value.trim()
  } catch {
    return
  }
  if (source === props.memoryId) {
    toastError(t('editor.mergeSelfError'))
    return
  }
  saving.value = true
  try {
    await mergeMemory(client, props.memoryId, source)
    toastSuccess(t('editor.mergedToast', { source, target: props.memoryId }))
    emit('update:visible', false)
    // The panel treats saved and deleted alike (reload + tags): the source vanished from it
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
/* Attribute rows: key picker + value input + remove on one line, per-row hint below */
.attr-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}
.attr-row {
  display: grid;
  grid-template-columns: minmax(140px, 220px) 1fr auto;
  gap: 8px;
  align-items: center;
}
.attr-hint {
  grid-column: 2 / 4;
  font-size: 12px;
  line-height: 1.4;
  color: var(--el-text-color-secondary);
}
.attr-hint.is-error {
  color: var(--el-color-danger);
}
.attr-footer {
  display: flex;
  align-items: center;
  gap: 12px;
}
.attr-option-type {
  float: right;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
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
