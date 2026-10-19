<template>
  <div class="lifecycle-cards">
    <el-card shadow="never">
      <template #header>{{ t('lifecycle.policyTitle') }}</template>
      <p class="hint">{{ t('lifecycle.orderHint') }}</p>
      <div v-for="(rule, index) in draft.rules" :key="index" class="decay-rule">
        <div class="setting-row expression-row">
          <span class="setting-label">{{ t('predicates.expression') }}</span>
          <el-input
            v-model="rule.predicate"
            class="expression-input"
            :maxlength="32768"
            placeholder="@temporary &amp; !durable"
          />
        </div>
        <div class="rule-controls">
          <el-checkbox :model-value="rule.half_life_days !== null" @change="rule.half_life_days = $event ? 30 : null">{{
            t('lifecycle.enableDecay')
          }}</el-checkbox>
          <el-input-number
            v-if="rule.half_life_days !== null"
            v-model="rule.half_life_days"
            class="number-control"
            :min="1"
            :max="36500"
            :precision="1"
          />
          <span>{{ rule.half_life_days === null ? t('lifecycle.noDecay') : t('lifecycle.days') }}</span>
          <span class="rule-actions">
            <el-button :disabled="index === 0" @click="move(index, -1)">{{ t('lifecycle.moveUp') }}</el-button>
            <el-button :disabled="index === draft.rules.length - 1" @click="move(index, 1)">{{
              t('lifecycle.moveDown')
            }}</el-button>
            <el-button @click="draft.rules.splice(index, 1)">{{ t('common.delete') }}</el-button>
          </span>
        </div>
      </div>
      <el-button class="add-rule" :disabled="draft.rules.length >= 128" @click="addRule">{{
        t('lifecycle.addRule')
      }}</el-button>

      <!-- One setting per line: label left, controls right, matching the rest of the admin cards -->
      <div class="setting-row default-decay">
        <span class="setting-label">{{ t('lifecycle.defaultDecay') }}</span>
        <div class="setting-controls">
          <el-checkbox
            :model-value="draft.default_half_life_days !== null"
            @change="draft.default_half_life_days = $event ? 30 : null"
            >{{ t('lifecycle.enableDecay') }}</el-checkbox
          >
          <el-input-number
            v-if="draft.default_half_life_days !== null"
            v-model="draft.default_half_life_days"
            class="number-control"
            :min="1"
            :max="36500"
            :precision="1"
          />
          <span>{{ draft.default_half_life_days === null ? t('lifecycle.noDecay') : t('lifecycle.days') }}</span>
        </div>
      </div>
      <div class="setting-row weight-row">
        <span class="setting-label">{{ t('lifecycle.freshnessWeight') }}</span>
        <el-input-number
          v-model="draft.freshness_weight"
          class="setting-control"
          :aria-label="t('lifecycle.freshnessWeight')"
          :min="0"
          :max="5"
          :step="0.1"
          :precision="2"
        />
      </div>
      <div class="setting-row weight-row">
        <span class="setting-label">{{ t('lifecycle.reinforcementWeight') }}</span>
        <el-input-number
          v-model="draft.reinforcement_weight"
          class="setting-control"
          :aria-label="t('lifecycle.reinforcementWeight')"
          :min="0"
          :max="5"
          :step="0.1"
          :precision="2"
        />
      </div>

      <p class="hint">{{ t('lifecycle.policyHint') }}</p>
      <el-button type="primary" :disabled="!dirty || !valid" :loading="saving" @click="save">{{
        t('common.save')
      }}</el-button>
    </el-card>
    <el-card shadow="never">
      <template #header>{{ t('lifecycle.accessTitle') }}</template>
      <div v-if="stats" class="stat-grid">
        <div class="stat">
          <span class="stat-label">{{ t('lifecycle.rawEvents') }}</span>
          <span class="stat-value">{{ stats.raw_events }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">{{ t('lifecycle.buckets') }}</span>
          <span class="stat-value">{{ stats.hourly_buckets }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">{{ t('lifecycle.tracked') }}</span>
          <span class="stat-value">{{ stats.tracked_memories }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">{{ t('lifecycle.dropped') }}</span>
          <span class="stat-value">{{ stats.dropped_read_events }}</span>
        </div>
      </div>
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
import type { AccessStats, LifecyclePolicy } from '../../types'
import { run } from './caps'

const props = defineProps<{ policy: LifecyclePolicy; active: boolean }>()
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()
const copy = (policy: LifecyclePolicy): LifecyclePolicy => ({
  ...policy,
  rules: policy.rules.map((rule) => ({ ...rule })),
})
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
    [draft.value.default_half_life_days, ...draft.value.rules.map((rule) => rule.half_life_days)].every(
      (days) => days === null || (Number.isFinite(days) && days >= 1 && days <= 36500),
    ) &&
    draft.value.rules.every((rule) => !!rule.predicate.trim()) &&
    [draft.value.freshness_weight, draft.value.reinforcement_weight].every(
      (weight) => Number.isFinite(weight) && weight >= 0 && weight <= 5,
    ),
)

function addRule(): void {
  draft.value.rules.push({ predicate: '', half_life_days: 30 })
}

function move(index: number, direction: number): void {
  const next = index + direction
  if (next < 0 || next >= draft.value.rules.length) return
  const [rule] = draft.value.rules.splice(index, 1)
  draft.value.rules.splice(next, 0, rule!)
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
/* One setting per line: label left, compact controls right */
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 40px;
}
/* Hairline between rows: keeps each label and its far-right control reading as one setting */
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
.number-control {
  flex: 0 0 auto;
  width: 150px;
}
.setting-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}
/* Predicate expressions are the one wide input: label left, field takes the rest of the row */
.expression-row {
  align-items: center;
}
.expression-row .setting-label {
  flex: 0 0 84px;
}
.expression-input {
  flex: 1;
  min-width: 0;
}
.decay-rule {
  padding: 10px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
/* Rule controls sit under the expression field, aligned with it */
.rule-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  padding: 2px 0 6px 84px;
}
.rule-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-left: auto;
}
.add-rule {
  margin: 10px 0 4px;
}
.default-decay {
  margin-top: 14px;
}
.weight-row + .weight-row {
  margin-top: 2px;
}
.hint {
  margin: 6px 0 10px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
}
/* Narrow dialog: drop the expression indent so the rule controls keep the full width */
@media (max-width: 600px) {
  .expression-row {
    flex-wrap: wrap;
  }
  .expression-row .setting-label {
    flex-basis: 100%;
  }
  .rule-controls {
    padding-left: 0;
  }
}
/* Event counters: compact tiles instead of four full-width label/value lines */
.stat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 8px;
  margin-bottom: 10px;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 8px;
}
.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.stat-value {
  font-size: 18px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: center;
}
</style>
