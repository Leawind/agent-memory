<template>
  <div ref="rootRef" class="am-panel">
    <el-config-provider :locale="elementPlusLocale">
      <div v-if="showHeader" class="am-panel-header">
        <div class="am-heading">
          <h2 class="am-panel-title">{{ props.title ?? t('tags.title') }}</h2>
          <el-tooltip :content="props.subtitle ?? t('tags.subtitle')" placement="top">
            <el-icon class="am-info"><InfoFilled /></el-icon>
          </el-tooltip>
        </div>
        <el-button type="primary" :icon="Plus" @click="openCreate">{{ t('tags.create') }}</el-button>
      </div>

      <!-- Regex filter on tag names (applied server-side; invalid regexes are reported by the server and toasted) -->
      <el-input
        v-model="filter"
        class="am-tag-filter"
        :placeholder="t('tags.filterPlaceholder')"
        clearable
        :prefix-icon="Search"
        @input="applyFilterDebounced"
        @clear="applyFilterDebounced"
      />

      <el-table :data="rows" v-loading="loading">
        <el-table-column prop="name" :label="t('tags.colName')" min-width="140" sortable>
          <template #default="{ row }">
            <el-tag>{{ row.name }}</el-tag>
            <el-tooltip :content="t('tags.reservedHint')" placement="top" :enterable="false">
              <el-tag v-if="row.reserved" size="small" type="warning" class="am-reserved">
                {{ t('tags.reserved') }}
              </el-tag>
            </el-tooltip>
          </template>
        </el-table-column>
        <el-table-column prop="description" :label="t('tags.colDescription')" min-width="150" show-overflow-tooltip>
          <template #default="{ row }">{{ row.description || '—' }}</template>
        </el-table-column>
        <el-table-column prop="count" :label="t('tags.colMemoryCount')" width="90" sortable />
        <el-table-column :label="t('memories.colActions')" width="100" fixed="right">
          <template #default="{ row }">
            <!-- Reserved tags can be neither renamed nor deleted server-side; hide the dead controls -->
            <template v-if="!row.reserved">
              <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
                <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="openEdit(row)" />
              </el-tooltip>
              <el-tooltip :content="t('common.delete')" placement="top" :enterable="false">
                <el-button
                  link
                  type="danger"
                  :icon="Delete"
                  :aria-label="t('common.delete')"
                  @click="openDelete(row)"
                />
              </el-tooltip>
            </template>
          </template>
        </el-table-column>
      </el-table>

      <FormDialog
        v-model:visible="dialogVisible"
        :title="form.oldName ? t('tags.editTitle') : t('tags.createTitle')"
        :width="narrow ? '96%' : '480px'"
        :saving="saving"
        @submit="save"
      >
        <!-- Prefill the current name when editing: small tweaks are direct edits, no need to retype the full name; the server treats an unchanged-name submit as a no-rename -->
        <el-form-item :label="t('tags.nameLabel')">
          <el-input v-model="form.name" maxlength="100" show-word-limit :placeholder="t('tags.namePlaceholder')" />
        </el-form-item>
        <el-form-item :label="t('tags.descLabel')">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="3"
            maxlength="512"
            show-word-limit
            :placeholder="t('tags.descPlaceholder')"
          />
        </el-form-item>
      </FormDialog>

      <!-- Delete-mode choice: the rich structure (tag name / count) can't be interpolated as one sentence, so it's split into three fixed-order segments -->
      <FormDialog
        v-model:visible="deleteVisible"
        :title="t('tags.deleteTitle')"
        :width="narrow ? '96%' : '480px'"
        :saving="saving"
        submit-type="danger"
        :submit-text="t('common.delete')"
        @submit="doDelete"
      >
        <p class="delete-body">
          {{ t('tags.deleteBefore') }}
          <el-tag>{{ target?.name }}</el-tag>
          {{ t('tags.deleteMiddle') }}
          <b>{{ target?.count }}</b>
          {{ t('tags.deleteAfter') }}
        </p>
        <el-radio-group v-model="deleteMode">
          <el-radio value="detach">{{ t('tags.detach') }}</el-radio>
          <el-radio value="purge">{{ t('tags.purge') }}</el-radio>
        </el-radio-group>
      </FormDialog>
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { ElMessageBox } from 'element-plus'
import { toastError, toastSuccess } from '../toast'
import { Delete, Edit, InfoFilled, Plus, Search } from '@element-plus/icons-vue'
import FormDialog from './FormDialog.vue'
import { t } from '../i18n'
import { useTags } from '../composables/useTags'
import { useContainerWidth } from '../composables/useContainerWidth'
import type { TagView } from '../types'

const props = withDefaults(
  defineProps<{
    /** Hide the title/subtitle area (when embedded in a host that already has a page title, only the table is wanted) */
    showHeader?: boolean
    /** Override the default title */
    title?: string
    /** Override the default subtitle */
    subtitle?: string
  }>(),
  { showHeader: true },
)

const { rows, loading, filter, reload, create, rename, remove } = useTags()

// Regex filter input -> debounced refetch; invalid-regex errors are toasted via reload's exception path
let filterTimer: ReturnType<typeof setTimeout> | null = null
function applyFilterDebounced() {
  if (filterTimer) clearTimeout(filterTimer)
  filterTimer = setTimeout(() => {
    filterTimer = null
    reload().catch((e) => toastError(e instanceof Error ? e.message : String(e)))
  }, 300)
}

const rootRef = ref<HTMLElement | null>(null)
// In narrow mode (<720px) dialogs narrow; all table columns are flexible-width so no columns are lost in narrow containers
const { narrow } = useContainerWidth(rootRef)

const saving = ref(false)
const dialogVisible = ref(false)
const form = reactive({ oldName: null as string | null, name: '', description: '' })
const deleteVisible = ref(false)
const deleteMode = ref<'detach' | 'purge'>('detach')
const target = ref<TagView | null>(null)

function openCreate() {
  Object.assign(form, { oldName: null, name: '', description: '' })
  dialogVisible.value = true
}

function openEdit(row: TagView) {
  Object.assign(form, { oldName: row.name, name: row.name, description: row.description ?? '' })
  dialogVisible.value = true
}

async function save() {
  saving.value = true
  try {
    if (form.oldName) {
      // When the name equals the original (or is blank) the api layer omits new_name, i.e. only the description is updated
      await rename(form.oldName, form.name, form.description)
      toastSuccess(t('tags.saved'))
    } else {
      await create(form.name, form.description)
      toastSuccess(t('tags.created'))
    }
    dialogVisible.value = false
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

function openDelete(row: TagView) {
  target.value = row
  deleteMode.value = 'detach'
  deleteVisible.value = true
}

async function doDelete() {
  if (deleteMode.value === 'purge') {
    try {
      await ElMessageBox.confirm(
        t('tags.purgeConfirm', { name: target.value?.name, count: target.value?.count ?? 0 }),
        t('tags.purgeConfirmTitle'),
        { type: 'error', confirmButtonText: t('tags.purgeButton') },
      )
    } catch {
      return
    }
  }
  if (!target.value) return
  saving.value = true
  try {
    await remove(target.value.name, deleteMode.value)
    toastSuccess(t('tags.deleted'))
    deleteVisible.value = false
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

// A permanently mounted panel cannot sense visibility itself: the host calls refresh when switching back to this panel to pull the latest data
defineExpose({
  refresh: () => reload().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e))),
})
</script>

<style scoped>
.am-tag-filter {
  margin-bottom: 12px;
  width: 100%;
}
.am-reserved {
  margin-left: 6px;
}
.delete-body {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
  margin: 0 0 16px;
}
</style>
