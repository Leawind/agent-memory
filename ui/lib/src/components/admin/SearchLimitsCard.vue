<template>
  <el-card shadow="never" class="search-limits-card">
    <template #header>{{ t('access.searchLimitsTitle') }}</template>

    <!-- Compact field grid: narrow numeric controls share rows instead of each claiming a
         full-width line (the old label-position="top" form turned 7 small fields into 7 rows). -->
    <div class="field-grid">
      <label class="field">
        <ModelFieldLabel :label="t('access.semanticCandidates')" />
        <el-input-number v-model="draft.semantic_candidates" :min="1" :max="1000" :step="10" :precision="0" />
      </label>
      <label class="field">
        <ModelFieldLabel :label="t('access.rerankCandidates')" />
        <el-input-number v-model="draft.rerank_candidates" :min="1" :max="1000" :step="10" :precision="0" />
      </label>
      <div class="field">
        <ModelFieldLabel :label="t('access.adaptiveRerank')" />
        <div class="switch-line"><el-switch v-model="draft.adaptive.enabled" /></div>
      </div>
    </div>

    <!-- Adaptive bounds are a separate concern and only exist while the switch is on: grouping
         them keeps the base budget readable and the extra controls from mixing into its row. -->
    <div v-if="draft.adaptive.enabled" class="adaptive-block">
      <div class="field-grid">
        <label class="field">
          <ModelFieldLabel :label="t('access.minCandidates')" />
          <el-input-number v-model="draft.adaptive.min_candidates" :min="1" :max="1000" :precision="0" />
        </label>
        <label class="field">
          <ModelFieldLabel :label="t('access.maxCandidates')" />
          <el-input-number v-model="draft.adaptive.max_candidates" :min="1" :max="1000" :precision="0" />
        </label>
        <label class="field">
          <ModelFieldLabel :label="t('access.targetLatency')" />
          <el-input-number
            v-model="draft.adaptive.target_latency_ms"
            :min="50"
            :max="15000"
            :step="100"
            :precision="0"
          />
        </label>
        <label class="field">
          <ModelFieldLabel :label="t('access.maxInputChars')" />
          <el-input-number
            v-model="draft.adaptive.max_input_chars"
            :min="draft.adaptive.min_candidates * 8000"
            :max="8000000"
            :step="8000"
            :precision="0"
          />
        </label>
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
import ModelFieldLabel from './ModelFieldLabel.vue'

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
/* Narrow numeric controls share rows; the track size keeps the +/- steppers usable */
.field-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
  gap: 12px 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.field :deep(.el-input-number) {
  width: 100%;
}
.switch-line {
  display: flex;
  align-items: center;
  min-height: 32px;
}
/* Adaptive bounds: an inset panel, so enabling the switch grows a clearly separate group */
.adaptive-block {
  margin-top: 12px;
  padding: 12px 14px;
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
