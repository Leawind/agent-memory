<template>
  <!-- 新建 / 编辑能力 -->
  <el-dialog
    v-model="visible"
    :title="editing ? t('access.editTitle', { name: editing.name }) : t('access.createTitle')"
    :width="compact ? '96%' : '480px'"
  >
    <el-form label-position="top" @submit.prevent>
      <el-form-item v-if="!editing" :label="t('access.nameLabel')">
        <el-input v-model="form.name" :placeholder="t('access.namePlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.permsLabel')">
        <div class="presets">
          <span class="presets-label">{{ t('access.presets') }}</span>
          <el-radio-group v-model="preset" size="small" @change="applyPreset">
            <el-radio-button value="admin">{{ t('access.presetAdmin') }}</el-radio-button>
            <el-radio-button value="member">{{ t('access.presetMember') }}</el-radio-button>
            <el-radio-button value="viewer">{{ t('access.presetViewer') }}</el-radio-button>
            <el-radio-button value="custom">{{ t('access.presetCustom') }}</el-radio-button>
          </el-radio-group>
        </div>
        <div class="caps">
          <el-checkbox v-for="c in CAPS" :key="c.key" v-model="form.caps[c.key]" @change="onManualToggle">
            {{ capLabel(c.key) }}
          </el-checkbox>
        </div>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="visible = false">{{ t('common.cancel') }}</el-button>
      <el-button type="primary" :loading="submitting" @click="submit">
        {{ t('common.save') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { CAPS, PRESETS, capLabel, emptyCaps, run, type IdentityRow } from './caps'

defineProps<{
  /** compact（<960px）时对话框加宽到 96% */
  compact: boolean
}>()

const emit = defineEmits<{
  /** 编辑保存成功（父层刷新列表） */
  saved: []
  /** 新建成功，附一次性 token 明文（父层弹 token 一次性展示并刷新列表） */
  created: [payload: { name: string; token: string }]
}>()

const api = useApiClient()

const visible = ref(false)
const submitting = ref(false)
const editing = ref<IdentityRow | null>(null)
const preset = ref<'admin' | 'member' | 'viewer' | 'custom'>('custom')
const form = reactive<{ name: string; caps: Record<string, boolean> }>({
  name: '',
  caps: emptyCaps(),
})

function applyPreset(): void {
  if (preset.value === 'custom') return
  const keys = new Set(PRESETS[preset.value] ?? [])
  for (const c of CAPS) form.caps[c.key] = keys.has(c.key)
}

function onManualToggle(): void {
  preset.value = 'custom'
}

function openCreate(): void {
  editing.value = null
  form.name = ''
  Object.assign(form.caps, emptyCaps())
  preset.value = 'member'
  applyPreset()
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

<style scoped>
.presets {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.presets-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 16px;
}
</style>
