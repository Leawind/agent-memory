<template>
  <!-- Status-first card: the card mirrors the server's own state machine
       (effective config = switch on AND base_url AND model; health = a probe; coverage =
       embedded/(embedded+pending) for the current model) instead of presenting a permanent
       form. The three-field config form is always visible — no expand/collapse dance; the
       save button only wakes up once the draft differs from the server state. -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.embeddingTitle') }}</span>
        <el-tag :type="badgeType" size="small">{{ t(badgeKey) }}</el-tag>
      </div>
    </template>

    <!-- Intent switch first: effective immediately (PUT on change), disabled until the service
         is configured — an intent without config has nothing to enable. The state line names
         the state and, when broken, carries the server's reason verbatim. -->
    <div class="state-row">
      <el-tooltip :disabled="configured" :content="t('access.embeddingNotConfigured')" placement="top">
        <el-switch
          :model-value="embeddingEnabled"
          :loading="toggling"
          :disabled="!configured"
          :aria-label="t('access.embeddingEnabledLabel')"
          @change="onToggle"
        />
      </el-tooltip>
      <span class="state-line" :class="{ 'is-broken': state === 'broken' }">
        <template v-if="state === 'unconfigured'">{{ t('access.embeddingPitch') }}</template>
        <template v-else-if="state === 'parked'">{{ t('access.embeddingParked') }}</template>
        <template v-else-if="state === 'testing'">{{ t('access.embeddingChecking') }}</template>
        <template v-else-if="state === 'untested'">
          {{ t('access.embeddingUntested', { model: embeddingModel.trim() }) }}
        </template>
        <template v-else-if="state === 'healthy'">
          {{ t('access.embeddingHealthy', { model: embeddingModel.trim(), dim: lastTest?.dim ?? 0 }) }}
        </template>
        <template v-else>
          {{ t('access.embeddingBroken') }}
          <code class="broken-err">{{ lastTest?.error }}</code>
        </template>
      </span>
      <span class="state-actions">
        <el-button
          v-if="state === 'untested' || state === 'healthy' || state === 'broken'"
          size="small"
          :loading="testing"
          @click="check"
        >
          {{ state === 'untested' ? t('access.embeddingCheck') : t('access.embeddingRecheck') }}
        </el-button>
      </span>
    </div>

    <!-- Vector coverage (only meaningful when the config is effective server-side). While a
         backfill drains, live progress replaces the static counts. -->
    <div v-if="coverage" class="coverage-row">
      <template v-if="progress">
        <span class="coverage-text">{{ t('access.embeddingBackfilling', progress) }}</span>
        <el-progress
          class="coverage-bar"
          :percentage="progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 100"
          :stroke-width="8"
        />
      </template>
      <template v-else>
        <span class="coverage-text">
          {{ t('access.embeddingCoverageLine', { embedded: coverage.embedded ?? 0, total: coverageTotal }) }}
        </span>
        <el-button v-if="(coverage.pending ?? 0) > 0" size="small" :loading="backfilling" @click="runBackfill">
          {{ t('access.runBackfill') }}
        </el-button>
      </template>
    </div>

    <!-- Configuration: always visible, seeded from the server state; saving commits the three
         fields, then (when enabled) immediately probes so the badge reflects reality. -->
    <el-form label-position="top" class="config-form" @submit.prevent>
      <el-form-item :label="t('access.embeddingBaseUrl')">
        <el-input v-model="draft.baseUrl" :placeholder="t('access.embeddingBaseUrlPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingModelLabel')">
        <el-input v-model="draft.model" placeholder="BAAI/bge-m3 / bge-m3 / nomic-embed-text" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingApiKeyLabel')">
        <el-input v-model="draft.apiKey" show-password :placeholder="t('access.embeddingApiKeyPlaceholder')" />
      </el-form-item>
      <!-- Instruction prefixes for asymmetric embedding models (E5 etc.); applied verbatim,
           so a trailing space is meaningful. Symmetric models (bge-m3) leave both empty. -->
      <el-form-item :label="t('access.embeddingQueryPrefixLabel')">
        <el-input v-model="draft.queryPrefix" :placeholder="t('access.embeddingQueryPrefixPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingPassagePrefixLabel')">
        <el-input v-model="draft.passagePrefix" :placeholder="t('access.embeddingPassagePrefixPlaceholder')" />
      </el-form-item>
      <!-- Recall floor for the semantic channel: query-time only, never re-keys vectors -->
      <el-form-item :label="t('access.embeddingMinSimilarityLabel')">
        <el-input v-model="draft.minSimilarity" :placeholder="t('access.embeddingMinSimilarityPlaceholder')" />
      </el-form-item>
      <!-- Vectors are keyed by model + prefixes: a switch orphans every stored vector until
           backfilled. Say the cost up front, with the count that will be invalidated. -->
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
        <el-button :disabled="!dirty" :loading="saving" type="primary" @click="save">
          {{ t('access.embeddingSaveTest') }}
        </el-button>
        <el-button :disabled="!dirty" @click="revert">{{ t('access.embeddingRevert') }}</el-button>
      </div>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import type { EmbedTestResp } from '../../types'
import { run } from './caps'

const props = defineProps<{
  embeddingEnabled: boolean
  embeddingBaseUrl: string
  embeddingModel: string
  embeddingApiKey: string
  embeddingQueryPrefix: string
  embeddingPassagePrefix: string
  embeddingMinSimilarity: string
  /** Stats fetched by the parent's load: the server's own view of effectivity + coverage */
  stats: import('../../types').StatsInfo | null
  /** In compact mode (<960px) the coverage progress bar narrows */
  compact: boolean
}>()

const emit = defineEmits<{ changed: [] }>()

const api = useApiClient()
const { backfilling, backfill } = useAdmin()

// ---- State: mirror the server, derive everything from props + the last probe ----
type CardState = 'unconfigured' | 'parked' | 'testing' | 'untested' | 'healthy' | 'broken'

const testing = ref(false)
const lastTest = ref<EmbedTestResp | null>(null)

// The server embeds only when the switch is on AND base_url AND model are set
const configured = computed(() => props.embeddingBaseUrl.trim() !== '' && props.embeddingModel.trim() !== '')

const state = computed<CardState>(() => {
  if (!configured.value) return 'unconfigured'
  if (!props.embeddingEnabled) return 'parked'
  if (testing.value) return 'testing'
  if (!lastTest.value) return 'untested'
  return lastTest.value.ok ? 'healthy' : 'broken'
})

const badgeKey = computed(
  () =>
    ({
      unconfigured: 'access.embeddingBadgeUnconfigured',
      parked: 'access.embeddingBadgeParked',
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
      parked: 'info',
      testing: 'warning',
      untested: 'warning',
      healthy: 'success',
      broken: 'danger',
    })[state.value] as 'info' | 'warning' | 'success' | 'danger',
)

// Coverage comes from the server's effective config (stats), not from the raw switch
const coverage = computed(() => (props.stats?.embedding?.enabled ? props.stats.embedding : undefined))
const coverageTotal = computed(() => (coverage.value?.embedded ?? 0) + (coverage.value?.pending ?? 0))
const vectorCount = computed(() => coverage.value?.embedded ?? 0)

// ---- Intent switch: immediate PUT; a failed PUT is toasted by run() and the switch stays put ----
const toggling = ref(false)

async function onToggle(value: boolean | string | number): Promise<void> {
  toggling.value = true
  try {
    const ok = await run(() => api.put('/api/settings', { embedding_enabled: value === true }))
    if (ok === undefined) return
    lastTest.value = null // the probe result belonged to the previous on/off state
    emit('changed')
  } finally {
    toggling.value = false
  }
}

// ---- Probe: always tests the server's saved config (that is the endpoint's contract) ----
async function check(): Promise<EmbedTestResp | null> {
  testing.value = true
  try {
    const out = await run(() => api.post<EmbedTestResp>('/api/embeddings/test', {}))
    if (out) lastTest.value = out
    return out ?? null
  } finally {
    testing.value = false
  }
}

// ---- Configuration form (always visible) ----
const saving = ref(false)
const draft = reactive({ baseUrl: '', model: '', apiKey: '', queryPrefix: '', passagePrefix: '', minSimilarity: '' })
// Baseline the draft diffs against: the values currently stored server-side
const saved = ref({ baseUrl: '', model: '', apiKey: '', queryPrefix: '', passagePrefix: '', minSimilarity: '' })

const dirty = computed(
  () =>
    draft.baseUrl !== saved.value.baseUrl ||
    draft.model !== saved.value.model ||
    draft.apiKey !== saved.value.apiKey ||
    draft.queryPrefix !== saved.value.queryPrefix ||
    draft.passagePrefix !== saved.value.passagePrefix ||
    draft.minSimilarity !== saved.value.minSimilarity,
)

// Keep fields in sync while the draft is clean (a config edit elsewhere or a reload must not
// be swallowed). An in-progress edit (dirty) owns the fields instead: a background refresh
// triggered by another card must not clobber what the operator is typing.
watch(
  () =>
    [
      props.embeddingBaseUrl,
      props.embeddingModel,
      props.embeddingApiKey,
      props.embeddingQueryPrefix,
      props.embeddingPassagePrefix,
      props.embeddingMinSimilarity,
    ] as const,
  ([u, m, k, q, p, s]) => {
    if (dirty.value) return
    saved.value = { baseUrl: u, model: m, apiKey: k, queryPrefix: q, passagePrefix: p, minSimilarity: s }
    draft.baseUrl = u
    draft.model = m
    draft.apiKey = k
    draft.queryPrefix = q
    draft.passagePrefix = p
    draft.minSimilarity = s
  },
  { immediate: true },
)

function revert(): void {
  draft.baseUrl = props.embeddingBaseUrl
  draft.model = props.embeddingModel
  draft.apiKey = props.embeddingApiKey
  draft.queryPrefix = props.embeddingQueryPrefix
  draft.passagePrefix = props.embeddingPassagePrefix
  draft.minSimilarity = props.embeddingMinSimilarity
  saved.value = {
    baseUrl: props.embeddingBaseUrl,
    model: props.embeddingModel,
    apiKey: props.embeddingApiKey,
    queryPrefix: props.embeddingQueryPrefix,
    passagePrefix: props.embeddingPassagePrefix,
    minSimilarity: props.embeddingMinSimilarity,
  }
}

// The embedding identity = model + prefixes: changing any of them re-keys every vector
const identityChanged = computed(
  () =>
    draft.model.trim() !== saved.value.model.trim() ||
    draft.queryPrefix !== saved.value.queryPrefix ||
    draft.passagePrefix !== saved.value.passagePrefix,
)

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const ok = await run(() =>
      api.put('/api/settings', {
        embedding_base_url: draft.baseUrl,
        embedding_model: draft.model,
        embedding_api_key: draft.apiKey,
        embedding_query_prefix: draft.queryPrefix,
        embedding_passage_prefix: draft.passagePrefix,
        embedding_min_similarity: draft.minSimilarity,
      }),
    )
    if (ok === undefined) return
    saved.value = {
      baseUrl: draft.baseUrl,
      model: draft.model,
      apiKey: draft.apiKey,
      queryPrefix: draft.queryPrefix,
      passagePrefix: draft.passagePrefix,
      minSimilarity: draft.minSimilarity,
    }
    emit('changed')
    lastTest.value = null
    if (props.embeddingEnabled) {
      // The operator expects "works?" answered now: probe right after committing
      await check()
      // A failed probe keeps the badge red with the reason verbatim; the form stays to fix it
    }
  } finally {
    saving.value = false
  }
}

// ---- Backfill: the admin endpoint drains one small batch per call and reports remaining;
// loop it and surface real progress instead of a bare spinner ----
const progress = ref<{ done: number; total: number } | null>(null)

async function runBackfill(): Promise<void> {
  try {
    // run() toasts a failed run and returns undefined; coverage refreshes either way
    const total = await run(() =>
      backfill((done, totalCount) => {
        progress.value = { done, total: totalCount }
      }),
    )
    if (total !== undefined) {
      if (total === 0) toastSuccess(t('access.embeddingUpToDate'))
      else toastSuccess(t('access.embeddingDone', { count: total }))
    }
  } finally {
    progress.value = null
    emit('changed') // refresh coverage even when the run failed mid-way
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
  display: block;
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

.coverage-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 14px;
}
.coverage-text {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  font-variant-numeric: tabular-nums;
}
.coverage-bar {
  flex: 1;
  max-width: 260px;
}

.config-form {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--el-border-color-lighter);
}
.form-hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}
.form-actions {
  display: flex;
  justify-content: flex-end;
}
</style>
