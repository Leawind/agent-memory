<template>
  <el-dialog
    :model-value="visible"
    :title="t('lifecycle.title')"
    width="min(480px, 95vw)"
    append-to-body
    @update:model-value="emit('update:visible', $event)"
  >
    <p>{{ t(`lifecycle.${metadata.state}`) }}</p>
    <el-form label-position="top" :disabled="saving" @submit.prevent>
      <el-form-item :label="t('lifecycle.kind')">
        <el-select v-model="draft.kind">
          <el-option v-for="kind in kinds" :key="kind" :value="kind" :label="t(`lifecycle.${kind}`)" />
        </el-select>
      </el-form-item>
      <el-form-item :label="t('lifecycle.expires')">
        <el-date-picker v-model="expiry" type="datetime" clearable :placeholder="t('lifecycle.noExpiry')" />
      </el-form-item>
      <el-form-item>
        <el-checkbox v-model="draft.pinned">{{ t('lifecycle.pinned') }}</el-checkbox>
      </el-form-item>
      <el-form-item>
        <el-checkbox v-model="draft.archived">{{ t('lifecycle.archived') }}</el-checkbox>
      </el-form-item>
    </el-form>
    <p class="lifecycle-hint">{{ t('lifecycle.hint') }}</p>
    <template #footer>
      <el-button @click="emit('update:visible', false)">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="saving" :disabled="!dirty || !valid" @click="save">{{
        t('common.save')
      }}</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../api/client'
import { t } from '../i18n'
import { toastError, toastSuccess } from '../toast'
import type { LifecycleMetadata } from '../types'

const props = defineProps<{ visible: boolean; memoryId: string; metadata: LifecycleMetadata }>()
const emit = defineEmits<{ 'update:visible': [value: boolean]; saved: [] }>()
const kinds = ['fact', 'preference', 'procedure', 'context', 'event'] as const
const api = useApiClient()
const initial = () => ({
  kind: props.metadata.kind,
  pinned: props.metadata.pinned,
  archived: props.metadata.archived_at !== null,
})
const draft = ref(initial())
const expiry = ref<Date | null>(null)
const saving = ref(false)
watch(
  () => [props.visible, props.metadata],
  () => {
    draft.value = initial()
    expiry.value = props.metadata.expires_at === null ? null : new Date(props.metadata.expires_at * 1000)
  },
  { immediate: true },
)
const timestamp = computed(() => (expiry.value ? Math.floor(expiry.value.getTime() / 1000) : null))
const valid = computed(
  () =>
    timestamp.value === null ||
    (Number.isFinite(timestamp.value) && timestamp.value >= 0 && timestamp.value <= 253402300799),
)
const dirty = computed(
  () => JSON.stringify(draft.value) !== JSON.stringify(initial()) || timestamp.value !== props.metadata.expires_at,
)

async function save(): Promise<void> {
  if (!dirty.value || !valid.value) return
  saving.value = true
  try {
    await api.put(`/api/memories/${encodeURIComponent(props.memoryId)}/lifecycle`, {
      ...draft.value,
      expires_at: timestamp.value,
    })
    toastSuccess(t('access.saved'))
    emit('saved')
    emit('update:visible', false)
  } catch (error) {
    toastError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.lifecycle-hint {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}
</style>
