<template>
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.settingsTitle') }}</span>
        <!-- Same control as the memory editor: single-icon edit/preview toggle -->
        <MarkdownModeToggle v-model="mode" />
      </div>
    </template>
    <el-form label-position="top" @submit.prevent>
      <el-form-item>
        <template #label>
          <span class="field-label">
            {{ t('access.instructionsLabel') }}
            <el-tooltip :content="t('access.instructionsInfo')" placement="top">
              <el-icon class="am-info"><InfoFilled /></el-icon>
            </el-tooltip>
          </span>
        </template>
        <el-input
          v-if="mode === 'edit'"
          v-model="instructions"
          type="textarea"
          :rows="5"
          :placeholder="t('access.instructionsPlaceholder')"
        />
        <MarkdownView v-else :source="instructions" class="prompt-preview" />
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button :disabled="!hasText" :loading="clearing" @click="clear">
        {{ t('access.clearInstructions') }}
      </el-button>
      <el-button type="primary" :loading="saving" :disabled="!dirty" @click="save">
        {{ t('common.save') }}
      </el-button>
    </div>
    <!-- Read-only reference: what agents already receive before this text, shipped with the binary -->
    <el-collapse class="built-in">
      <el-collapse-item :title="t('access.instructionsBuiltIn')" name="built-in">
        <MarkdownView :source="defaultInstructions" class="prompt-preview" />
      </el-collapse-item>
    </el-collapse>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { InfoFilled } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import MarkdownView from '../MarkdownView.vue'
import MarkdownModeToggle from '../MarkdownModeToggle.vue'
import { run } from './caps'

const props = defineProps<{
  instructions: string
  /** The built-in prompt every caller receives first; shown read-only, never editable here */
  defaultInstructions: string
}>()

const emit = defineEmits<{
  /** Saved successfully (the parent refreshes settings so the save button settles back to disabled) */
  saved: []
}>()

// Local editing state, refreshed when the parent loads
const instructions = ref(props.instructions)
watch(
  () => props.instructions,
  (i) => {
    instructions.value = i
  },
)

const hasText = computed(() => instructions.value.trim().length > 0)
// The stored value is exactly what a save would send, so "no change" is a plain comparison
const dirty = computed(() => instructions.value !== props.instructions)

const mode = ref<'edit' | 'preview'>('edit')

const api = useApiClient()
const saving = ref(false)
const clearing = ref(false)

async function save(): Promise<void> {
  if (!dirty.value) return
  saving.value = true
  try {
    await run(() => api.put('/api/settings', { instructions: instructions.value }))
    toastSuccess(t('access.saved'))
    emit('saved')
  } finally {
    saving.value = false
  }
}

// Clearing only touches this setting and does not refetch the whole card (keeps an unsaved draft)
async function clear(): Promise<void> {
  clearing.value = true
  try {
    await run(() => api.put('/api/settings', { instructions: '' }))
    instructions.value = ''
    toastSuccess(t('access.saved'))
  } finally {
    clearing.value = false
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.field-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
/* Preview mirrors the textarea outline so the border doesn't jump when switching */
.prompt-preview {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  padding: 5px 11px;
  min-height: 110px;
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
.built-in {
  margin-top: 12px;
}
</style>
