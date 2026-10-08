<template>
  <el-card shadow="never" class="tag-rules-card">
    <template #header>{{ t('access.tagRulesTitle') }}</template>
    <el-form label-position="top" @submit.prevent>
      <section v-for="section in sections" :key="section.key">
        <h4>{{ t(section.title) }}</h4>
        <p class="form-hint">{{ t(section.hint) }}</p>
        <div v-for="(rule, index) in draft[section.key]" :key="index" class="rule-row">
          <el-form-item :label="t('access.ruleName')">
            <el-input v-model="rule.name" :maxlength="100" />
          </el-form-item>
          <el-form-item :label="t('access.ruleExpression')">
            <el-input v-model="rule.expression" :maxlength="32768" :placeholder="section.example" />
          </el-form-item>
          <el-button @click="draft[section.key].splice(index, 1)">{{ t('common.delete') }}</el-button>
        </div>
        <el-button
          :disabled="draft[section.key].length >= 128"
          @click="draft[section.key].push({ name: '', expression: '' })"
        >
          {{ t(section.add) }}
        </el-button>
      </section>
    </el-form>
    <div class="rule-actions">
      <el-button :disabled="!valid" :loading="checking" @click="check">{{ t('access.previewRules') }}</el-button>
      <el-button type="primary" :disabled="!valid || !dirty" :loading="saving" @click="save">
        {{ t('common.save') }}
      </el-button>
    </div>
    <template v-if="preview">
      <el-alert v-if="preview.positive_cycles" type="info" :title="t('access.positiveCycles')" :closable="false" />
      <el-alert
        :type="preview.valid ? 'success' : 'warning'"
        :title="preview.valid ? t('access.rulesValid') : t('access.rulesInvalid', { count: preview.total_violations })"
        :closable="false"
      />
      <el-table v-if="preview.violations.length" :data="preview.violations" size="small">
        <el-table-column prop="id" label="ID" width="90" />
        <el-table-column prop="summary" :label="t('access.ruleMemory')" />
        <el-table-column :label="t('access.failedRules')">
          <template #default="{ row }">{{ row.rules.join(', ') }}</template>
        </el-table-column>
      </el-table>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../../api/client'
import { t } from '../../i18n'
import { toastSuccess } from '../../toast'
import type { TagRule, TagRulePreview } from '../../types'
import { run } from './caps'

const props = withDefaults(defineProps<{ rules: TagRule[]; derivations?: TagRule[] }>(), { derivations: () => [] })
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()
const initial = () => ({
  constraints: props.rules.map((rule) => ({ ...rule })),
  derivations: props.derivations.map((rule) => ({ ...rule })),
})
const draft = ref(initial())
const sections = [
  {
    key: 'constraints' as const,
    title: 'access.constraintsTitle',
    hint: 'access.tagRulesHint',
    add: 'access.addRule',
    example: 'mutex(wind,horse,cow)',
  },
  {
    key: 'derivations' as const,
    title: 'access.derivationsTitle',
    hint: 'access.derivationsHint',
    add: 'access.addDerivation',
    example: 'vue3 => web',
  },
]
const saving = ref(false)
const checking = ref(false)
const preview = ref<TagRulePreview | null>(null)
watch(
  () => [props.rules, props.derivations],
  () => {
    draft.value = initial()
  },
)
watch(
  draft,
  () => {
    preview.value = null
  },
  { deep: true },
)
const valid = computed(() =>
  [...draft.value.constraints, ...draft.value.derivations].every((rule) => rule.name.trim() && rule.expression.trim()),
)
const dirty = computed(() => JSON.stringify(draft.value) !== JSON.stringify(initial()))

async function check(): Promise<void> {
  checking.value = true
  const snapshot = JSON.stringify(draft.value)
  try {
    await run(async () => {
      const result = await api.post<TagRulePreview>('/api/tag-rules/preview', JSON.parse(snapshot))
      if (snapshot === JSON.stringify(draft.value)) preview.value = result
    })
  } finally {
    checking.value = false
  }
}

async function save(): Promise<void> {
  if (!valid.value || !dirty.value) return
  saving.value = true
  try {
    await run(async () => {
      await api.put('/api/tag-rules', draft.value)
      toastSuccess(t('access.saved'))
      emit('changed')
    })
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.form-hint {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}
.rule-row {
  padding: 12px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.rule-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 16px 0;
}
.rule-actions .el-button {
  margin-left: 0;
}
</style>
