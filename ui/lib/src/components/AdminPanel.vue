<template>
  <div ref="rootRef" class="am-panel admin-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div class="am-heading">
        <h2 class="am-panel-title">{{ props.title ?? t('access.title') }}</h2>
        <el-tooltip :content="props.subtitle ?? t('access.subtitle')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
      </div>
    </div>

    <el-alert v-if="!who" type="info" :title="t('access.needAdmin')" :closable="false" />
    <el-alert v-else-if="who.mode === 'open'" type="warning" :title="t('access.openMode')" :closable="false" />
    <el-alert v-else-if="!isAdmin" type="info" :title="t('access.needAdmin')" :closable="false" />

    <template v-if="who && isAdmin">
      <AuthSwitchCard :auth-required="authRequired" :has-admin-identity="hasAdminIdentity" />

      <IdentityTableCard
        :identities="identities"
        :compact="compact"
        @create="formDialog?.openCreate()"
        @edit="formDialog?.openEdit($event)"
        @delete="askDelete"
        @reset-token="askResetToken"
      />

      <!-- 语义搜索配置 + 向量覆盖率同卡（原底部独立覆盖率卡已并入） -->
      <EmbeddingSettingsCard
        :embedding-enabled="embeddingEnabled"
        :embedding-base-url="embeddingBaseUrl"
        :embedding-model="embeddingModel"
        :embedding-api-key="embeddingApiKey"
        :stats="stats"
        :compact="compact"
        @changed="load"
      />

      <PromptSettingsCard
        :instructions="instructions"
        :conventions="conventions"
        :default-instructions="defaultInstructions"
      />

      <BackupCard />

      <DoctorCard />
    </template>

    <!-- 新建 / 编辑身份；成功后的列表刷新在父层 -->
    <IdentityFormDialog ref="formDialog" :compact="compact" @saved="load" @created="onIdentityCreated" />

    <!-- 新建 / 重置成功：token 明文仅此一次展示 -->
    <TokenOnceDialog ref="tokenDialog" :compact="compact" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { InfoFilled } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { useApiClient } from '../api/client'
import { useContainerWidth } from '../composables/useContainerWidth'
import { toastSuccess } from '../toast'
import type { WhoAmI } from '../types'
import { run, type IdentityRow } from './admin/caps'
import AuthSwitchCard from './admin/AuthSwitchCard.vue'
import IdentityTableCard from './admin/IdentityTableCard.vue'
import IdentityFormDialog from './admin/IdentityFormDialog.vue'
import TokenOnceDialog from './admin/TokenOnceDialog.vue'
import EmbeddingSettingsCard from './admin/EmbeddingSettingsCard.vue'
import PromptSettingsCard from './admin/PromptSettingsCard.vue'
import BackupCard from './admin/BackupCard.vue'
import DoctorCard from './admin/DoctorCard.vue'

const props = withDefaults(
  defineProps<{
    /** 调用者身份摘要：决定面板是否可用（admin 能力）与开放模式提示；嵌入宿主未接入时传 null */
    who?: WhoAmI | null
    /** 隐藏标题/副标题区（嵌入宿主已有页面标题时只要内容卡） */
    showHeader?: boolean
    /** 覆盖默认标题 */
    title?: string
    /** 覆盖默认副标题 */
    subtitle?: string
  }>(),
  { who: null, showHeader: true },
)

const api = useApiClient()

const identities = ref<IdentityRow[]>([])
const instructions = ref('')
const conventions = ref('')
const defaultInstructions = ref('')
const authRequired = ref(false)
const embeddingEnabled = ref(false)
const embeddingBaseUrl = ref('')
const embeddingModel = ref('')
const embeddingApiKey = ref('')

const isAdmin = computed(() => !!props.who && props.who.permissions?.admin === true)

// 语义搜索覆盖率：统计随 load 一并拉取（/api/stats 只需 read，但补跑按钮仅 admin 可用）
const stats = ref<import('../types').StatsInfo | null>(null)

const rootRef = ref<HTMLElement | null>(null)
// compact（<960px）时创建时间列收起、对话框加宽到 96%
const { compact } = useContainerWidth(rootRef)

async function load(): Promise<void> {
  await run(async () => {
    const [list, settings, statsResp] = await Promise.all([
      api.get<{ identities?: IdentityRow[] }>('/api/identities'),
      api.get<{
        instructions?: string | null
        conventions?: string | null
        auth_required?: boolean
        embedding_enabled?: boolean
        embedding_base_url?: string | null
        embedding_model?: string | null
        embedding_api_key?: string | null
        default_instructions?: string | null
      }>('/api/settings'),
      api.get<import('../types').StatsInfo>('/api/stats'),
    ])
    identities.value = Array.isArray(list?.identities) ? list.identities : []
    instructions.value = settings?.instructions ?? ''
    conventions.value = settings?.conventions ?? ''
    defaultInstructions.value = settings?.default_instructions ?? ''
    authRequired.value = settings?.auth_required === true
    embeddingEnabled.value = settings?.embedding_enabled === true
    embeddingBaseUrl.value = settings?.embedding_base_url ?? ''
    embeddingModel.value = settings?.embedding_model ?? ''
    embeddingApiKey.value = settings?.embedding_api_key ?? ''
    stats.value = statsResp ?? null
  })
}

onMounted(() => {
  // 无 admin 权限时服务端会 403，静默跳过加载（页面提示已足够）
  if (isAdmin.value) void load()
})

// 挂载通常早于 whoami 返回（who 初始为 null）：admin 能力就绪后补一次加载
watch(isAdmin, (granted) => {
  if (granted) void load()
})

// 面板常驻挂载时无法自行感知可见性：宿主切回此面板时调 refresh 拉最新数据
defineExpose({
  refresh: () => {
    if (isAdmin.value) void run(load)
  },
})

// ---- 身份行操作 ----
const formDialog = ref<InstanceType<typeof IdentityFormDialog> | null>(null)
const tokenDialog = ref<InstanceType<typeof TokenOnceDialog> | null>(null)

const hasAdminIdentity = computed(() => identities.value.some((row) => row.permissions?.admin === true))

// 新建成功：弹一次性 token 展示并刷新列表
function onIdentityCreated(payload: { name: string; token: string }): void {
  tokenDialog.value?.show(t('access.createTitle'), payload.name, payload.token)
  void load()
}

async function askDelete(row: IdentityRow): Promise<void> {
  try {
    await ElMessageBox.confirm(t('access.deleteConfirm', { name: row.name }), t('access.deleteTitle'), {
      type: 'warning',
      confirmButtonText: t('common.delete'),
      cancelButtonText: t('common.cancel'),
    })
  } catch {
    return
  }
  const ok = await run(() => api.del(`/api/identities/${encodeURIComponent(row.name)}`))
  if (ok === undefined) return
  toastSuccess(t('access.deleted'))
  await load()
}

async function askResetToken(row: IdentityRow): Promise<void> {
  try {
    await ElMessageBox.confirm(t('access.resetConfirm', { name: row.name }), t('access.resetTitle'), {
      type: 'warning',
      confirmButtonText: t('access.resetToken'),
      cancelButtonText: t('common.cancel'),
    })
  } catch {
    return
  }
  const reset = await run(() =>
    api.post<{ token?: string }>(`/api/identities/${encodeURIComponent(row.name)}/token-reset`, {}),
  )
  if (reset === undefined) return
  if (reset?.token) tokenDialog.value?.show(t('access.resetTitle'), row.name, reset.token)
  await load()
}
</script>

<style scoped>
.admin-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
