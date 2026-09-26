<template>
  <el-card shadow="never">
    <template #header>{{ t('access.settingsTitle') }}</template>
    <!-- 留空即默认：行为说明放在各自字段的提示行里，紧邻输入框 -->
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.instructionsLabel')">
        <el-input v-model="instructions" type="textarea" :rows="5" :placeholder="t('access.instructionsPlaceholder')" />
        <div class="field-hint">{{ t('access.instructionsHint') }}</div>
        <details v-if="instructions" class="default-view">
          <summary>{{ t('access.viewDefault') }}</summary>
          <pre class="default-text">{{ defaultInstructions }}</pre>
        </details>
      </el-form-item>
      <el-form-item :label="t('access.conventionsLabel')">
        <el-input v-model="conventions" type="textarea" :rows="4" :placeholder="t('access.conventionsPlaceholder')" />
        <div class="field-hint">{{ t('access.conventionsHint') }}</div>
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button type="primary" :loading="saving" @click="save">
        {{ t('common.save') }}
      </el-button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { run } from './caps'

const props = defineProps<{
  instructions: string
  conventions: string
  /** 服务端当前生效的内置默认提示词（查看用） */
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

const api = useApiClient()
const saving = ref(false)

// 设置：留空即默认（服务端对空值回退内置文案），保存时一并提交
async function save(): Promise<void> {
  saving.value = true
  try {
    await run(() =>
      api.put('/api/settings', {
        instructions: instructions.value,
        conventions: conventions.value,
      }),
    )
    toastSuccess(t('access.saved'))
  } finally {
    saving.value = false
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
.default-view {
  margin-top: 6px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.default-view summary {
  cursor: pointer;
  user-select: none;
}
.default-text {
  margin: 6px 0 0;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-fill-color-light);
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 12px;
  line-height: 1.6;
  max-height: 220px;
  overflow: auto;
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
