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
      <el-form-item>
        <template #label>
          <div class="content-label">
            <span>{{ t('editor.tagsLabel') }}</span>
          </div>
        </template>
        <!-- Format guidance above the input: the dropdown opens downward, so a hint below
             would sit exactly under the open panel -->
        <div class="form-hint">{{ t('editor.tagsHint') }}</div>
        <!-- The create flow is taken over from allow-create (it has no pre-create hook): the
             query tracked by filter-method drives a controlled first dropdown entry — the "add
             token" action when the token can land, a disabled red refusal otherwise. Illegal
             tokens never become chips, and the explanation sits in the open dropdown where the
             user is already looking. -->
        <el-select
          v-model="form.tags"
          multiple
          filterable
          default-first-option
          :filter-method="onTagQuery"
          :placeholder="t('editor.tagsPlaceholder')"
          class="tags-select"
          @visible-change="onTagsDropdown"
        >
          <el-option
            v-if="tagQueryEntry"
            :key="`query:${tagQueryEntry.token}`"
            :label="tagQueryEntry.token"
            :value="tagQueryEntry.token"
            :disabled="tagQueryEntry.problem !== ''"
          >
            <span :style="tagQueryEntry.problem !== '' ? { color: 'var(--el-color-danger)' } : undefined">
              {{ tagQueryEntry.label }}
            </span>
          </el-option>
          <!-- One tag vocabulary: plain names and value-bearing key:value tokens live side by
               side; the save maps the chips onto the wire's tags object -->
          <el-option v-for="tag in filteredTagOptions" :key="tag" :label="tag" :value="tag" />
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
import { validTagName, validTagValue } from '../tagNames'
import type { LifecycleMetadata, MemoryCreateResp } from '../types'
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
const form = ref<MemoryDraft>({ id: null, summary: '', content: '', tags: [] })
// Snapshot of the loaded memory: the save button stays disabled until the form differs
const original = ref({ summary: '', content: '', tags: [] as string[] })

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

// One tag vocabulary: a token with a colon is a value-bearing pair `key:value` (a key is unique
// per memory — one value each), a plain token a bare tag. Chips are the editing form; the save
// maps them onto the wire's tags object.
function keyedToken(tag: string): Array<string> | null {
  const i = tag.indexOf(':')
  if (i <= 0) return null
  return [tag.slice(0, i), tag.slice(i + 1)]
}

/** Chip set → per-key value text (null for bare tags), the diff basis of an edit. */
function tagMap(tags: Iterable<string>): Map<string, string | null> {
  const map = new Map<string, string | null>()
  for (const tag of tags) {
    const pair = keyedToken(tag)
    if (pair) map.set(pair[0], pair[1])
    else map.set(tag, null)
  }
  return map
}

/** Wire shape of one value: numbers stay JSON numbers so the server stores their canonical form. */
function tagWireValue(value: string | null): string | number | null {
  if (value === null) return null
  return value !== '' && Number.isFinite(Number(value)) ? Number(value) : value
}

const stableTagMap = (tags: Iterable<string>) =>
  JSON.stringify([...tagMap(tags).entries()].sort(([a], [b]) => a.localeCompare(b)))

// ---- Tag entry: the typed query drives a controlled dropdown entry ----
const tagQuery = ref('')

function onTagQuery(q: string): void {
  tagQuery.value = q
}

function onTagsDropdown(open: boolean): void {
  if (!open) tagQuery.value = ''
}

watch(
  () => form.value.tags.length,
  () => {
    tagQuery.value = ''
  },
)

const filteredTagOptions = computed(() => {
  const q = tagQuery.value.trim().toLowerCase()
  return q ? props.tagOptions.filter((tag) => tag.toLowerCase().includes(q)) : props.tagOptions
})

/** Why the typed token cannot land: malformed `key:value`, an illegal name or value half, or a
 * duplicate (a key already valued, or a bare tag already present — re-adding would toggle it
 * off). An empty problem means the token may be added. */
function judgeTagToken(token: string): { problem: string; detail: string } {
  if (token.includes(':')) {
    const pair = keyedToken(token)
    if (pair === null || pair[0].trim() === '' || pair[1].trim() === '') return { problem: 'token', detail: token }
    if (!validTagName(pair[0])) return { problem: 'name', detail: pair[0].trim() }
    if (!validTagValue(pair[1])) return { problem: 'value', detail: pair[1].trim() }
    if (tagMap(form.value.tags).has(pair[0].trim())) return { problem: 'keydup', detail: pair[0].trim() }
    return { problem: '', detail: '' }
  }
  if (!validTagName(token)) return { problem: 'name', detail: token.trim() }
  if (form.value.tags.includes(token)) return { problem: 'tagdup', detail: token.trim() }
  return { problem: '', detail: '' }
}

const tagQueryEntry = computed(() => {
  const token = tagQuery.value.trim()
  // A token that already exists in the vocabulary needs no add entry — the real option serves
  if (!token || props.tagOptions.includes(token)) return null
  const { problem, detail } = judgeTagToken(token)
  const label =
    problem === 'token'
      ? t('editor.tagTokenError')
      : problem === 'name'
        ? t('tags.invalidNameFmt', { name: detail })
        : problem === 'value'
          ? t('editor.tagValueErrorFmt', { value: detail })
          : problem === 'keydup'
            ? t('editor.tagKeyExistsFmt', { key: detail })
            : problem === 'tagdup'
              ? t('editor.tagExistsFmt', { name: detail })
              : t('editor.tagAddFmt', { token })
  return { token, label, problem }
})

const tagsDirty = computed(() => stableTagMap(form.value.tags) !== stableTagMap(original.value.tags))

const dirty = computed(() => {
  if (!props.memoryId) return false
  const f = form.value
  const o = original.value
  if (f.summary !== o.summary || f.content !== o.content) return true
  return tagsDirty.value
})

const canSave = computed(() => {
  if (loading.value) return false
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
    form.value = { id: null, summary: '', content: '', tags: [] }
    original.value = { summary: '', content: '', tags: [] }
    lifecycle.value = null
    lifecycleVisible.value = false
    if (props.memoryId) {
      loading.value = true
      try {
        const full = await getMemory(client, props.memoryId)
        // All tags live in one vocabulary: plain names and key:value pairs are chips alike
        form.value = { id: full.id, summary: full.summary, content: full.content, tags: [...full.tags] }
        original.value = { summary: full.summary, content: full.content, tags: [...full.tags] }
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
  saving.value = true
  try {
    if (form.value.id) {
      // Editing: a per-key diff — added keys and value changes upsert through add_tags, missing
      // keys drop through remove_tags by name. Untouched tags never travel.
      const before = tagMap(original.value.tags)
      const after = tagMap(form.value.tags)
      const add: Record<string, string | number | null> = {}
      for (const [key, value] of after) {
        if (before.get(key) !== value) add[key] = tagWireValue(value)
      }
      const remove: string[] = [...before.keys()].filter((key) => !after.has(key))
      await updateMemory(client, form.value.id, {
        summary: form.value.summary,
        content: form.value.content,
        ...(Object.keys(add).length > 0 ? { add_tags: add } : {}),
        ...(remove.length > 0 ? { remove_tags: remove } : {}),
        create_missing_tags: true,
      })
      toastSuccess(t('editor.updated'))
    } else {
      const tags: Record<string, string | number | null> = {}
      for (const [key, value] of tagMap(form.value.tags)) tags[key] = tagWireValue(value)
      const resp = await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        tags,
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
.form-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  width: 100%;
  margin: 0 0 4px;
}
.form-hint.is-error {
  color: var(--el-color-danger);
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
