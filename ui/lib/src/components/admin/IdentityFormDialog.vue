<template>
  <!-- Create / edit capabilities -->
  <FormDialog
    v-model:visible="visible"
    :title="editing ? t('access.editTitle', { name: editing.name }) : t('access.createTitle')"
    :width="compact ? '96%' : '480px'"
    :saving="submitting"
    @submit="submit"
  >
    <el-form-item v-if="!editing" :label="t('access.nameLabel')">
      <el-input v-model="form.name" :placeholder="t('access.namePlaceholder')" />
    </el-form-item>
    <el-form-item :label="t('access.permsLabel')">
      <CapsEditor v-model:caps="form.caps" v-model:preset="preset" />
    </el-form-item>
  </FormDialog>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { CAPS, emptyCaps, run, type IdentityRow, type PresetKey } from './caps'
import FormDialog from '../FormDialog.vue'
import CapsEditor from './CapsEditor.vue'

defineProps<{
  /** In compact mode (<960px) the dialog widens to 96% */
  compact: boolean
}>()

const emit = defineEmits<{
  /** Edit saved successfully (the parent refreshes the list) */
  saved: []
  /** Creation succeeded, carrying the one-time token in plaintext (the parent shows the one-time token dialog and refreshes the list) */
  created: [payload: { name: string; token: string }]
}>()

const api = useApiClient()

const visible = ref(false)
const submitting = ref(false)
const editing = ref<IdentityRow | null>(null)
const preset = ref<PresetKey>('custom')
const form = reactive<{ name: string; caps: Record<string, boolean> }>({
  name: '',
  caps: emptyCaps(),
})

function openCreate(): void {
  editing.value = null
  form.name = ''
  Object.assign(form.caps, emptyCaps())
  preset.value = 'member'
  visible.value = true
}

function openEdit(row: IdentityRow): void {
  editing.value = row
  form.name = row.name
  for (const c of CAPS) form.caps[c.key] = row.permissions?.[c.key] === true
  preset.value = 'custom'
  visible.value = true
}

defineExpose({ openCreate, openEdit })

async function submit(): Promise<void> {
  submitting.value = true
  try {
    if (editing.value) {
      const name = editing.value.name
      await run(() => api.put(`/api/identities/${encodeURIComponent(name)}`, { permissions: { ...form.caps } }))
      toastSuccess(t('access.saved'))
      emit('saved')
    } else {
      const created = await run(() =>
        api.post<IdentityRow & { token?: string }>('/api/identities', {
          name: form.name,
          permissions: { ...form.caps },
        }),
      )
      if (created?.token) {
        emit('created', { name: form.name.trim(), token: created.token })
      }
    }
    visible.value = false
  } finally {
    submitting.value = false
  }
}
</script>
