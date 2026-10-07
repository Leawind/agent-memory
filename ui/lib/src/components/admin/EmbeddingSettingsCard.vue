<template>
  <!-- Semantic search config: OpenAI-compatible /embeddings (cloud or local Ollama); falls back to keyword search automatically when the service is unavailable.
       Vector coverage and backfill live in this card too (the former standalone card at the bottom was removed): config and coverage on one screen, visible once enabled -->
  <el-card shadow="never">
    <template #header>{{ t('access.embeddingTitle') }}</template>
    <el-alert :title="t('access.embeddingHint')" type="info" show-icon :closable="false" class="settings-hint" />
    <div class="enable-row">
      <span class="enable-label">{{ t('access.embeddingEnabledLabel') }}</span>
      <el-switch v-model="enabled" />
      <span class="enable-hint">{{ t('access.embeddingEnabledHint') }}</span>
    </div>
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.embeddingBaseUrl')">
        <el-input v-model="baseUrl" :placeholder="t('access.embeddingBaseUrlPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingModelLabel')">
        <el-input v-model="model" placeholder="BAAI/bge-m3 / bge-m3 / nomic-embed-text" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingApiKeyLabel')">
        <el-input v-model="apiKey" show-password :placeholder="t('access.embeddingApiKeyPlaceholder')" />
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button type="primary" :loading="saving" :disabled="!dirty" @click="saveAndTest">
        {{ t('access.embeddingSaveTest') }}
      </el-button>
    </div>
    <el-alert
      v-if="test?.ok"
      :title="t('access.embeddingTestOk', { dim: test.dim ?? 0, ms: test.elapsed_ms ?? 0 })"
      type="success"
      show-icon
      :closable="false"
      class="settings-hint"
    />
    <el-alert
      v-else-if="test && !test.ok"
      :title="t('access.embeddingTestFail', { error: test.error ?? '' })"
      type="error"
      show-icon
      :closable="false"
      class="settings-hint"
    />

    <!-- Vector coverage and backfill (admin endpoints; the button loops until nothing is pending): stats fetched with the parent's load, not rendered while disabled -->
    <template v-if="coverage">
      <el-divider class="coverage-divider" />
      <div class="coverage-header">
        <span class="coverage-title">{{ t('access.embeddingCoverageTitle') }}</span>
        <el-button size="small" :icon="Refresh" :loading="backfilling" @click="runBackfill">
          {{ t('access.runBackfill') }}
        </el-button>
      </div>
      <el-descriptions size="small" :column="compact ? 1 : 2">
        <el-descriptions-item :label="t('access.embeddingModel')">
          <code class="coverage-model">{{ coverage.model ?? '—' }}</code>
        </el-descriptions-item>
        <el-descriptions-item :label="t('access.embeddingCoverage')">
          {{ coverage.embedded ?? 0 }} / {{ coverageTotal }}
        </el-descriptions-item>
      </el-descriptions>
      <el-alert
        v-if="(coverage.pending ?? 0) > 0"
        :title="t('access.embeddingPending', { count: coverage.pending })"
        type="warning"
        show-icon
        :closable="false"
        class="settings-hint"
      />
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import type { EmbeddingCoverage } from '../../types'
import { run } from './caps'

const props = defineProps<{
  embeddingEnabled: boolean
  embeddingBaseUrl: string
  embeddingModel: string
  embeddingApiKey: string
  /** Vector coverage stats (fetched by the parent's load); the coverage section is hidden when not enabled or not loaded */
  stats: import('../../types').StatsInfo | null
  /** In compact mode (<960px) the coverage descriptions list renders in a single column */
  compact: boolean
}>()

const emit = defineEmits<{ changed: [] }>()

// Local editing state, refreshed when the parent loads
const enabled = ref(props.embeddingEnabled)
const baseUrl = ref(props.embeddingBaseUrl)
const model = ref(props.embeddingModel)
const apiKey = ref(props.embeddingApiKey)
watch(
  () => [props.embeddingEnabled, props.embeddingBaseUrl, props.embeddingModel, props.embeddingApiKey] as const,
  ([e, u, m, k]) => {
    enabled.value = e
    baseUrl.value = u
    model.value = m
    apiKey.value = k
  },
)

// A save re-sends the whole config and re-tests the connection: only meaningful when something
// actually differs from the server state the props mirror (the parent reloads after each save,
// which re-syncs the local draft and settles the button back to disabled)
const dirty = computed(
  () =>
    enabled.value !== props.embeddingEnabled ||
    baseUrl.value !== props.embeddingBaseUrl ||
    model.value !== props.embeddingModel ||
    apiKey.value !== props.embeddingApiKey,
)

// Only show the coverage section when the service is enabled (also not rendered while stats are
// unloaded, avoiding a flash of "not enabled")
const coverage = computed<EmbeddingCoverage | undefined>(() =>
  props.stats?.embedding?.enabled ? props.stats.embedding : undefined,
)
// Coverage denominator = embedded + pending backfill
const coverageTotal = computed(() => (coverage.value?.embedded ?? 0) + (coverage.value?.pending ?? 0))

const api = useApiClient()
const { backfilling, backfill } = useAdmin()
const saving = ref(false)
// run returns undefined on failure (already toasted); test must accommodate three states:
// null = not tested, undefined = test failed, object = result
const test = ref<null | undefined | { ok: boolean; dim?: number; elapsed_ms?: number; error?: string }>(null)

// After saving, immediately run a connectivity test with the server-side config; `changed` makes
// the parent refetch the stats so the coverage section updates right away
async function saveAndTest(): Promise<void> {
  if (!dirty.value) return
  saving.value = true
  test.value = null
  try {
    const saved = await run(() =>
      api.put('/api/settings', {
        embedding_enabled: enabled.value,
        embedding_base_url: baseUrl.value,
        embedding_model: model.value,
        embedding_api_key: apiKey.value,
      }),
    )
    if (saved === undefined) return // save failure already toasted
    toastSuccess(t('access.saved'))
    emit('changed')
    if (!enabled.value) return // no test needed when disabled
    test.value = await run(() => api.post('/api/embeddings/test', {}))
  } finally {
    saving.value = false
  }
}

/** Give result feedback after the backfill finishes (0 processed still counts as success — there was nothing pending) and trigger the parent to refresh coverage */
async function runBackfill(): Promise<void> {
  const total = await run(backfill)
  if (total === undefined) return
  emit('changed')
  if (total === 0) toastSuccess(t('access.embeddingUpToDate'))
  else toastSuccess(t('access.embeddingDone', { count: total }))
}
</script>

<style scoped>
.enable-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 18px;
}
.enable-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.enable-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
.coverage-divider {
  margin: 20px 0 16px;
}
.coverage-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.coverage-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.coverage-model {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
