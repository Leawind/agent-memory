<template>
  <!-- 匿名访问：鉴权开启后，无 token 请求按这里勾选的能力解析为匿名身份；
       未设置（开关关）时匿名被整体拒绝。鉴权开关未开时配置不生效（开关禁用）。 -->
  <el-card shadow="never">
    <div class="anon-row">
      <div class="anon-text">
        <span class="anon-label">{{ t('access.anonTitle') }}</span>
        <span class="anon-hint">{{ t('access.anonHint') }}</span>
      </div>
      <el-tooltip :disabled="authRequired" :content="t('access.anonDisabled')" placement="top">
        <el-switch v-model="anonOn" :disabled="!authRequired" />
      </el-tooltip>
    </div>
    <template v-if="anonOn">
      <el-divider />
      <CapsEditor v-model:caps="caps" v-model:preset="preset" />
      <div class="anon-actions">
        <el-button type="primary" :loading="saving" :disabled="!dirty || !hasAnyCap" @click="save">
          {{ t('common.save') }}
        </el-button>
      </div>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { CAPS, emptyCaps, isEmptyCaps, run, type PresetKey } from './caps'
import CapsEditor from './CapsEditor.vue'

const props = defineProps<{
  /** 服务器当前的鉴权开关状态：关闭时匿名能力不生效（开关禁用并提示） */
  authRequired: boolean
  /** 服务器当前配置的匿名能力集（全键对象）；null = 未设置（匿名被拒绝） */
  anonymousPermissions: Record<string, boolean> | null
}>()

const emit = defineEmits<{
  /** 保存成功（父层刷新设置） */
  saved: []
}>()

const api = useApiClient()

const caps = reactive<Record<string, boolean>>(emptyCaps())
const preset = ref<PresetKey>('custom')
const anonOn = ref(false)
const saving = ref(false)

// 服务器数据到达/刷新后重建本地草稿（保存成功后的 load 也会走回这里）
watch(
  () => props.anonymousPermissions,
  (v) => {
    for (const c of CAPS) caps[c.key] = v?.[c.key] === true
    preset.value = 'custom'
    anonOn.value = !isEmptyCaps(v)
  },
  { immediate: true },
)

const hasAnyCap = computed(() => !isEmptyCaps(caps))

// 关闭态：与服务器不一致（服务器有配置）才需要保存；
// 开启态：服务器无配置，或勾选与服务器不同
const dirty = computed(() =>
  anonOn.value
    ? props.anonymousPermissions === null ||
      CAPS.some((c) => props.anonymousPermissions?.[c.key] !== (caps[c.key] === true))
    : props.anonymousPermissions !== null,
)

async function save(): Promise<void> {
  saving.value = true
  try {
    // 关闭 = 清除配置（匿名被整体拒绝）；开启 = 保存勾选的全键对象
    const body = anonOn.value ? { anonymous_permissions: { ...caps } } : { anonymous_permissions: null }
    const ok = await run(() => api.put('/api/settings', body), t('access.anonSaved'))
    if (ok === undefined) return
    emit('saved')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.anon-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.anon-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.anon-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.anon-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.anon-actions {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
