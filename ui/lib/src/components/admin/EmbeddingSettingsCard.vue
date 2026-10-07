<template>
  <!-- Status-first card: the card mirrors the server's own state machine
       (effective config = switch on AND base_url AND model; health = a probe; coverage =
       embedded/(embedded+pending) for the current model) instead of presenting a permanent
       form. The form is a task: it opens from "配置服务" (auto-open while unconfigured) and
       closes once a save probes healthy. -->
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
        <el-button v-if="configured && !configuring" size="small" @click="openConfig">
          {{ t('access.embeddingConfigure') }}
        </el-button>
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

    <!-- Configuration task: seeded from the server state on open; saving commits the three
         fields, then (when enabled) immediately probes so the badge reflects reality. It opens
         by itself while unconfigured — there, configuring IS the primary flow. -->
    <el-form v-if="configuring || state === 'unconfigured'" label-position="top" class="config-form" @submit.prevent>
      <el-form-item :label="t('access.embeddingBaseUrl')">
        <el-input v-model="draft.baseUrl" :placeholder="t('access.embeddingBaseUrlPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingModelLabel')">
        <el-input v-model="draft.model" placeholder="BAAI/bge-m3 / bge-m3 / nomic-embed-text" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingApiKeyLabel')">
        <el-input v-model="draft.apiKey" show-password :placeholder="t('access.embeddingApiKeyPlaceholder')" />
      </el-form-item>
      <!-- Vectors are keyed by model: a switch orphans every stored vector until backfilled.
           Say the cost up front, with the count that will be invalidated. -->
      <el-alert
        v-if="modelChanged && vectorCount > 0"
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
        <el-button v-if="configured" @click="closeConfig">{{ t('common.cancel') }}</el-button>
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

// ---- Configuration task ----
const configuring = ref(false)
const saving = ref(false)
const draft = reactive({ baseUrl: '', model: '', apiKey: '' })
// Baseline the draft diffs against: the values currently stored server-side
const saved = ref({ baseUrl: '', model: '', apiKey: '' })

// Keep fields in sync while the form is closed (a config edit elsewhere or a reload must not
// be swallowed; the auto-open unconfigured form must show the server's current values).
// While configuring, the operator's draft owns the fields.
watch(
  () => [props.embeddingBaseUrl, props.embeddingModel, props.embeddingApiKey] as const,
  ([u, m, k]) => {
    if (configuring.value) return
    saved.value = { baseUrl: u, model: m, apiKey: k }
    draft.baseUrl = u
    draft.model = m
    draft.apiKey = k
  },
  { immediate: true },
)

function openConfig(): void {
  draft.baseUrl = props.embeddingBaseUrl
  draft.model = props.embeddingModel
  draft.apiKey = props.embeddingApiKey
  saved.value = { baseUrl: props.embeddingBaseUrl, model: props.embeddingModel, apiKey: props.embeddingApiKey }
  configuring.value = true
}

function closeConfig(): void {
  configuring.value = false
}

const dirty = computed(
  () =>
    draft.baseUrl !== saved.value.baseUrl || draft.model !== saved.value.model || draft.apiKey !== saved.value.apiKey,
)

const modelChanged = computed(() => draft.model.trim() !== saved.value.model.trim())

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const ok = await run(() =>
      api.put('/api/settings', {
        embedding_base_url: draft.baseUrl,
        embedding_model: draft.model,
        embedding_api_key: draft.apiKey,
      }),
    )
    if (ok === undefined) return
    saved.value = { baseUrl: draft.baseUrl, model: draft.model, apiKey: draft.apiKey }
    emit('changed')
    lastTest.value = null
    if (props.embeddingEnabled) {
      // The operator expects "works?" answered now: probe right after committing
      const result = await check()
      if (result?.ok) configuring.value = false
      // Stay open on failure: the badge carries the reason, the form is the fix loop
    } else {
      configuring.value = false // parked: nothing to probe
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
