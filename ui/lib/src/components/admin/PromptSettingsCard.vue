<template>
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.settingsTitle') }}</span>
        <!-- 与记忆编辑器同款：编辑/预览切换，两个槽位联动 -->
        <el-radio-group v-model="mode" size="small">
          <el-tooltip :content="t('editor.tabEdit')" placement="top" :enterable="false">
            <el-radio-button value="edit" :aria-label="t('editor.tabEdit')">
              <el-icon><Edit /></el-icon>
            </el-radio-button>
          </el-tooltip>
          <el-tooltip :content="t('editor.tabPreview')" placement="top" :enterable="false">
            <el-radio-button value="preview" :aria-label="t('editor.tabPreview')">
              <el-icon><View /></el-icon>
            </el-radio-button>
          </el-tooltip>
        </el-radio-group>
      </div>
    </template>
    <!-- 淡色即内置默认：透明度纯由内容推导，所见即生效；用法说明收进标签旁的 ⓘ -->
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
      <el-form-item>
        <template #label>
          <span class="field-label">
            {{ t('access.conventionsLabel') }}
            <el-tooltip :content="t('access.conventionsInfo')" placement="top">
              <el-icon class="am-info"><InfoFilled /></el-icon>
            </el-tooltip>
          </span>
        </template>
        <el-input
          v-if="mode === 'edit'"
          v-model="conventions"
          type="textarea"
          :rows="4"
          :placeholder="t('access.conventionsPlaceholder')"
        />
        <MarkdownView v-else :source="conventions" class="prompt-preview" />
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
import { Edit, InfoFilled, View } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import MarkdownView from '../MarkdownView.vue'
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

const mode = ref<'edit' | 'preview'>('edit')

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
/* 预览态镜像文本框轮廓，切换时边框不跳 */
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
