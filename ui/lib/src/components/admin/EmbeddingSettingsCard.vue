<template>
  <!-- Multi-candidate card: an ordered embedding list (failover priority = list order), each
       entry independently enabled, plus per-identity vector-cache management. The editor is
       always visible; the save button wakes up once the draft differs from the server state. -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.embeddingTitle') }}</span>
        <el-tag :type="badgeType" size="small">{{ t(badgeKey) }}</el-tag>
      </div>
    </template>

    <div class="state-row">
      <span class="state-line" :class="{ 'is-broken': state === 'broken' }">
        <template v-if="state === 'unconfigured'">{{ t('access.embeddingPitch') }}</template>
        <template v-else-if="state === 'testing'">{{ t('access.embeddingChecking') }}</template>
        <template v-else-if="state === 'untested'">{{ t('access.embeddingUntested') }}</template>
        <template v-else-if="state === 'healthy'">{{ t('access.embeddingHealthy') }}</template>
        <template v-else>{{ t('access.embeddingBroken') }}</template>
      </span>
      <span class="state-actions">
        <el-button
          v-if="state !== 'unconfigured' && state !== 'testing'"
          size="small"
          :loading="testing"
          @click="check"
        >
          {{ state === 'untested' ? t('access.embeddingCheck') : t('access.embeddingRecheck') }}
        </el-button>
      </span>
    </div>

    <!-- Per-candidate probe verdicts (the failover order the server walks at search time) -->
    <div v-if="lastTest" class="probe-results">
      <div v-for="r in lastTest.results" :key="r.model" class="probe-line" :class="r.ok ? 'is-ok' : 'is-bad'">
        {{ r.ok ? '✓' : '✗' }} {{ r.model
        }}<template v-if="r.ok">
          · {{ t('access.embeddingDim', { dim: r.dim ?? 0 }) }} · {{ r.elapsed_ms ?? 0 }}ms</template
        ><template v-else>
          · <code class="broken-err">{{ r.error }}</code></template
        >
      </div>
    </div>

    <!-- Candidate editor: list order is the failover priority -->
    <div class="entry-list">
      <div v-for="(row, i) in draft" :key="i" class="entry" :class="{ 'is-off': !row.enabled }">
        <div class="entry-head">
          <el-switch v-model="row.enabled" :aria-label="t('access.embeddingEntryEnabled')" />
          <span class="entry-index">#{{ i + 1 }}</span>
          <el-input v-model="row.model" :placeholder="t('access.embeddingModelLabel')" class="entry-model" />
          <el-button-group class="entry-ops">
            <el-button size="small" :disabled="i === 0" @click="move(i, -1)">
              <el-icon><ArrowUp /></el-icon>
            </el-button>
            <el-button size="small" :disabled="i === draft.length - 1" @click="move(i, 1)">
              <el-icon><ArrowDown /></el-icon>
            </el-button>
            <el-button size="small" @click="removeEntry(i)">
              <el-icon><Delete /></el-icon>
            </el-button>
          </el-button-group>
        </div>
        <div class="entry-fields">
          <el-input v-model="row.baseUrl" :placeholder="t('access.embeddingBaseUrlPlaceholder')" class="span-2" />
          <el-input v-model="row.apiKey" show-password :placeholder="t('access.embeddingApiKeyPlaceholder')" />
          <el-input v-model="row.minSimilarity" :placeholder="t('access.embeddingMinSimilarityPlaceholder')" />
          <el-input v-model="row.queryPrefix" :placeholder="t('access.embeddingQueryPrefixPlaceholder')" />
          <el-input v-model="row.passagePrefix" :placeholder="t('access.embeddingPassagePrefixPlaceholder')" />
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
      <el-button @click="addEntry">{{ t('access.embeddingAddCandidate') }}</el-button>
      <span class="form-actions-main">
        <el-button :disabled="!dirty" @click="revert">{{ t('access.embeddingRevert') }}</el-button>
        <el-button :disabled="!dirty" :loading="saving" type="primary" @click="save">
          {{ t('access.embeddingSaveTest') }}
        </el-button>
      </span>
    </div>

    <!-- Vector caches per identity: several models' caches coexist; each can be backfilled or
         dropped independently. Unconfigured leftovers can only be deleted. -->
    <div v-if="caches.length" class="cache-section">
      <div class="cache-title">{{ t('access.cacheTitle') }}</div>
      <div v-for="c in caches" :key="c.key" class="cache-row">
        <span class="cache-model" :title="c.key">{{ c.model }}</span>
        <span class="cache-counts">
          {{ t('access.cacheCounts', { embedded: c.embedded, pending: c.pending }) }}
          <el-tag v-if="!c.configured" size="small" type="info">{{ t('access.cacheUnconfigured') }}</el-tag>
        </span>
        <span class="cache-actions">
          <el-button
            v-if="c.pending > 0 && c.configured"
            size="small"
            :loading="backfillKey === c.key"
            @click="backfillModel(c.key)"
          >
            {{ t('access.cacheBackfill') }}
          </el-button>
          <el-button size="small" :loading="deletingKey === c.key" @click="askDeleteCache(c)">
            {{ t('access.cacheDelete') }}
          </el-button>
        </span>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ArrowDown, ArrowUp, Delete } from '@element-plus/icons-vue'
import { ElMessageBox } from 'element-plus'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { backfillEmbeddings, deleteVectorCache, listVectorCaches, testEmbeddings } from '../../api/ops'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import type { EmbedModelEntry, EmbedTestResp, StatsInfo, VectorCacheInfo } from '../../types'
import { run } from './caps'

const props = defineProps<{
  /** The ordered candidate list as stored server-side (settings GET) */
  models: EmbedModelEntry[]
  /** Stats fetched by the parent's load: the server's own view of effectivity */
  stats: StatsInfo | null
}>()

const emit = defineEmits<{ changed: [] }>()

const api = useApiClient()
const { backfill } = useAdmin()

// ---- State: mirror the server, derive everything from props + the last probe ----
type CardState = 'unconfigured' | 'testing' | 'untested' | 'healthy' | 'broken'

const testing = ref(false)
const saving = ref(false)
const lastTest = ref<EmbedTestResp | null>(null)

const state = computed<CardState>(() => {
  if (!props.stats?.embedding?.enabled) return 'unconfigured'
  if (testing.value) return 'testing'
  if (!lastTest.value) return 'untested'
  return lastTest.value.ok ? 'healthy' : 'broken'
})

const badgeKey = computed(
  () =>
    ({
      unconfigured: 'access.embeddingBadgeUnconfigured',
      testing: 'access.embeddingBadgeTesting',
      untested: 'access.embeddingBadgeUntested',
      healthy: 'access.embeddingBadgeHealthy',
      broken: 'access.embeddingBadgeBroken',
    })[state.value],
)
const badgeType = computed(
  () =>
    ({
      unconfigured: 'info',
      testing: 'warning',
      untested: 'warning',
      healthy: 'success',
      broken: 'danger',
    })[state.value] as 'info' | 'warning' | 'success' | 'danger',
)

// ---- Probe: always tests the candidates saved server-side (that is the endpoint's contract) ----
async function check(): Promise<void> {
  testing.value = true
  try {
    const out = await run(() => testEmbeddings(api))
    if (out) lastTest.value = out
  } finally {
    testing.value = false
  }
}

// ---- Candidate editor ----
interface EntryDraft {
  enabled: boolean
  baseUrl: string
  model: string
  apiKey: string
  queryPrefix: string
  passagePrefix: string
  minSimilarity: string
}

function toDraft(entries: EmbedModelEntry[]): EntryDraft[] {
  return entries.map((e) => ({
    enabled: e.enabled,
    baseUrl: e.base_url,
    model: e.model,
    apiKey: e.api_key ?? '',
    queryPrefix: e.query_prefix ?? '',
    passagePrefix: e.passage_prefix ?? '',
    minSimilarity: e.min_similarity == null ? '' : String(e.min_similarity),
  }))
}

const draft = ref<EntryDraft[]>([])
const saved = ref<EntryDraft[]>([])
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(saved.value))

// Keep the fields in sync while the draft is clean (a config edit elsewhere or a reload must
// not be swallowed). An in-progress edit (dirty) owns the fields instead.
watch(
  () => props.models,
  (models) => {
    if (dirty.value) return
    saved.value = toDraft(models)
    draft.value = toDraft(models)
  },
  { immediate: true, deep: true },
)

function addEntry(): void {
  draft.value.push({
    enabled: true,
    baseUrl: '',
    model: '',
    apiKey: '',
    queryPrefix: '',
    passagePrefix: '',
    minSimilarity: '',
  })
}

function removeEntry(i: number): void {
  draft.value.splice(i, 1)
}

function move(i: number, delta: number): void {
  const j = i + delta
  if (j < 0 || j >= draft.value.length) return
  const rows = draft.value
  ;[rows[i], rows[j]] = [rows[j], rows[i]]
}

function revert(): void {
  draft.value = saved.value.map((r) => ({ ...r }))
}

// The embedding identity = model + prefixes: changing any of them re-keys every vector
function identityOf(row: EntryDraft): string {
  return `${row.model.trim()}|q=${row.queryPrefix}|p=${row.passagePrefix}`
}
const identityChanged = computed(
  () =>
    draft.value.length !== saved.value.length ||
    draft.value.some((row, i) => identityOf(row) !== identityOf(saved.value[i])),
)
const vectorCount = computed(() => caches.value.reduce((sum, c) => sum + c.embedded, 0))

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const payload = draft.value.map((r) => ({
      enabled: r.enabled,
      base_url: r.baseUrl.trim(),
      model: r.model.trim(),
      api_key: r.apiKey.trim() === '' ? null : r.apiKey,
      query_prefix: r.queryPrefix === '' ? null : r.queryPrefix,
      passage_prefix: r.passagePrefix === '' ? null : r.passagePrefix,
      min_similarity: r.minSimilarity.trim() === '' ? null : Number(r.minSimilarity),
    }))
    const ok = await run(() => api.put('/api/settings', { embedding_models: payload }))
    if (ok === undefined) return
    saved.value = draft.value.map((r) => ({ ...r }))
    emit('changed')
    lastTest.value = null
    if (payload.some((e) => e.enabled)) {
      // The operator expects "works?" answered now: probe right after committing
      await check()
    }
  } finally {
    saving.value = false
  }
}

// ---- Vector caches per identity ----
const caches = ref<VectorCacheInfo[]>([])
const backfillKey = ref<string | null>(null)
const deletingKey = ref<string | null>(null)

async function refreshCaches(): Promise<void> {
  const out = await run(() => listVectorCaches(api))
  caches.value = out?.caches ?? []
}

onMounted(() => {
  void refreshCaches()
})

async function backfillModel(key: string): Promise<void> {
  backfillKey.value = key
  try {
    const total = await run(() => backfill(key))
    if (total !== undefined) {
      if (total === 0) toastSuccess(t('access.embeddingUpToDate'))
      else toastSuccess(t('access.embeddingDone', { count: total }))
    }
  } finally {
    backfillKey.value = null
    void refreshCaches()
    emit('changed') // refresh coverage even when the run failed mid-way
  }
}

async function askDeleteCache(c: VectorCacheInfo): Promise<void> {
  try {
    await ElMessageBox.confirm(
      t('access.cacheDeleteConfirm', { model: c.model, count: c.embedded }),
      t('access.cacheDeleteTitle'),
      {
        type: 'warning',
        confirmButtonText: t('access.cacheDelete'),
        cancelButtonText: t('common.cancel'),
      },
    )
  } catch {
    return
  }
  deletingKey.value = c.key
  try {
    const out = await run(() => deleteVectorCache(api, c.key))
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
  margin-top: 4px;
}

.state-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.state-line {
  font-size: 13px;
  line-height: 1.6;
  color: var(--el-text-color-regular);
  min-width: 0;
}
.state-line.is-broken {
  color: var(--el-color-danger);
}
.broken-err {
  font-size: 12px;
  word-break: break-all;
  color: var(--el-text-color-secondary);
}
.state-actions {
  margin-left: auto;
  flex-shrink: 0;
  display: inline-flex;
  gap: 8px;
}

.probe-results {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.probe-line {
  font-size: 12px;
  line-height: 1.6;
  font-variant-numeric: tabular-nums;
}
.probe-line.is-ok {
  color: var(--el-color-success);
}
.probe-line.is-bad {
  color: var(--el-color-danger);
}

.entry-list {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.entry {
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.entry.is-off {
  opacity: 0.55;
}
.entry-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.entry-index {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.entry-model {
  flex: 1;
  min-width: 120px;
}
.entry-ops {
  flex-shrink: 0;
}
.entry-fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.entry-fields .span-2 {
  grid-column: span 2;
}

.form-hint {
  margin: 10px 0 12px;
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

.cache-section {
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
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
  flex: 1;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.cache-actions {
  flex-shrink: 0;
  display: inline-flex;
  gap: 8px;
}
</style>
