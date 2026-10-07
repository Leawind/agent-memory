<template>
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.settingsTitle') }}</span>
        <!-- Same control as the memory editor: single-icon edit/preview toggle -->
        <MarkdownModeToggle v-model="mode" />
      </div>
    </template>
    <!-- Dimmed text means the built-in default: opacity derives purely from content, so what you see is what applies; usage notes live in the info icon next to the label -->
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
          :class="{ 'is-default': isDefault }"
        />
        <MarkdownView v-else :source="instructions" class="prompt-preview" :class="{ 'is-default': isDefault }" />
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button :disabled="isDefault" :loading="resetting" @click="resetToDefault">
        {{ t('access.resetToDefault') }}
      </el-button>
      <el-button type="primary" :loading="saving" :disabled="!dirty" @click="save">
        {{ t('common.save') }}
      </el-button>
    </div>
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
  /** The server's currently effective built-in default prompt (baseline for save normalization and restore-to-default) */
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

// Content identical to the built-in default counts as "not customized" (shown faded, saved as an empty string)
const isDefault = computed(() => instructions.value.trim() === props.defaultInstructions.trim())

const mode = ref<'edit' | 'preview'>('edit')

const api = useApiClient()
const saving = ref(false)
const resetting = ref(false)

// A value identical to the default is submitted as an empty string: the server normalizes it to
// unset so it keeps following the built-in default instead of freezing a snapshot
function normalizedInstructions(): string {
  return isDefault.value ? '' : instructions.value
}

// The baseline is the server's stored value, passed through the same normalization a save
// applies (stored value is '' when following the default). A save would send exactly
// normalizedInstructions(), so "no change" = it equals the normalized baseline.
const baseline = computed(() =>
  props.instructions.trim() === props.defaultInstructions.trim() ? '' : props.instructions,
)
const dirty = computed(() => normalizedInstructions() !== baseline.value)

async function save(): Promise<void> {
  if (!dirty.value) return
  saving.value = true
  try {
    await run(() => api.put('/api/settings', { instructions: normalizedInstructions() }))
    toastSuccess(t('access.saved'))
    emit('saved')
  } finally {
    saving.value = false
  }
}

// Restore default: only touch instructions, do not refetch the whole card (keeps an unsaved draft)
async function resetToDefault(): Promise<void> {
  resetting.value = true
  try {
    await run(() => api.put('/api/settings', { instructions: '' }))
    instructions.value = props.defaultInstructions
    toastSuccess(t('access.saved'))
  } finally {
    resetting.value = false
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
.is-default :deep(textarea) {
  opacity: 0.9;
}
.is-default {
  opacity: 0.9;
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
