<template>
  <!-- Reranker card: an ordered cross-encoder candidate list that re-scores search results after
       fusion. Same editor as the embedding card (collapsed line per candidate, expand for the
       labeled fields and per-candidate actions), minus the vector cache: rerankers keep no
       derived data, so there is nothing to backfill or delete. -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.rerankTitle') }}</span>
        <el-tag :type="badgeType" size="small">{{ t(badgeKey) }}</el-tag>
      </div>
    </template>

    <p class="state-line" :class="{ 'is-broken': state === 'broken' }">{{ stateText }}</p>
    <div v-if="failures.length" class="probe-results">
      <div v-for="f in failures" :key="f.id" class="probe-line">
        <span>{{ f.model || t('access.embeddingEntryUntitled') }}</span> ·
        <code class="broken-err">{{ f.error }}</code>
      </div>
    </div>

    <div ref="listEl" class="entry-list">
      <div
        v-for="(row, i) in draft"
        :key="row.id"
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
            <span class="entry-index">#{{ i + 1 }}</span>
            <span class="entry-name">{{ row.model.trim() || t('access.embeddingEntryUntitled') }}</span>
          </button>
          <span class="entry-tags">
            <span v-if="verdictOf(row)" class="entry-verdict" :class="verdictOf(row)!.ok ? 'is-ok' : 'is-bad'">
              <el-icon>
                <CircleCheckFilled v-if="verdictOf(row)!.ok" />
                <CircleCloseFilled v-else />
              </el-icon>
              {{
                verdictOf(row)!.ok
                  ? t('access.entryTestRerank', {
                      count: verdictOf(row)!.scored ?? 0,
                      ms: verdictOf(row)!.elapsed_ms ?? 0,
                    })
                  : t('access.entryTestFail')
              }}
            </span>
            <span v-else-if="row.enabled && !complete(row)" class="entry-verdict is-warn">
              {{ t('access.embeddingEntryIncomplete') }}
            </span>
          </span>
          <el-switch v-model="row.enabled" class="entry-switch" :aria-label="t('access.entryEnable')" />
        </div>

        <div v-if="row.open" class="entry-detail">
          <div class="field-grid">
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
          </div>

          <div class="entry-actions">
            <el-button size="small" :loading="testingRowId === row.id" @click="testRow(row)">
              {{ t('access.embeddingTest') }}
            </el-button>
            <span class="spacer" />
            <el-button size="small" type="danger" plain @click="removeEntry(i)">
              {{ t('common.delete') }}
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <p class="form-hint">{{ t('access.rerankFormHint') }}</p>
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
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ArrowRight, CircleCheckFilled, CircleCloseFilled, Plus } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { testRerankers } from '../../api/ops'
import { useDragOrder } from '../../composables/useDragOrder'
import type { RerankModelEntry, RerankTestResult } from '../../types'
import { run } from './caps'

const props = defineProps<{
  /** The ordered reranker candidate list as stored server-side (settings GET) */
  models: RerankModelEntry[]
}>()

const emit = defineEmits<{ changed: [] }>()

const api = useApiClient()

// ---- Draft rows: form fields plus UI state (expanded / verdict), keyed by a stable id ----
interface EntryDraft {
  id: number
  enabled: boolean
  baseUrl: string
  model: string
  apiKey: string
  open: boolean
  /** The last probe verdict with the payload it was taken for (stale verdicts are dropped) */
  probe: { payload: string; result: RerankTestResult } | null
}

let nextId = 1

/** One row's canonical write shape — exactly what a settings PUT sends for that row */
function entryPayload(r: EntryDraft) {
  return {
    enabled: r.enabled,
    base_url: r.baseUrl.trim(),
    model: r.model.trim(),
    api_key: r.apiKey.trim() === '' ? null : r.apiKey,
  }
}

function payloadKey(r: EntryDraft): string {
  return JSON.stringify(entryPayload(r))
}

function toDraftRow(e: RerankModelEntry): EntryDraft {
  return {
    id: nextId++,
    enabled: e.enabled,
    baseUrl: e.base_url,
    model: e.model,
    apiKey: e.api_key ?? '',
    open: false,
    probe: null,
  }
}

const draft = ref<EntryDraft[]>([])
const saved = ref<EntryDraft[]>([])

function payloads(rows: EntryDraft[]): string {
  return JSON.stringify(rows.map(entryPayload))
}

const dirty = computed(() => payloads(draft.value) !== payloads(saved.value))

watch(
  () => props.models,
  (models) => {
    if (dirty.value) return
    saved.value = models.map(toDraftRow)
    draft.value = models.map((e, i) => {
      const fresh = toDraftRow(e)
      const prev = draft.value[i]
      // Same candidate at the same position = the row the operator was just looking at: keep its
      // UI state across the reload (a verdict is only shown while its payload still matches)
      if (!prev || payloadKey(prev) !== payloadKey(fresh)) return fresh
      return { ...fresh, id: prev.id, open: prev.open, probe: prev.probe }
    })
  },
  { immediate: true, deep: true },
)

// Drag reordering: the list container is handed to SortableJS, the draft array follows the drop
const listEl = ref<HTMLElement | null>(null)
useDragOrder(listEl, draft)

function addEntry(): void {
  draft.value.push({
    id: nextId++,
    enabled: true,
    baseUrl: '',
    model: '',
    apiKey: '',
    open: true,
    probe: null,
  })
}

function removeEntry(i: number): void {
  draft.value.splice(i, 1)
}

function complete(r: EntryDraft): boolean {
  return r.baseUrl.trim() !== '' && r.model.trim() !== ''
}

// ---- Probes: per row (exactly that row's current values) and over the whole list ----
const testingAll = ref(false)
const testingRowId = ref<number | null>(null)
const enabledRows = computed(() => draft.value.filter((r) => r.enabled))

function verdictOf(r: EntryDraft): RerankTestResult | null {
  return r.probe && r.probe.payload === payloadKey(r) ? r.probe.result : null
}

function record(rows: EntryDraft[], results: RerankTestResult[] | undefined): void {
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
    const out = await run(() => testRerankers(api, rows.map(entryPayload)))
    if (out) record(rows, out.results)
  } finally {
    testingAll.value = false
  }
}

async function testRow(row: EntryDraft): Promise<void> {
  if (testingRowId.value !== null) return
  testingRowId.value = row.id
  try {
    const out = await run(() => testRerankers(api, [entryPayload(row)]))
    if (out) record([row], out.results)
  } finally {
    testingRowId.value = null
  }
}

type CardState = 'unconfigured' | 'testing' | 'untested' | 'healthy' | 'broken'

const failures = computed(() =>
  enabledRows.value
    .map((r) => ({ r, verdict: verdictOf(r) }))
    .filter((x): x is { r: EntryDraft; verdict: RerankTestResult } => x.verdict != null && !x.verdict.ok)
    .map((x) => ({ id: x.r.id, model: x.r.model.trim(), error: x.verdict.error ?? t('access.entryTestFail') })),
)

const state = computed<CardState>(() => {
  if (!enabledRows.value.length) return 'unconfigured'
  if (testingAll.value || testingRowId.value !== null) return 'testing'
  const verdicts = enabledRows.value.map(verdictOf)
  if (verdicts.some((v) => v === null)) return 'untested'
  return verdicts.every((v) => v!.ok) ? 'healthy' : 'broken'
})

const badgeSuffix = computed(
  () =>
    ({ unconfigured: 'Unconfigured', testing: 'Testing', untested: 'Untested', healthy: 'Healthy', broken: 'Broken' })[
      state.value
    ],
)
const badgeKey = computed(() => `access.embeddingBadge${badgeSuffix.value}`)
const badgeType = computed(
  () =>
    ({ unconfigured: 'info', testing: 'warning', untested: 'warning', healthy: 'success', broken: 'danger' })[
      state.value
    ] as 'info' | 'warning' | 'success' | 'danger',
)
const stateText = computed(
  () =>
    ({
      unconfigured: t('access.rerankPitch'),
      testing: t('access.embeddingChecking'),
      untested: t('access.rerankUntested'),
      healthy: t('access.rerankHealthy'),
      broken: t('access.rerankBroken'),
    })[state.value],
)

const saving = ref(false)

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const ok = await run(() => api.put('/api/settings', { rerank_models: draft.value.map(entryPayload) }))
    if (ok === undefined) return
    saved.value = draft.value.map((r) => ({ ...r }))
    emit('changed')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
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
.entry-index {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  font-variant-numeric: tabular-nums;
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
.spacer {
  flex: 1;
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
</style>
