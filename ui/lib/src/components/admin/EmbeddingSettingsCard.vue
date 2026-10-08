<template>
  <!-- Multi-candidate card: an ordered embedding list (list order = failover priority), each entry
       independently enabled and independently tailable/backfillable. Every entry is one collapsed
       line — name plus its live verdict and vector coverage — and expands into the labeled fields
       and per-entry actions. The vector caches of all identities coexist server-side; this card is
       their per-model management surface. -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.embeddingTitle') }}</span>
        <el-tag :type="badgeType" size="small">{{ t(badgeKey) }}</el-tag>
      </div>
    </template>

    <p class="state-line" :class="{ 'is-broken': state === 'broken' }">{{ stateText }}</p>
    <!-- Failure reasons stay visible without expanding anything: the badge alone does not say which
         candidate is down and why -->
    <div v-if="failures.length" class="probe-results">
      <div v-for="f in failures" :key="f.id" class="probe-line">
        <span>{{ f.model || t('access.embeddingEntryUntitled') }}</span> ·
        <code class="broken-err">{{ f.error }}</code>
      </div>
    </div>

    <div ref="listEl" class="entry-list">
      <div
        v-for="(row, i) in draft"
        :key="row.uiKey"
        class="entry"
        :class="{ 'is-off': !row.enabled, 'is-open': row.open }"
      >
        <div class="entry-bar">
          <span class="entry-handle" :title="t('access.entryDrag')">
            <!-- GitHub-style grip: two columns of three dots, the usual drag affordance -->
            <svg class="grip" width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
              <path
                d="M10 4a1 1 0 1 1-2 0 1 1 0 0 1 2 0Zm-4 0a1 1 0 1 1-2 0 1 1 0 0 1 2 0Zm4 4a1 1 0 1 1-2 0 1 1 0 0 1 2 0Zm-4 0a1 1 0 1 1-2 0 1 1 0 0 1 2 0Zm4 4a1 1 0 1 1-2 0 1 1 0 0 1 2 0Zm-4 0a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z"
              />
            </svg>
          </span>
          <button type="button" class="entry-toggle" @click="row.open = !row.open">
            <el-icon class="entry-caret"><ArrowRight /></el-icon>
            <span class="entry-name">{{
              row.name.trim() || row.model.trim() || t('access.embeddingEntryUntitled')
            }}</span>
          </button>
          <span class="entry-tags">
            <span v-if="verdictOf(row)" class="entry-verdict" :class="verdictOf(row)!.ok ? 'is-ok' : 'is-bad'">
              <el-icon>
                <CircleCheckFilled v-if="verdictOf(row)!.ok" />
                <CircleCloseFilled v-else />
              </el-icon>
              {{
                verdictOf(row)!.ok
                  ? t('access.entryTestOk', { dim: verdictOf(row)!.dim ?? 0, ms: verdictOf(row)!.elapsed_ms ?? 0 })
                  : t('access.entryTestFail')
              }}
            </span>
            <span v-else-if="row.enabled && !complete(row)" class="entry-verdict is-warn">
              {{ t('access.embeddingEntryIncomplete') }}
            </span>
            <span v-if="cacheOf(row)" class="entry-usage">
              {{ t('access.cacheRowSummary', { embedded: cacheOf(row)!.embedded, pending: cacheOf(row)!.pending }) }}
            </span>
          </span>
          <el-switch v-model="row.enabled" class="entry-switch" :aria-label="t('access.entryEnable')" />
        </div>

        <div v-if="row.open" class="entry-detail">
          <div class="field-grid">
            <label class="field">
              <span class="field-label">{{ t('access.entryIdLabel') }}</span>
              <el-input v-model="row.modelId" class="f-id" :placeholder="t('access.entryIdPlaceholder')" />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryNameLabel') }}</span>
              <el-input v-model="row.name" class="f-name" />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryBaseUrlLabel') }}</span>
              <el-input v-model="row.baseUrl" class="f-base-url" :placeholder="t('access.entryBaseUrlPlaceholder')" />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryModelLabel') }}</span>
              <el-input v-model="row.model" class="f-model" />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryApiKeyLabel') }}</span>
              <el-input
                v-model="row.apiKey"
                class="f-api-key"
                show-password
                :placeholder="t('access.entryApiKeyPlaceholder')"
              />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryQueryPrefixLabel') }}</span>
              <el-input
                v-model="row.queryPrefix"
                class="f-query-prefix"
                :placeholder="t('access.entryQueryPrefixPlaceholder')"
              />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryPassagePrefixLabel') }}</span>
              <el-input
                v-model="row.passagePrefix"
                class="f-passage-prefix"
                :placeholder="t('access.entryPassagePrefixPlaceholder')"
              />
            </label>
            <label class="field">
              <span class="field-label">{{ t('access.entryMinSimilarityLabel') }}</span>
              <el-input
                v-model="row.minSimilarity"
                class="f-min-similarity"
                :placeholder="t('access.entryMinSimilarityPlaceholder')"
              />
            </label>
          </div>

          <div class="entry-actions">
            <el-button size="small" :loading="testingRowId === row.uiKey" @click="testRow(row)">
              {{ t('access.embeddingTest') }}
            </el-button>
            <span v-if="cacheOf(row)" class="cache-detail">
              {{ t('access.cacheRowDetail', { embedded: cacheOf(row)!.embedded, pending: cacheOf(row)!.pending }) }}
            </span>
            <span class="spacer" />
            <el-button
              v-if="canBackfill(row)"
              size="small"
              type="primary"
              plain
              :disabled="runningRowId !== null"
              @click="startBackfill(row)"
            >
              {{ t('access.cacheBackfill') }}
            </el-button>
            <el-button
              v-if="(cacheOf(row)?.embedded ?? 0) > 0"
              size="small"
              :loading="deletingKey === identityOf(row)"
              @click="deleteCacheAt(identityOf(row), row.name.trim() || row.modelId.trim(), cacheOf(row)!.embedded)"
            >
              {{ t('access.cacheDelete') }}
            </el-button>
            <el-button size="small" type="danger" plain @click="removeEntry(i)">
              {{ t('common.delete') }}
            </el-button>
          </div>

          <div v-if="row.progress" class="entry-progress">
            <el-progress class="progress-bar" :percentage="progressPct(row)" :stroke-width="8" />
            <span class="progress-text">
              {{ t('access.cacheBackfillProgress', { done: row.progress.done, total: row.progress.total }) }}
            </span>
            <el-button size="small" @click="cancelBackfill">{{ t('access.cacheBackfillStop') }}</el-button>
          </div>
        </div>
      </div>
    </div>

    <el-alert
      v-if="identityChanged && vectorCount > 0"
      :title="t('access.embeddingModelChangeWarn', { count: vectorCount })"
      type="warning"
      show-icon
      :closable="false"
      class="settings-hint"
    />
    <p class="form-hint">{{ t('access.embeddingFormHint') }}</p>
    <div class="form-actions">
      <el-button :icon="Plus" @click="addEntry">{{ t('access.embeddingAdd') }}</el-button>
      <span class="form-actions-main">
        <el-button :loading="testingAll" :disabled="!enabledRows.length" @click="testAll">
          {{ t('access.embeddingTest') }}
        </el-button>
        <el-button :disabled="!dirty" :loading="saving" type="primary" @click="save">
          {{ t('access.embeddingSave') }}
        </el-button>
      </span>
    </div>

    <!-- Caches whose identity is no longer configured: nothing can backfill them, only delete -->
    <div v-if="leftovers.length" class="cache-leftovers">
      <div class="cache-title">{{ t('access.cacheLeftoverTitle') }}</div>
      <div v-for="c in leftovers" :key="c.key" class="cache-row">
        <span class="cache-model" :title="c.key">{{ c.name || c.key }}</span>
        <span class="cache-counts">
          {{ t('access.cacheRowSummary', { embedded: c.embedded, pending: c.pending }) }}
          <el-tag size="small" type="info">{{ t('access.cacheUnconfigured') }}</el-tag>
        </span>
        <el-button
          size="small"
          :loading="deletingKey === c.key"
          @click="deleteCacheAt(c.key, c.name || c.key, c.embedded)"
        >
          {{ t('access.cacheDelete') }}
        </el-button>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ArrowRight, CircleCheckFilled, CircleCloseFilled, Plus } from '@element-plus/icons-vue'
import { ElMessageBox } from 'element-plus'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { deleteVectorCache, listVectorCaches, testEmbeddings } from '../../api/ops'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import { useDragOrder } from '../../composables/useDragOrder'
import type { EmbedModelEntry, EmbedTestResult, VectorCacheInfo } from '../../types'
import { run } from './caps'

const props = defineProps<{
  /** The ordered candidate list as stored server-side (settings GET) */
  models: EmbedModelEntry[]
}>()

const emit = defineEmits<{ changed: [] }>()

const api = useApiClient()
const { backfill, cancelBackfill } = useAdmin()

// ---- Draft rows: form fields plus UI state (expanded / verdict / progress) ----
// `uiKey` is the stable rendering key and the verdict's owner: rows are keyed by it, so reordering never
// mixes up which row a verdict or an open panel belongs to.
interface EntryDraft {
  uiKey: number
  modelId: string
  name: string
  enabled: boolean
  baseUrl: string
  model: string
  apiKey: string
  queryPrefix: string
  passagePrefix: string
  minSimilarity: string
  open: boolean
  /** The last probe verdict, together with the payload it was taken for (stale verdicts are
   *  dropped instead of being shown against values that were never tested) */
  probe: { payload: string; result: EmbedTestResult } | null
  progress: { done: number; total: number } | null
}

let nextId = 1

/** One row's canonical write shape — exactly what a settings PUT sends for that row */
function entryPayload(r: EntryDraft) {
  return {
    id: r.modelId.trim(),
    name: r.name.trim(),
    enabled: r.enabled,
    base_url: r.baseUrl.trim(),
    model: r.model.trim(),
    api_key: r.apiKey.trim() === '' ? null : r.apiKey,
    query_prefix: r.queryPrefix === '' ? null : r.queryPrefix,
    passage_prefix: r.passagePrefix === '' ? null : r.passagePrefix,
    min_similarity: r.minSimilarity.trim() === '' ? null : Number(r.minSimilarity),
  }
}

function payloadKey(r: EntryDraft): string {
  return JSON.stringify(entryPayload(r))
}

function toDraftRow(e: EmbedModelEntry): EntryDraft {
  return {
    uiKey: nextId++,
    modelId: e.id,
    name: e.name,
    enabled: e.enabled,
    baseUrl: e.base_url,
    model: e.model,
    apiKey: e.api_key ?? '',
    queryPrefix: e.query_prefix ?? '',
    passagePrefix: e.passage_prefix ?? '',
    minSimilarity: e.min_similarity == null ? '' : String(e.min_similarity),
    open: false,
    probe: null,
    progress: null,
  }
}

const draft = ref<EntryDraft[]>([])
const saved = ref<EntryDraft[]>([])

/** Every row's payload, in order: list order is part of the configuration, so it is compared too */
function payloads(rows: EntryDraft[]): string {
  return JSON.stringify(rows.map(entryPayload))
}

const dirty = computed(() => payloads(draft.value) !== payloads(saved.value))

// Keep the fields in sync while the draft is clean (a config change elsewhere or a reload must not
// be swallowed); an in-progress edit (dirty) owns the fields instead. Per-row UI state survives the
// resync: a server reload after a save must not collapse what the operator just opened.
watch(
  () => props.models,
  (models) => {
    if (dirty.value) return
    saved.value = models.map(toDraftRow)
    draft.value = models.map((e) => {
      const fresh = toDraftRow(e)
      const prev = draft.value.find((r) => r.modelId === fresh.modelId)
      // Keep row state by the persisted ID when settings are reloaded.
      if (!prev) return fresh
      return { ...fresh, uiKey: prev.uiKey, open: prev.open, probe: prev.probe }
    })
  },
  { immediate: true, deep: true },
)

// Drag reordering: the list container is handed to SortableJS, the draft array follows the drop
const listEl = ref<HTMLElement | null>(null)
useDragOrder(listEl, draft)

function addEntry(): void {
  draft.value.push({
    uiKey: nextId++,
    modelId: '',
    name: '',
    enabled: true,
    baseUrl: '',
    model: '',
    apiKey: '',
    queryPrefix: '',
    passagePrefix: '',
    minSimilarity: '',
    open: true,
    probe: null,
    progress: null,
  })
}

function removeEntry(i: number): void {
  draft.value.splice(i, 1)
}

function complete(r: EntryDraft): boolean {
  return r.modelId.trim() !== '' && r.name.trim() !== '' && r.baseUrl.trim() !== '' && r.model.trim() !== ''
}

// Cache IDs remain stable across reordering and display-name changes.
function identityOf(r: EntryDraft): string {
  return r.modelId.trim()
}

function fingerprintOf(r: EntryDraft): string {
  return JSON.stringify([r.baseUrl.trim().replace(/\/+$/, ''), r.model.trim(), r.queryPrefix, r.passagePrefix])
}

const changedCacheKeys = computed(
  () =>
    new Set(
      saved.value
        .filter((before) => {
          const after = draft.value.find((r) => identityOf(r) === identityOf(before))
          return !after || fingerprintOf(after) !== fingerprintOf(before)
        })
        .map(identityOf),
    ),
)
const identityChanged = computed(() => changedCacheKeys.value.size > 0)

// ---- Vector caches (per identity) ----
const caches = ref<VectorCacheInfo[]>([])
const deletingKey = ref<string | null>(null)
const runningRowId = ref<number | null>(null)

const cacheByKey = computed(() => new Map(caches.value.map((c) => [c.key, c])))
const vectorCount = computed(() =>
  caches.value.reduce((sum, c) => sum + (changedCacheKeys.value.has(c.key) ? c.embedded : 0), 0),
)
/** Identities no row claims anymore: nothing can backfill them, so they are listed apart and are
 *  only deletable. A row's own cache, however unconfigured it is (disabled or half-filled), is
 *  reported by that row instead of being repeated here. */
const leftovers = computed(() => {
  const claimed = new Set(draft.value.map(identityOf))
  return caches.value.filter((c) => !claimed.has(c.key))
})

function cacheOf(r: EntryDraft): VectorCacheInfo | undefined {
  return cacheByKey.value.get(identityOf(r))
}

/** Backfill drains the identity's queue through the saved entry — it must exist there and be
 *  enabled, so an uncommitted or switched-off row's button stays hidden */
function canBackfill(r: EntryDraft): boolean {
  const cache = cacheOf(r)
  if (!cache || cache.pending === 0 || !r.enabled) return false
  const savedEntry = saved.value.find((entry) => identityOf(entry) === identityOf(r))
  return !!savedEntry && savedEntry.enabled && fingerprintOf(savedEntry) === fingerprintOf(r)
}

async function refreshCaches(): Promise<void> {
  const out = await run(() => listVectorCaches(api))
  caches.value = out?.caches ?? []
}

onMounted(() => {
  void refreshCaches()
})

// Coverage and pending counts move with every write / external backfill, so they are re-read
// whenever the settings arrive fresh (the host reloads the panel on every reopen)
watch(
  () => props.models,
  () => void refreshCaches(),
)

// ---- Probes: per row (exactly that row's current values) and over the whole list ----
const testingAll = ref(false)
const testingRowId = ref<number | null>(null)
const enabledRows = computed(() => draft.value.filter((r) => r.enabled))

function verdictOf(r: EntryDraft): EmbedTestResult | null {
  return r.probe && r.probe.payload === payloadKey(r) ? r.probe.result : null
}

function record(rows: EntryDraft[], results: EmbedTestResult[] | undefined): void {
  for (const result of results ?? []) {
    const row = rows[result.index]
    if (row) row.probe = { payload: payloadKey(row), result }
  }
}

async function testAll(): Promise<void> {
  const rows = enabledRows.value
  if (!rows.length || testingAll.value) return
  testingAll.value = true
  try {
    const out = await run(() => testEmbeddings(api, rows.map(entryPayload)))
    if (out) record(rows, out.results)
  } finally {
    testingAll.value = false
  }
}

async function testRow(row: EntryDraft): Promise<void> {
  if (testingRowId.value !== null) return
  testingRowId.value = row.uiKey
  try {
    const out = await run(() => testEmbeddings(api, [entryPayload(row)]))
    if (out) record([row], out.results)
  } finally {
    testingRowId.value = null
  }
}

type CardState = 'unconfigured' | 'testing' | 'untested' | 'healthy' | 'broken'

const failures = computed(() =>
  enabledRows.value
    .map((r) => ({ r, verdict: verdictOf(r) }))
    .filter((x): x is { r: EntryDraft; verdict: EmbedTestResult } => x.verdict != null && !x.verdict.ok)
    .map((x) => ({
      id: x.r.uiKey,
      model: x.r.name.trim() || x.r.model.trim(),
      error: x.verdict.error ?? t('access.entryTestFail'),
    })),
)

const state = computed<CardState>(() => {
  if (!enabledRows.value.length) return 'unconfigured'
  if (testingAll.value || testingRowId.value !== null) return 'testing'
  const verdicts = enabledRows.value.map(verdictOf)
  if (verdicts.some((v) => v === null)) return 'untested'
  return verdicts.every((v) => v!.ok) ? 'healthy' : 'broken'
})

const badgeKey = computed(() => `access.embeddingBadge${badgeSuffix.value}`)
const badgeType = computed(
  () =>
    ({ unconfigured: 'info', testing: 'warning', untested: 'warning', healthy: 'success', broken: 'danger' })[
      state.value
    ] as 'info' | 'warning' | 'success' | 'danger',
)
const badgeSuffix = computed(
  () =>
    ({ unconfigured: 'Unconfigured', testing: 'Testing', untested: 'Untested', healthy: 'Healthy', broken: 'Broken' })[
      state.value
    ],
)
const stateText = computed(
  () =>
    ({
      unconfigured: t('access.embeddingPitch'),
      testing: t('access.embeddingChecking'),
      untested: t('access.embeddingUntested'),
      healthy: t('access.embeddingHealthy'),
      broken: t('access.embeddingBroken'),
    })[state.value],
)

// ---- Save / backfill / cache deletion ----
const saving = ref(false)

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const ok = await run(() => api.put('/api/settings', { embedding_models: draft.value.map(entryPayload) }))
    if (ok === undefined) return
    saved.value = draft.value.map((r) => ({ ...r }))
    emit('changed')
    void refreshCaches()
  } finally {
    saving.value = false
  }
}

function progressPct(row: EntryDraft): number {
  const p = row.progress
  if (!p || p.total <= 0) return 0
  return Math.max(0, Math.min(100, Math.round((p.done / p.total) * 100)))
}

async function startBackfill(row: EntryDraft): Promise<void> {
  if (runningRowId.value !== null) return
  runningRowId.value = row.uiKey
  row.progress = { done: 0, total: cacheOf(row)?.pending ?? 0 }
  try {
    const out = await run(() =>
      backfill(identityOf(row), (done, total) => {
        row.progress = { done, total }
      }),
    )
    if (!out) return
    if (out.cancelled) toastSuccess(t('access.cacheBackfillStopped', { count: out.done }))
    else if (out.done === 0) toastSuccess(t('access.embeddingUpToDate'))
    else toastSuccess(t('access.embeddingDone', { count: out.done }))
  } finally {
    runningRowId.value = null
    row.progress = null
    void refreshCaches()
    emit('changed')
  }
}

async function deleteCacheAt(key: string, model: string, count: number): Promise<void> {
  try {
    await ElMessageBox.confirm(t('access.cacheDeleteConfirm', { model, count }), t('access.cacheDeleteTitle'), {
      type: 'warning',
      confirmButtonText: t('access.cacheDelete'),
      cancelButtonText: t('common.cancel'),
    })
  } catch {
    return
  }
  deletingKey.value = key
  try {
    const out = await run(() => deleteVectorCache(api, key))
    if (out) toastSuccess(t('access.cacheDeleted', { count: out.deleted }))
  } finally {
    deletingKey.value = null
    void refreshCaches()
    emit('changed')
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.settings-hint {
  margin-bottom: 10px;
}

.state-line {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
}
.state-line.is-broken {
  color: var(--el-color-danger);
}

.probe-results {
  margin: -6px 0 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.probe-line {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-color-danger);
  font-variant-numeric: tabular-nums;
}
.broken-err {
  font-size: 12px;
  word-break: break-all;
  color: var(--el-text-color-secondary);
}

.entry-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.entry {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
}
.entry.is-off {
  opacity: 0.6;
}
/* SortableJS state, shared by both drag modes: the row left in the list is the landing preview */
.entry.is-ghost {
  border-style: dashed;
  background: var(--el-fill-color-light);
}
/* ... and the floating clone is the row being carried (touch only: a native drag is drawn by the
   browser from the row itself) */
.entry.is-drag {
  background: var(--el-bg-color);
  box-shadow: var(--el-box-shadow-light);
}
.entry-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px 5px 3px;
}
.entry-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  /* The glyph is small, the grab area should not be: full row height, generous width */
  align-self: stretch;
  width: 26px;
  min-height: 26px;
  color: var(--el-text-color-placeholder);
  cursor: grab;
  user-select: none;
  touch-action: none;
}
.grip {
  display: block;
}
.entry-handle:active {
  cursor: grabbing;
}
.entry-toggle {
  flex: 1;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0;
  border: none;
  background: transparent;
  font: inherit;
  font-size: 13px;
  color: var(--el-text-color-primary);
  text-align: left;
  cursor: pointer;
}
.entry-caret {
  color: var(--el-text-color-placeholder);
  transition: transform 0.15s;
}
.entry.is-open .entry-caret {
  transform: rotate(90deg);
}
.entry-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.entry-tags {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}
.entry-verdict {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.entry-verdict.is-ok {
  color: var(--el-color-success);
}
.entry-verdict.is-bad {
  color: var(--el-color-danger);
}
.entry-verdict.is-warn {
  color: var(--el-color-warning);
}
.entry-switch {
  flex-shrink: 0;
}

.entry-detail {
  border-top: 1px solid var(--el-border-color-lighter);
  padding: 10px;
}
.field-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 8px 10px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.field-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.entry-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}
.cache-detail {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}
.spacer {
  flex: 1;
}
.entry-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 8px;
}
.progress-bar {
  flex: 1;
  min-width: 120px;
}
.progress-text {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}

.form-hint {
  margin: 12px 0 10px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}
.form-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.form-actions-main {
  display: inline-flex;
  gap: 10px;
}

.cache-leftovers {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--el-border-color-lighter);
}
.cache-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-regular);
  margin-bottom: 8px;
}
.cache-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 0;
  font-size: 13px;
}
.cache-model {
  font-weight: 500;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cache-counts {
  flex: 1;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}
</style>
