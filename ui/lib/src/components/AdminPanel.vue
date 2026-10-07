<template>
  <div ref="rootRef" class="am-panel admin-panel">
    <el-config-provider :locale="elementPlusLocale">
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
        <!-- Auth: toggle (right of the title row) + anonymous access + identity table in one card -->
        <AuthCard
          :auth-required="authRequired"
          :has-admin-identity="hasAdminIdentity"
          :anonymous-permissions="anonymousPermissions"
          :identities="identities"
          :compact="compact"
          @changed="load"
          @saved="load"
          @create="formDialog?.openCreate()"
          @edit="formDialog?.openEdit($event)"
          @delete="askDelete"
          @reset-token="askResetToken"
        />

        <!-- Semantic search config and vector coverage share a card (the former standalone coverage card at the bottom was merged in) -->
        <EmbeddingSettingsCard
          :embedding-enabled="embeddingEnabled"
          :embedding-base-url="embeddingBaseUrl"
          :embedding-model="embeddingModel"
          :embedding-api-key="embeddingApiKey"
          :stats="stats"
          :compact="compact"
          @changed="load"
        />

        <PromptSettingsCard :instructions="instructions" :default-instructions="defaultInstructions" @saved="load" />

        <BackupCard />

        <DoctorCard />
      </template>

      <!-- Create / edit identity; the parent refreshes the list on success -->
      <IdentityFormDialog ref="formDialog" :compact="compact" @saved="load" @created="onIdentityCreated" />

      <!-- After create / reset: the token is shown in plaintext only once -->
      <TokenOnceDialog ref="tokenDialog" :compact="compact" />
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { ElMessageBox } from 'element-plus'
import { InfoFilled } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { useApiClient } from '../api/client'
import { useContainerWidth } from '../composables/useContainerWidth'
import { toastSuccess } from '../toast'
import type { WhoAmI } from '../types'
import { run, type IdentityRow } from './admin/caps'
import AuthCard from './admin/AuthCard.vue'
import IdentityFormDialog from './admin/IdentityFormDialog.vue'
import TokenOnceDialog from './admin/TokenOnceDialog.vue'
import EmbeddingSettingsCard from './admin/EmbeddingSettingsCard.vue'
import PromptSettingsCard from './admin/PromptSettingsCard.vue'
import BackupCard from './admin/BackupCard.vue'
import DoctorCard from './admin/DoctorCard.vue'

const props = withDefaults(
  defineProps<{
    /** Caller identity summary: determines whether the panel is usable (admin capability) and the open-mode notice; pass null when the host has no identity wiring */
    who?: WhoAmI | null
    /** Hide the title/subtitle area (when embedded in a host that already has a page title, only the content cards are wanted) */
    showHeader?: boolean
    /** Override the default title */
    title?: string
    /** Override the default subtitle */
    subtitle?: string
  }>(),
  { who: null, showHeader: true },
)

const api = useApiClient()

const identities = ref<IdentityRow[]>([])
const instructions = ref('')
const defaultInstructions = ref('')
const authRequired = ref(false)
const anonymousPermissions = ref<Record<string, boolean> | null>(null)
const embeddingEnabled = ref(false)
const embeddingBaseUrl = ref('')
const embeddingModel = ref('')
const embeddingApiKey = ref('')

const isAdmin = computed(() => !!props.who && props.who.permissions?.admin === true)

// Semantic search coverage: stats are fetched together with load (/api/stats only needs read,
// but the backfill button is admin-only)
const stats = ref<import('../types').StatsInfo | null>(null)

const rootRef = ref<HTMLElement | null>(null)
// In compact mode (<960px) the created-at column is hidden and dialogs widen to 96%
const { compact } = useContainerWidth(rootRef)

async function load(): Promise<void> {
  await run(async () => {
    const [list, settings, statsResp] = await Promise.all([
      api.get<{ identities?: IdentityRow[] }>('/api/identities'),
      api.get<{
        instructions?: string | null
        auth_required?: boolean
        anonymous_permissions?: Record<string, boolean> | null
        embedding_enabled?: boolean
        embedding_base_url?: string | null
        embedding_model?: string | null
        embedding_api_key?: string | null
        default_instructions?: string | null
      }>('/api/settings'),
      api.get<import('../types').StatsInfo>('/api/stats'),
    ])
    identities.value = Array.isArray(list?.identities) ? list.identities : []
    // When unset, prefill the built-in default directly (what you see is what applies); the card derives its faded state from the content
    instructions.value = settings?.instructions ?? settings?.default_instructions ?? ''
    defaultInstructions.value = settings?.default_instructions ?? ''
    authRequired.value = settings?.auth_required === true
    anonymousPermissions.value = settings?.anonymous_permissions ?? null
    embeddingEnabled.value = settings?.embedding_enabled === true
    embeddingBaseUrl.value = settings?.embedding_base_url ?? ''
    embeddingModel.value = settings?.embedding_model ?? ''
    embeddingApiKey.value = settings?.embedding_api_key ?? ''
    stats.value = statsResp ?? null
  })
}

onMounted(() => {
  // The server responds 403 without admin permission; skip loading silently (the page notice is enough)
  if (isAdmin.value) void load()
})

// Mounting usually happens before whoami returns (who starts as null): load once more once the admin capability is ready
watch(isAdmin, (granted) => {
  if (granted) void load()
})

// A permanently mounted panel cannot sense visibility itself: the host calls refresh when switching back to this panel to pull the latest data
defineExpose({
  refresh: () => {
    if (isAdmin.value) void run(load)
  },
})

// ---- Identity row actions ----
const formDialog = ref<InstanceType<typeof IdentityFormDialog> | null>(null)
const tokenDialog = ref<InstanceType<typeof TokenOnceDialog> | null>(null)

const hasAdminIdentity = computed(() => identities.value.some((row) => row.permissions?.admin === true))

// Creation succeeded: show the one-time token dialog and refresh the list
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
