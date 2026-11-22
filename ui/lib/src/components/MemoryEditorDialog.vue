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
          <el-tooltip v-if="memoryId" :content="t('common.delete')" placement="top" :enterable="false">
            <el-button
              type="danger"
              circle
              plain
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
        <!-- The create flow is taken over from allow-create (it has no pre-create hook): the
             query tracked by filter-method drives a controlled first dropdown entry — the "add
             token" action when the token can land, a disabled red refusal otherwise. Illegal
             tokens never become chips, and the explanation sits in the open dropdown where the
             user is already looking. reserve-keyword=false: the add entry is a regular option,
             not an allow-create one, so the default keyword reservation would leave the landed
             token in the input where the next keystroke concatenates onto it. -->
        <el-select
          v-model="form.tags"
          multiple
          filterable
          default-first-option
          :reserve-keyword="false"
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
          <el-option v-for="tag in filteredTagOptions" :key="tag" :label="tag" :value="tag" />
        </el-select>
      </el-form-item>
      <el-form-item v-if="scopeRows.length > 0">
        <template #label>
          <div class="content-label">
            <span>{{ t('editor.scopeValuesLabel') }}</span>
          </div>
        </template>
        <!-- One row per scope: at most one value each. A required scope is declared on every
             write — its select offers the explicit no-value entry and cannot be cleared, only
             switched; an optional scope clears to no declaration at all. -->
        <div class="scope-rows">
          <div v-for="scope in scopeRows" :key="scope.name" class="scope-row">
            <span class="scope-name">{{ scope.name }}</span>
            <el-tooltip v-if="scope.required" :content="t('scopes.requiredHint')" placement="top" :enterable="false">
              <el-tag size="small" type="info" class="scope-required">{{ t('scopes.required') }}</el-tag>
            </el-tooltip>
            <el-select
              :model-value="attrModel(scope.name)"
              filterable
              :clearable="!scope.required"
              :placeholder="t('editor.scopePlaceholder')"
              class="scope-select"
              @update:model-value="setAttr(scope.name, $event)"
            >
              <el-option v-if="scope.required" :value="NO_VALUE" :label="t('editor.noScopeValue')" />
              <el-option v-for="value in scope.values" :key="value" :label="value" :value="value" />
            </el-select>
          </div>
        </div>
        <!-- A required scope is a declaration, not a label: the create must state each one, and
             the names still missing are called out here -->
        <div v-if="!memoryId && missingRequired.length > 0" class="required-hint" role="status">
          {{ t('editor.requiredMissingFmt', { scopes: missingRequired.join(', ') }) }}
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
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { InputInstance } from 'element-plus'
import { ElMessageBox } from 'element-plus'
import { Connection, Delete } from '@element-plus/icons-vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createMemory, deleteMemory, getMemory, mergeMemory, updateMemory } from '../api/memories'
import { validTagName, parseScopedName } from '../tagNames'
import type { MemoryCreateResp, MemoryFull } from '../types'
import type { MemoryDraft, ScopeColumn } from '../composables/useMemories'
import MarkdownView from './MarkdownView.vue'
import MarkdownModeToggle from './MarkdownModeToggle.vue'
import { useDialogEdgeResize } from '../composables/useDialogEdgeResize'
import { t } from '../i18n'

const props = defineProps<{
  visible: boolean
  /** The memory id being edited; null means creating new */
  memoryId: string | null
  /** Bare tag names offered as chips (scoped names belong to the scope rows below) */
  tagOptions: string[]
  /** Scope columns: the required write contract and each scope's known values */
  scopeOptions?: ScopeColumn[]
  /** Scopes the store requires on every write: a new memory must declare each one (value or none) */
  requiredScopes?: string[]
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
const original = ref({
  summary: '',
  content: '',
  tags: [] as string[],
  attributes: {} as Record<string, string | null>,
})

// ---- Scope rows: the known columns, plus any scope the loaded memory already declares ----

/** Sentinel select value for the explicit "takes no tag in this scope" declaration — a control
 *  value that can never collide with a tag name. */
const NO_VALUE = '\u0000'

const scopeRows = computed<ScopeColumn[]>(() => {
  const columns = props.scopeOptions ?? []
  const known = new Set(columns.map((c) => c.name))
  const extras = Object.keys(form.value.attributes)
    .filter((name) => !known.has(name))
    .map((name) => ({ name, required: (props.requiredScopes ?? []).includes(name), values: [] }))
  return [...columns, ...extras]
})

/** The select's model: '' = no declaration, the sentinel = the null declaration, else the value */
function attrModel(scope: string): string {
  const value = form.value.attributes[scope]
  return value === undefined ? '' : value === null ? NO_VALUE : value
}

function setAttr(scope: string, value: string): void {
  const attributes = { ...form.value.attributes }
  if (value === '') delete attributes[scope]
  else attributes[scope] = value === NO_VALUE ? null : value
  form.value = { ...form.value, attributes }
}

/** Scope declarations of a loaded memory: values from the scoped names among its tags, nulls
 *  from an attributes view that carries them. */
function attributesOf(full: MemoryFull): Record<string, string | null> {
  const out: Record<string, string | null> = {}
  for (const name of full.tags) {
    const scoped = parseScopedName(name)
    if (scoped) out[scoped.scope] = scoped.value
  }
  for (const [scope, value] of Object.entries(full.attributes ?? {})) {
    if (value === null) out[scope] = null
  }
  return out
}

// ---- Bare tag entry: the typed query drives a controlled dropdown entry ----
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

/** Why the typed token cannot land: a scoped name (those belong to the scope rows), an illegal
 *  name, or a duplicate (re-adding would toggle it off). An empty problem means the token may
 *  be added. */
function judgeTagToken(token: string): { problem: string; detail: string } {
  if (token.includes('::')) return { problem: 'scoped', detail: token }
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
    problem === 'scoped'
      ? t('editor.tagScopedError')
      : problem === 'name'
        ? t('tags.invalidNameFmt', { name: detail })
        : problem === 'tagdup'
          ? t('editor.tagExistsFmt', { name: detail })
          : t('editor.tagAddFmt', { token })
  return { token, label, problem }
})

// ---- Dirty tracking over both containers ----

const stableTags = (tags: string[]) => JSON.stringify([...tags].sort())
const stableAttrs = (attrs: Record<string, string | null>) =>
  JSON.stringify(Object.entries(attrs).sort(([a], [b]) => a.localeCompare(b)))

const dirty = computed(() => {
  if (!props.memoryId) return false
  const f = form.value
  const o = original.value
  if (f.summary !== o.summary || f.content !== o.content) return true
  if (stableTags(f.tags) !== stableTags(o.tags)) return true
  return stableAttrs(f.attributes) !== stableAttrs(o.attributes)
})

/** Required scopes this draft does not declare yet: every one of them needs a value (or the
 *  explicit none) on a create. */
const missingRequired = computed(() => (props.requiredScopes ?? []).filter((name) => !(name in form.value.attributes)))

const canSave = computed(() => {
  if (loading.value) return false
  // Creating needs a summary, and a declaration for every required scope
  if (!props.memoryId) return form.value.summary.trim() !== '' && missingRequired.value.length === 0
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
    if (props.memoryId) {
      loading.value = true
      try {
        const full = await getMemory(client, props.memoryId)
        // The two containers: bare names are chips, scoped names land in their scope rows
        const attributes = attributesOf(full)
        form.value = {
          id: full.id,
          summary: full.summary,
          content: full.content,
          tags: full.tags.filter((name) => !parseScopedName(name)),
          attributes,
        }
        original.value = {
          summary: full.summary,
          content: full.content,
          tags: [...form.value.tags],
          attributes: { ...attributes },
        }
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
      // Editing: an incremental diff over the two containers — a new bare tag travels in tags, a
      // changed scope value in attributes, a dropped tag through remove_tags by full name.
      // Untouched tags never travel.
      const before = original.value
      const after = form.value
      const addedBare = after.tags.filter((tag) => !before.tags.includes(tag))
      const removes = before.tags.filter((tag) => !after.tags.includes(tag))
      const changed: Record<string, string | null> = {}
      for (const [scope, value] of Object.entries(after.attributes)) {
        if (before.attributes[scope] === value) continue
        changed[scope] = value
      }
      for (const [scope, prev] of Object.entries(before.attributes)) {
        if (scope in after.attributes) continue
        // Dropping a required scope's value is the explicit "this memory takes no tag in it"
        // declaration (a removal would leave the memory undeclared and is refused); other
        // dropped scopes are plain removals by full tag name.
        if ((props.requiredScopes ?? []).includes(scope)) changed[scope] = null
        else if (prev !== null) removes.push(`${scope}::${prev}`)
      }
      await updateMemory(client, form.value.id, {
        summary: form.value.summary,
        content: form.value.content,
        ...(addedBare.length > 0 ? { tags: addedBare } : {}),
        ...(Object.keys(changed).length > 0 ? { attributes: changed } : {}),
        ...(removes.length > 0 ? { remove_tags: removes } : {}),
        create_missing_tags: true,
      })
      toastSuccess(t('editor.updated'))
    } else {
      const resp = await createMemory(client, {
        summary: form.value.summary,
        content: form.value.content,
        ...(form.value.tags.length > 0 ? { tags: [...form.value.tags] } : {}),
        ...(Object.keys(form.value.attributes).length > 0 ? { attributes: { ...form.value.attributes } } : {}),
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
.scope-rows {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
}
.scope-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.scope-name {
  flex-shrink: 0;
  min-width: 64px;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.scope-required {
  flex-shrink: 0;
}
.scope-select {
  flex: 1;
  min-width: 0;
}
.required-hint {
  margin-top: 4px;
  font-size: 12px;
  color: var(--el-color-danger);
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
