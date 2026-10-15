<template>
  <el-card shadow="never" class="keys-card">
    <template #header>
      <div class="card-head">
        <span>{{ t('keys.title') }}</span>
        <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('keys.create') }}</el-button>
      </div>
    </template>
    <p class="form-hint">{{ t('keys.hint') }}</p>
    <el-alert v-if="!keys.length" :title="t('keys.empty')" type="info" :closable="false" />
    <el-table v-else :data="keys" size="small">
      <el-table-column prop="name" :label="t('keys.colName')" min-width="140" show-overflow-tooltip />
      <el-table-column :label="t('keys.colType')" width="150">
        <template #default="{ row }">
          <el-tag size="small">{{ t(`editor.attrType.${row.type}`) }}</el-tag>
          <el-tag v-if="row.multi" size="small" type="info">{{ t('keys.multi') }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="description" :label="t('keys.colDescription')" min-width="160" show-overflow-tooltip>
        <template #default="{ row }">{{ row.description || '—' }}</template>
      </el-table-column>
      <el-table-column :label="t('keys.colValues')" min-width="200">
        <template #default="{ row }">
          <span v-if="!row.values?.length" class="values-empty">—</span>
          <span v-else class="values-cell">
            <el-tag v-for="v in shownValues(row)" :key="v" size="small" type="warning" effect="plain">{{ v }}</el-tag>
            <el-tooltip v-if="(row.values?.length ?? 0) > shownCount" :content="overflowValues(row)" placement="top">
              <span class="values-more">+{{ row.values.length - shownCount }}</span>
            </el-tooltip>
          </span>
        </template>
      </el-table-column>
      <el-table-column :label="t('keys.colActions')" width="120" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openEdit(row)">{{ t('common.edit') }}</el-button>
          <el-button link type="danger" @click="askDelete(row)">{{ t('common.delete') }}</el-button>
        </template>
      </el-table-column>
    </el-table>

    <FormDialog
      v-model:visible="dialogVisible"
      :title="editing ? t('keys.editTitleFmt', { name: editing.name }) : t('keys.createTitle')"
      :saving="saving"
      :submit-disabled="!canSave"
      @submit="save"
    >
      <el-form-item :label="t('keys.nameLabel')">
        <el-input v-model="form.name" :disabled="!!editing" :maxlength="100" :placeholder="t('keys.namePlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('keys.typeLabel')">
        <!-- Type reconfiguration is allowed (the server grandfathers existing data; only future writes follow the new type) -->
        <el-radio-group v-model="form.keyType">
          <el-radio value="enum">{{ t('editor.attrType.enum') }}</el-radio>
          <el-radio value="number">{{ t('editor.attrType.number') }}</el-radio>
        </el-radio-group>
        <div class="form-hint">{{ t(form.keyType === 'enum' ? 'keys.typeEnumHint' : 'keys.typeNumberHint') }}</div>
      </el-form-item>
      <el-form-item :label="t('keys.multiLabel')">
        <el-switch v-model="form.multi" />
        <div class="form-hint">{{ t('keys.multiHint') }}</div>
      </el-form-item>
      <el-form-item :label="t('keys.descLabel')">
        <el-input v-model="form.description" type="textarea" :rows="2" :maxlength="512" show-word-limit />
      </el-form-item>
    </FormDialog>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'
import { useApiClient } from '../../api/client'
import { deleteKey, setKey, type KeyType } from '../../api/keys'
import type { KeyView } from '../../types'
import { t } from '../../i18n'
import { toastSuccess } from '../../toast'
import { run } from './caps'
import FormDialog from '../FormDialog.vue'

const props = defineProps<{ keys: KeyView[] }>()
const emit = defineEmits<{ changed: [] }>()
const api = useApiClient()

// Values stay legible: show the first few in the cell, the rest behind a +N tooltip
const shownCount = 6
const shownValues = (row: KeyView): string[] => (row.values ?? []).slice(0, shownCount)
const overflowValues = (row: KeyView): string => (row.values ?? []).slice(shownCount).join(', ')

const dialogVisible = ref(false)
const saving = ref(false)
const editing = ref<KeyView | null>(null)
const form = ref<{ name: string; keyType: KeyType; multi: boolean; description: string }>({
  name: '',
  keyType: 'enum',
  multi: false,
  description: '',
})

// The name is the primary key: create validates its presence; edits save only a real
// reconfiguration (type/multi/description) — name and registry membership never change
const canSave = computed(() => {
  if (editing.value) {
    const e = editing.value
    return form.value.keyType !== e.type || form.value.multi !== e.multi || form.value.description !== e.description
  }
  return form.value.name.trim() !== ''
})

function openCreate(): void {
  editing.value = null
  form.value = { name: '', keyType: 'enum', multi: false, description: '' }
  dialogVisible.value = true
}

function openEdit(row: KeyView): void {
  editing.value = row
  form.value = { name: row.name, keyType: row.type, multi: row.multi, description: row.description }
  dialogVisible.value = true
}

async function save(): Promise<void> {
  if (!canSave.value || saving.value) return
  saving.value = true
  try {
    // run() toasts the failure and returns undefined: the dialog stays open with the draft intact
    const out = await run(() =>
      setKey(api, {
        name: form.value.name.trim(),
        key_type: form.value.keyType,
        multi: form.value.multi,
        description: form.value.description,
      }),
    )
    if (out === undefined) return
    toastSuccess(t('keys.saved'))
    dialogVisible.value = false
    emit('changed')
  } finally {
    saving.value = false
  }
}

async function askDelete(row: KeyView): Promise<void> {
  try {
    await ElMessageBox.confirm(t('keys.deleteConfirmFmt', { name: row.name }), t('keys.deleteTitle'), {
      type: 'warning',
      confirmButtonText: t('common.delete'),
      cancelButtonText: t('common.cancel'),
    })
  } catch {
    return
  }
  const out = await run(() => deleteKey(api, row.name))
  if (out === undefined) return
  toastSuccess(t('keys.deletedFmt', { count: out.memories_updated }))
  emit('changed')
}
</script>

<style scoped>
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.values-cell {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}
.values-more {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.values-empty {
  color: var(--el-text-color-placeholder);
}
.form-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  width: 100%;
  margin-top: 2px;
}
</style>
