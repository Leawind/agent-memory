<template>
  <!-- Reranker card: an ordered cross-encoder candidate list that re-scores search results
       after fusion. Same editor discipline as the embedding card, minus the vector cache
       (rerankers keep no derived data). -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.rerankTitle') }}</span>
        <el-tag :type="badgeType" size="small">{{ t(badgeKey) }}</el-tag>
      </div>
    </template>

    <div class="state-row">
      <span class="state-line" :class="{ 'is-broken': state === 'broken' }">
        <template v-if="state === 'unconfigured'">{{ t('access.rerankPitch') }}</template>
        <template v-else-if="state === 'testing'">{{ t('access.embeddingChecking') }}</template>
        <template v-else-if="state === 'untested'">{{ t('access.rerankUntested') }}</template>
        <template v-else-if="state === 'healthy'">{{ t('access.rerankHealthy') }}</template>
        <template v-else>{{ t('access.rerankBroken') }}</template>
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

    <div v-if="lastTest" class="probe-results">
      <div v-for="r in lastTest.results" :key="r.model" class="probe-line" :class="r.ok ? 'is-ok' : 'is-bad'">
        {{ r.ok ? '✓' : '✗' }} {{ r.model }}<template v-if="r.ok"> · {{ r.elapsed_ms ?? 0 }}ms</template
        ><template v-else>
          · <code class="broken-err">{{ r.error }}</code></template
        >
      </div>
    </div>

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
        </div>
      </div>
    </div>

    <p class="form-hint">{{ t('access.rerankFormHint') }}</p>
    <div class="form-actions">
      <el-button @click="addEntry">{{ t('access.embeddingAddCandidate') }}</el-button>
      <span class="form-actions-main">
        <el-button :disabled="!dirty" @click="revert">{{ t('access.embeddingRevert') }}</el-button>
        <el-button :disabled="!dirty" :loading="saving" type="primary" @click="save">
          {{ t('access.embeddingSaveTest') }}
        </el-button>
      </span>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ArrowDown, ArrowUp, Delete } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { testRerankers } from '../../api/ops'
import type { EmbedModelEntry, RerankModelEntry, RerankTestResp } from '../../types'
import { run } from './caps'

const props = defineProps<{
  /** The ordered reranker candidate list as stored server-side (settings GET) */
  models: EmbedModelEntry[] | RerankModelEntry[]
}>()

const emit = defineEmits<{ changed: [] }>()

const api = useApiClient()

type CardState = 'unconfigured' | 'testing' | 'untested' | 'healthy' | 'broken'

const testing = ref(false)
const saving = ref(false)
const lastTest = ref<RerankTestResp | null>(null)

const state = computed<CardState>(() => {
  if (!props.models.some((e) => e.enabled && e.base_url.trim() !== '' && e.model.trim() !== '')) {
    return 'unconfigured'
  }
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

async function check(): Promise<void> {
  testing.value = true
  try {
    const out = await run(() => testRerankers(api))
    if (out) lastTest.value = out
  } finally {
    testing.value = false
  }
}

interface EntryDraft {
  enabled: boolean
  baseUrl: string
  model: string
  apiKey: string
}

function toDraft(entries: Array<EmbedModelEntry | RerankModelEntry>): EntryDraft[] {
  return entries.map((e) => ({
    enabled: e.enabled,
    baseUrl: e.base_url,
    model: e.model,
    apiKey: e.api_key ?? '',
  }))
}

const draft = ref<EntryDraft[]>([])
const saved = ref<EntryDraft[]>([])
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(saved.value))

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
  draft.value.push({ enabled: true, baseUrl: '', model: '', apiKey: '' })
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

async function save(): Promise<void> {
  if (!dirty.value || saving.value) return
  saving.value = true
  try {
    const payload = draft.value.map((r) => ({
      enabled: r.enabled,
      base_url: r.baseUrl.trim(),
      model: r.model.trim(),
      api_key: r.apiKey.trim() === '' ? null : r.apiKey,
    }))
    const ok = await run(() => api.put('/api/settings', { rerank_models: payload }))
    if (ok === undefined) return
    saved.value = draft.value.map((r) => ({ ...r }))
    emit('changed')
    lastTest.value = null
    if (payload.some((e) => e.enabled)) {
      await check()
    }
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
</style>
