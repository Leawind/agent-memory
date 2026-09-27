<template>
  <!-- 鉴权开关：鉴权边界由显式开关决定，与是否已创建身份无关 -->
  <el-card shadow="never">
    <div class="auth-row">
      <div class="auth-text">
        <span class="auth-label">{{ t('access.authTitle') }}</span>
        <span class="auth-hint">{{ t('access.authHint') }}</span>
      </div>
      <el-tooltip :disabled="hasAdminIdentity" :content="t('access.enableBlocked')" placement="top">
        <el-switch
          v-model="authOn"
          :before-change="confirmAuthToggle"
          :loading="togglingAuth"
          :disabled="!hasAdminIdentity"
        />
      </el-tooltip>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { useMemoryConfig } from '../../config'
import { toastError } from '../../toast'
import { run } from './caps'

const props = defineProps<{
  /** 服务器当前的鉴权开关状态（随父层 load 刷新） */
  authRequired: boolean
  /** 库里是否存在 admin 身份（开启的前置条件） */
  hasAdminIdentity: boolean
}>()

const authOn = ref(props.authRequired)
watch(
  () => props.authRequired,
  (v) => {
    authOn.value = v
  },
)

const togglingAuth = ref(false)
const api = useApiClient()
const config = useMemoryConfig()

const emit = defineEmits<{
  /** 开关状态实际改变（父层据此刷新 settings，匿名访问卡的可用性随之联动） */
  changed: [enabled: boolean]
}>()

// 先确认再变更（el-switch before-change）。
// 无 admin 身份时开关禁用（disabled + tooltip 说明），这里的预检只作兜底。
async function confirmAuthToggle(): Promise<boolean> {
  const target = !authOn.value
  // 预检兜底：开启要求库里已有 admin 身份（服务端守卫同规则）。
  // token 拿到手之前别翻开关。
  if (target && !props.hasAdminIdentity) {
    toastError(t('access.enableBlocked'))
    return false
  }
  try {
    await ElMessageBox.confirm(
      t(target ? 'access.authEnableConfirm' : 'access.authDisableConfirm'),
      t('access.authTitle'),
      {
        type: 'warning',
        confirmButtonText: t('common.save'),
        cancelButtonText: t('common.cancel'),
      },
    )
  } catch {
    return false
  }
  togglingAuth.value = true
  try {
    const saved = await run(() => api.put('/api/settings', { auth_required: target }))
    if (saved !== undefined) {
      config.onAuthChanged?.(target)
      emit('changed', target)
    }
    return true
  } finally {
    togglingAuth.value = false
  }
}
</script>

<style scoped>
.auth-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.auth-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.auth-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.auth-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
