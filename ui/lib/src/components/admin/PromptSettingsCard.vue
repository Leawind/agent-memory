<template>
  <el-card shadow="never">
    <template #header>{{ t('access.settingsTitle') }}</template>
    <!-- 淡色即内置默认：透明度纯由内容推导，所见即生效 -->
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.instructionsLabel')">
        <el-input
          v-model="instructions"
          type="textarea"
          :rows="5"
          :placeholder="t('access.instructionsPlaceholder')"
          :class="{ 'is-default': isDefault }"
        />
        <div class="field-hint">{{ t('access.instructionsHint') }}</div>
      </el-form-item>
      <el-form-item :label="t('access.conventionsLabel')">
        <el-input v-model="conventions" type="textarea" :rows="4" :placeholder="t('access.conventionsPlaceholder')" />
        <div class="field-hint">{{ t('access.conventionsHint') }}</div>
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button :disabled="isDefault" :loading="resetting" @click="resetToDefault">
        {{ t('access.resetToDefault') }}
      </el-button>
      <el-button type="primary" :loading="saving" @click="save">
        {{ t('common.save') }}
      </el-button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { run } from './caps'

const props = defineProps<{
  instructions: string
  conventions: string
  /** 服务端当前生效的内置默认提示词（保存归一化与恢复默认的基准） */
  defaultInstructions: string
}>()

// 本地编辑态，随父层 load 刷新
const instructions = ref(props.instructions)
const conventions = ref(props.conventions)
watch(
  () => [props.instructions, props.conventions],
  ([i, c]) => {
    instructions.value = i
    conventions.value = c
  },
)

// 内容与内置默认一致即视为"未自定义"（淡色显示，保存归一为空串）
const isDefault = computed(() => instructions.value.trim() === props.defaultInstructions.trim())

const api = useApiClient()
const saving = ref(false)
const resetting = ref(false)

// 与默认一致的值提交空串：服务端归一为未设置，继续跟随内置默认而非冻结快照
function normalizedInstructions(): string {
  return isDefault.value ? '' : instructions.value
}

async function save(): Promise<void> {
  saving.value = true
  try {
    await run(() =>
      api.put('/api/settings', {
        instructions: normalizedInstructions(),
        conventions: conventions.value,
      }),
    )
    toastSuccess(t('access.saved'))
  } finally {
    saving.value = false
  }
}

// 恢复默认：只动 instructions，不整卡重拉（避免丢未保存的 conventions 草稿）
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
.field-hint {
  width: 100%;
  margin-top: 2px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
}
.is-default :deep(textarea) {
  opacity: 0.9;
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
