<template>
  <el-card shadow="never" class="search-limits-card">
    <template #header>{{ t('access.searchLimitsTitle') }}</template>

    <!-- Settings list: one setting per line, label on the left and a compact control on the
         right. (The earlier top-label form made every narrow number input own a full row.) -->
    <div class="setting-list">
      <div class="setting-row">
        <span class="setting-label">{{ t('access.semanticCandidates') }}</span>
        <el-input-number
          v-model="draft.semantic_candidates"
          class="setting-control"
          :aria-label="t('access.semanticCandidates')"
          :min="1"
          :max="1000"
          :step="10"
          :precision="0"
        />
      </div>
      <div class="setting-row">
        <span class="setting-label">{{ t('access.rerankCandidates') }}</span>
        <el-input-number
          v-model="draft.rerank_candidates"
          class="setting-control"
          :aria-label="t('access.rerankCandidates')"
          :min="1"
          :max="1000"
          :step="10"
          :precision="0"
        />
      </div>
      <div class="setting-row">
        <span class="setting-label">{{ t('access.adaptiveRerank') }}</span>
        <el-switch v-model="draft.adaptive.enabled" :aria-label="t('access.adaptiveRerank')" />
      </div>
    </div>

    <!-- Adaptive bounds only exist while the switch is on; the inset panel keeps them a
         clearly separate group without breaking the one-setting-per-line rhythm. -->
    <div v-if="draft.adaptive.enabled" class="adaptive-block">
      <div class="setting-list">
        <div class="setting-row">
          <span class="setting-label">{{ t('access.minCandidates') }}</span>
          <el-input-number
            v-model="draft.adaptive.min_candidates"
            class="setting-control"
            :aria-label="t('access.minCandidates')"
            :min="1"
            :max="1000"
            :precision="0"
          />
        </div>
        <div class="setting-row">
          <span class="setting-label">{{ t('access.maxCandidates') }}</span>
          <el-input-number
            v-model="draft.adaptive.max_candidates"
            class="setting-control"
            :aria-label="t('access.maxCandidates')"
            :min="1"
            :max="1000"
            :precision="0"
          />
        </div>
        <div class="setting-row">
          <span class="setting-label">{{ t('access.targetLatency') }}</span>
          <el-input-number
            v-model="draft.adaptive.target_latency_ms"
            class="setting-control"
            :aria-label="t('access.targetLatency')"
            :min="50"
            :max="15000"
            :step="100"
            :precision="0"
          />
        </div>
        <div class="setting-row">
          <span class="setting-label">{{ t('access.maxInputChars') }}</span>
          <el-input-number
            v-model="draft.adaptive.max_input_chars"
            class="setting-control"
            :aria-label="t('access.maxInputChars')"
            :min="draft.adaptive.min_candidates * 8000"
            :max="8000000"
            :step="8000"
            :precision="0"
          />
        </div>
      </div>
    </div>

    <p class="form-hint" :class="{ 'is-invalid': !valid }">{{ t('access.searchLimitsHint') }}</p>
    <div class="actions">
      <el-button type="primary" :disabled="!dirty || !valid" :loading="saving" @click="save">
        {{ t('common.save') }}
      </el-button>
      <el-button :loading="refreshing" @click="refresh">{{ t('access.runtimeRefresh') }}</el-button>
    </div>
    <p class="form-hint">{{ t('access.adaptiveHint') }}</p>
    <el-table v-if="runtime?.models?.length" class="runtime-table" :data="runtime.models" size="small">
      <el-table-column prop="id" :label="t('access.runtimeModel')" min-width="100" />
      <el-table-column prop="selected_candidates" :label="t('access.runtimeBudget')" min-width="100" />
      <el-table-column prop="last_candidates" :label="t('access.runtimeLastPool')" min-width="100" />
      <el-table-column :label="t('access.runtimeLatency')" min-width="110"
        ><template #default="{ row }">{{ Math.round(row.last_latency_ms) }} ms</template></el-table-column
      >
      <el-table-column prop="samples" :label="t('access.runtimeSamples')" min-width="90" />
      <el-table-column prop="failures" :label="t('access.runtimeFailures')" min-width="90" />
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../../api/client'
import { t } from '../../i18n'
import { toastSuccess } from '../../toast'
import type { SearchLimits, SearchRuntime } from '../../types'
import { run } from './caps'

const props = defineProps<{ limits: SearchLimits; active: boolean }>()
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()
const copy = (limits: SearchLimits): SearchLimits => ({ ...limits, adaptive: { ...limits.adaptive } })
const draft = ref(copy(props.limits))
const saving = ref(false)
const refreshing = ref(false)
const runtime = ref<SearchRuntime | null>(null)
watch(
  () => props.active,
  (active) => {
    if (active) void refresh()
  },
  { immediate: true },
)
watch(
  () => props.limits,
  (limits) => {
    draft.value = copy(limits)
  },
)
const valid = computed(() => {
  const { semantic_candidates: semantic, rerank_candidates: rerank } = draft.value
  const a = draft.value.adaptive
  return (
    Number.isInteger(semantic) &&
    Number.isInteger(rerank) &&
    semantic >= rerank &&
    semantic <= 1000 &&
    rerank >= 1 &&
    [a.min_candidates, a.max_candidates, a.target_latency_ms, a.max_input_chars].every(Number.isInteger) &&
    a.min_candidates >= 1 &&
    a.max_candidates >= a.min_candidates &&
    a.max_candidates <= 1000 &&
    a.target_latency_ms >= 50 &&
    a.target_latency_ms <= 15000 &&
    a.max_input_chars >= a.min_candidates * 8000 &&
    a.max_input_chars <= 8000000 &&
    (!a.enabled || (semantic >= a.max_candidates && rerank >= a.min_candidates && rerank <= a.max_candidates))
  )
})
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(props.limits))

async function save(): Promise<void> {
  if (!valid.value || !dirty.value) return
  saving.value = true
  try {
    await run(async () => {
      await api.put('/api/settings', { search_limits: draft.value })
      toastSuccess(t('access.saved'))
      emit('changed')
    })
  } finally {
    saving.value = false
  }
}

async function refresh(): Promise<void> {
  if (refreshing.value) return
  refreshing.value = true
  try {
    await run(async () => {
      runtime.value = await api.get<SearchRuntime>('/api/search/runtime')
    })
  } finally {
    refreshing.value = false
  }
}
</script>

<style scoped>
/* One setting per line: label left, compact control right (same rhythm as the auth toggles) */
.setting-list {
  display: flex;
  flex-direction: column;
}
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 40px;
}
/* Hairline between rows: keeps the label and its far-right control reading as one setting */
.setting-row + .setting-row {
  border-top: 1px solid var(--el-border-color-extra-light);
}
.setting-label {
  flex-shrink: 0;
  font-size: 13px;
  color: var(--el-text-color-regular);
}
.setting-control {
  flex: 0 0 auto;
  width: 160px;
}
/* Adaptive bounds: an inset panel, so enabling the switch grows a clearly separate group */
.adaptive-block {
  margin-top: 8px;
  padding: 2px 14px;
  border-radius: 10px;
  background: var(--el-fill-color-lighter);
}
.form-hint {
  margin: 10px 0 0;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  line-height: 1.6;
}
.is-invalid {
  color: var(--el-color-danger);
}
.actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin: 12px 0 4px;
}
.runtime-table {
  margin-top: 12px;
}
</style>
