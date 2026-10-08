<template>
  <div class="lifecycle-cards">
    <el-card shadow="never">
      <template #header>{{ t('lifecycle.policyTitle') }}</template>
      <el-form label-position="top" @submit.prevent>
        <el-form-item v-for="kind in kinds" :key="kind" :label="t(`lifecycle.${kind}`)">
          <div class="half-life-row">
            <el-checkbox :model-value="draft.half_life_days[kind] !== null" @change="setDecay(kind, !!$event)">
              {{ t('lifecycle.enableDecay') }}
            </el-checkbox>
            <el-input-number
              v-if="draft.half_life_days[kind] !== null"
              v-model="draft.half_life_days[kind]"
              :min="1"
              :max="36500"
              :precision="1"
            />
            <span>{{ draft.half_life_days[kind] === null ? t('lifecycle.noDecay') : t('lifecycle.days') }}</span>
          </div>
        </el-form-item>
        <el-form-item :label="t('lifecycle.freshnessWeight')">
          <el-input-number v-model="draft.freshness_weight" :min="0" :max="5" :step="0.1" :precision="2" />
        </el-form-item>
        <el-form-item :label="t('lifecycle.reinforcementWeight')">
          <el-input-number v-model="draft.reinforcement_weight" :min="0" :max="5" :step="0.1" :precision="2" />
        </el-form-item>
      </el-form>
      <p class="hint">{{ t('lifecycle.policyHint') }}</p>
      <el-button type="primary" :disabled="!dirty || !valid" :loading="saving" @click="save">{{
        t('common.save')
      }}</el-button>
    </el-card>
    <el-card shadow="never">
      <template #header>{{ t('lifecycle.accessTitle') }}</template>
      <el-descriptions v-if="stats" :column="1" size="small">
        <el-descriptions-item :label="t('lifecycle.rawEvents')">{{ stats.raw_events }}</el-descriptions-item>
        <el-descriptions-item :label="t('lifecycle.buckets')">{{ stats.hourly_buckets }}</el-descriptions-item>
        <el-descriptions-item :label="t('lifecycle.tracked')">{{ stats.tracked_memories }}</el-descriptions-item>
        <el-descriptions-item :label="t('lifecycle.dropped')">{{ stats.dropped_read_events }}</el-descriptions-item>
      </el-descriptions>
      <p class="hint">{{ t('lifecycle.accessHint') }}</p>
      <div class="actions">
        <el-button :loading="busy === 'refresh'" :disabled="!!busy" @click="refresh">{{
          t('lifecycle.refresh')
        }}</el-button>
        <el-button :loading="busy === 'compact'" :disabled="!!busy" @click="maintain('compact')">{{
          t('lifecycle.compact')
        }}</el-button>
        <el-button :loading="busy === 'rebuild'" :disabled="!!busy" @click="maintain('rebuild')">{{
          t('lifecycle.rebuild')
        }}</el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../../api/client'
import { t } from '../../i18n'
import { toastSuccess } from '../../toast'
import type { AccessStats, LifecyclePolicy, LifecycleMetadata } from '../../types'
import { run } from './caps'

const props = defineProps<{ policy: LifecyclePolicy; active: boolean }>()
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()
const kinds = ['fact', 'preference', 'procedure', 'context', 'event'] as const
const defaults = { fact: 365, preference: 365, procedure: 730, context: 30, event: 7 }
const copy = (policy: LifecyclePolicy): LifecyclePolicy => ({ ...policy, half_life_days: { ...policy.half_life_days } })
const draft = ref(copy(props.policy))
const stats = ref<AccessStats | null>(null)
const saving = ref(false)
const busy = ref('')
watch(
  () => props.policy,
  (value) => {
    draft.value = copy(value)
  },
)
watch(
  () => props.active,
  (active) => {
    if (active) void refresh()
  },
  { immediate: true },
)
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(props.policy))
const valid = computed(
  () =>
    Object.values(draft.value.half_life_days).every(
      (days) => days === null || (Number.isFinite(days) && days >= 1 && days <= 36500),
    ) &&
    [draft.value.freshness_weight, draft.value.reinforcement_weight].every(
      (weight) => Number.isFinite(weight) && weight >= 0 && weight <= 5,
    ),
)

function setDecay(kind: LifecycleMetadata['kind'], enabled: boolean): void {
  draft.value.half_life_days[kind] = enabled ? (draft.value.half_life_days[kind] ?? defaults[kind]) : null
}

async function save(): Promise<void> {
  if (!dirty.value || !valid.value) return
  saving.value = true
  try {
    await run(async () => {
      await api.put('/api/settings', { lifecycle_policy: draft.value })
      toastSuccess(t('access.saved'))
      emit('changed')
    })
  } finally {
    saving.value = false
  }
}

async function refresh(): Promise<void> {
  if (busy.value) return
  busy.value = 'refresh'
  try {
    await run(async () => {
      stats.value = await api.get<AccessStats>('/api/access/stats')
    })
  } finally {
    busy.value = ''
  }
}

async function maintain(action: 'compact' | 'rebuild'): Promise<void> {
  if (busy.value) return
  busy.value = action
  try {
    await run(async () => {
      await api.post(`/api/access/${action}`, {})
      stats.value = await api.get<AccessStats>('/api/access/stats')
    })
  } finally {
    busy.value = ''
  }
}
</script>

<style scoped>
.lifecycle-cards {
  display: grid;
  gap: 16px;
}
.half-life-row,
.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}
.hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
