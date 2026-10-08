<template>
  <el-card shadow="never" class="search-limits-card">
    <template #header>{{ t('access.searchLimitsTitle') }}</template>
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.semanticCandidates')">
        <el-input-number v-model="draft.semantic_candidates" :min="1" :max="1000" :step="10" :precision="0" />
      </el-form-item>
      <el-form-item :label="t('access.rerankCandidates')">
        <el-input-number v-model="draft.rerank_candidates" :min="1" :max="1000" :step="10" :precision="0" />
      </el-form-item>
    </el-form>
    <p class="form-hint" :class="{ 'is-invalid': !valid }">{{ t('access.searchLimitsHint') }}</p>
    <el-button type="primary" :disabled="!dirty || !valid" :loading="saving" @click="save">
      {{ t('common.save') }}
    </el-button>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../../api/client'
import { t } from '../../i18n'
import { toastSuccess } from '../../toast'
import type { SearchLimits } from '../../types'
import { run } from './caps'

const props = defineProps<{ limits: SearchLimits }>()
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()
const draft = ref({ ...props.limits })
const saving = ref(false)
watch(
  () => props.limits,
  (limits) => {
    draft.value = { ...limits }
  },
)
const valid = computed(() => {
  const { semantic_candidates: semantic, rerank_candidates: rerank } = draft.value
  return Number.isInteger(semantic) && Number.isInteger(rerank) && semantic >= rerank && semantic <= 1000 && rerank >= 1
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
</script>

<style scoped>
.form-hint {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}
.is-invalid {
  color: var(--el-color-danger);
}
</style>
