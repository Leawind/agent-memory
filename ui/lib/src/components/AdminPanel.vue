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
      <el-alert v-else-if="!isAdmin" type="info" :title="t('access.needAdmin')" :closable="false" />

      <template v-if="who && isAdmin">
        <!-- Category nav on the left, one card per section on the right (DeepSeek-settings
             style). Sections stay mounted via v-show so each card's own state (probe results,
             doctor findings, in-progress drafts) survives switching back and forth. -->
        <div class="admin-layout" :class="{ 'is-compact': compact }">
          <nav class="admin-nav">
            <button
              v-for="cat in categories"
              :key="cat.id"
              type="button"
              class="nav-item"
              :class="{ 'is-active': activeSection === cat.id }"
              @click="activeSection = cat.id"
            >
              <el-icon :size="16"><component :is="cat.icon" /></el-icon>
              <span>{{ t(cat.labelKey) }}</span>
            </button>
          </nav>

          <div class="admin-sections">
            <!-- Open-mode notice: an identity/auth concern, so it lives in the access section
                 only — a global banner would reshuffle every tab's layout on the fixed-height
                 dialog. Stateless, so v-if (not v-show) is fine here. -->
            <el-alert
              v-if="who?.mode === 'open' && activeSection === 'access'"
              type="warning"
              :title="t('access.openMode')"
              :closable="false"
            />

            <!-- Auth: toggle (right of the title row) + anonymous access + identity table in one card -->
            <AuthCard
              v-show="activeSection === 'access'"
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
              v-show="activeSection === 'embedding'"
              :embedding-enabled="embeddingEnabled"
              :embedding-base-url="embeddingBaseUrl"
              :embedding-model="embeddingModel"
              :embedding-api-key="embeddingApiKey"
              :embedding-query-prefix="embeddingQueryPrefix"
              :embedding-passage-prefix="embeddingPassagePrefix"
              :stats="stats"
              :compact="compact"
              @changed="load"
            />

            <PromptSettingsCard
              v-show="activeSection === 'prompt'"
              :instructions="instructions"
              :default-instructions="defaultInstructions"
              @saved="load"
            />

            <BackupCard v-show="activeSection === 'backup'" />

            <DoctorCard v-show="activeSection === 'doctor'" :stats="stats" />
          </div>
        </div>
      </template>

      <!-- Create / edit identity; the parent refreshes the list on success -->
      <IdentityFormDialog ref="formDialog" :compact="compact" @saved="load" @created="onIdentityCreated" />

      <!-- After create / reset: the token is shown in plaintext only once -->
      <TokenOnceDialog ref="tokenDialog" :compact="compact" />
    </el-config-provider>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch, type Component } from 'vue'
import { elementPlusLocale } from '../i18n/elementPlus'
import { ElMessageBox } from 'element-plus'
import { ChatDotRound, FirstAidKit, FolderOpened, InfoFilled, Search, User } from '@element-plus/icons-vue'
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
const embeddingQueryPrefix = ref('')
const embeddingPassagePrefix = ref('')

const isAdmin = computed(() => !!props.who && props.who.permissions?.admin === true)

// Semantic search coverage: stats are fetched together with load (/api/stats only needs read,
// but the backfill button is admin-only)
const stats = ref<import('../types').StatsInfo | null>(null)

const rootRef = ref<HTMLElement | null>(null)
// In compact mode (<960px) the nav folds into a horizontal pill row and the created-at column is hidden
const { compact } = useContainerWidth(rootRef)

// ---- Category nav: one entry per settings card; the sections themselves stay mounted ----
type SectionId = 'access' | 'embedding' | 'prompt' | 'backup' | 'doctor'

const categories: Array<{ id: SectionId; labelKey: string; icon: Component }> = [
  { id: 'access', labelKey: 'access.navAccess', icon: User },
  { id: 'embedding', labelKey: 'access.navEmbedding', icon: Search },
  { id: 'prompt', labelKey: 'access.navPrompt', icon: ChatDotRound },
  { id: 'backup', labelKey: 'access.navBackup', icon: FolderOpened },
  { id: 'doctor', labelKey: 'access.navDoctor', icon: FirstAidKit },
]

const activeSection = ref<SectionId>('access')

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
        embedding_query_prefix?: string | null
        embedding_passage_prefix?: string | null
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
    embeddingQueryPrefix.value = settings?.embedding_query_prefix ?? ''
    embeddingPassagePrefix.value = settings?.embedding_passage_prefix ?? ''
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

/* Sidebar + sections: a DeepSeek-settings-style category nav on the left, the active card on
   the right. Cards keep their internal state across switches (v-show, not v-if). */
.admin-layout {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}
.admin-nav {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 172px;
  position: sticky;
  top: 12px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--el-text-color-regular);
  font: inherit;
  font-size: 14px;
  line-height: 1.4;
  text-align: left;
  cursor: pointer;
}
.nav-item:hover {
  background: var(--el-fill-color);
}
.nav-item.is-active {
  background: var(--el-fill-color-dark);
  color: var(--el-text-color-primary);
  font-weight: 600;
}
.admin-sections {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Compact containers: the nav folds into a wrapping pill row above the active section.
   flex-start (the row layout's top alignment) must become stretch here — in a column flex
   container it would shrink the sections to content width instead of filling the panel. */
.admin-layout.is-compact {
  flex-direction: column;
  align-items: stretch;
}
.is-compact .admin-nav {
  flex-direction: row;
  flex-wrap: wrap;
  width: auto;
  position: static;
}
.is-compact .nav-item {
  width: auto;
  padding: 7px 12px;
}
</style>
