<template>
  <!-- Auth card: toggle (right of the title row) + anonymous access + identity table merged into one card.
       The auth boundary is determined by the explicit switch, regardless of whether identities exist yet. -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span class="card-title">
          {{ t('access.authTitle') }}
          <el-tooltip :content="t('access.authHint')" placement="top">
            <el-icon class="am-info"><InfoFilled /></el-icon>
          </el-tooltip>
        </span>
        <el-tooltip :disabled="hasAdminIdentity" :content="t('access.enableBlocked')" placement="top">
          <el-switch
            v-model="authOn"
            :before-change="confirmAuthToggle"
            :loading="togglingAuth"
            :disabled="!hasAdminIdentity"
          />
        </el-tooltip>
      </div>
    </template>

    <div class="anon-row">
      <span class="anon-label">
        {{ t('access.anonTitle') }}
        <el-tooltip :content="t('access.anonHint')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
      </span>
      <el-tooltip :disabled="authRequired" :content="t('access.anonDisabled')" placement="top">
        <el-switch v-model="anonOn" :disabled="!authRequired" />
      </el-tooltip>
    </div>
    <template v-if="anonOn">
      <CapsEditor v-model:caps="caps" v-model:preset="preset" />
      <div class="anon-actions">
        <el-button type="primary" :loading="saving" :disabled="!dirty || !hasAnyCap" @click="save">
          {{ t('common.save') }}
        </el-button>
      </div>
    </template>

    <el-divider />

    <!-- Identities: the create entry sits right next to the table (right of the section row) -->
    <div class="card-header id-header">
      <span class="section-title">{{ t('access.identityCard') }}</span>
      <el-button type="primary" :icon="Plus" @click="emit('create')">
        {{ t('access.create') }}
      </el-button>
    </div>
    <!-- Takes no body space when empty (create entry lives in the section row); narrow containers: card list replaces the table -
         el-table's fixed action column crushes the capability tags at phone widths -->
    <div v-if="compact && identities.length > 0" class="id-cards">
      <div v-for="row in identities" :key="row.name" class="id-card">
        <div class="id-card-head">
          <span class="id-card-name">{{ row.name }}</span>
          <span class="id-card-actions">
            <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
              <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="emit('edit', row)" />
            </el-tooltip>
            <el-tooltip :content="t('access.resetToken')" placement="top" :enterable="false">
              <el-button
                link
                type="primary"
                :icon="RefreshRight"
                :aria-label="t('access.resetToken')"
                @click="emit('reset-token', row)"
              />
            </el-tooltip>
            <el-tooltip :content="t('common.delete')" placement="top" :enterable="false">
              <el-button
                link
                type="danger"
                :icon="Delete"
                :aria-label="t('common.delete')"
                @click="emit('delete', row)"
              />
            </el-tooltip>
          </span>
        </div>
        <div class="id-card-caps">
          <el-tag
            v-for="c in enabledCaps(row)"
            :key="c"
            size="small"
            class="cap-tag"
            :type="c === 'admin' ? 'danger' : 'info'"
          >
            {{ capLabel(c) }}
          </el-tag>
          <span v-if="enabledCaps(row).length === 0" class="muted">—</span>
        </div>
        <code class="token-text">{{ maskToken(row.token_hint) }}</code>
      </div>
    </div>
    <el-table v-else-if="identities.length > 0" :data="identities">
      <el-table-column prop="name" :label="t('access.colName')" min-width="120" />
      <el-table-column :label="t('access.colPermissions')" min-width="240">
        <template #default="{ row }">
          <el-tag
            v-for="c in enabledCaps(row)"
            :key="c"
            size="small"
            class="cap-tag"
            :type="c === 'admin' ? 'danger' : 'info'"
          >
            {{ capLabel(c) }}
          </el-tag>
          <span v-if="enabledCaps(row).length === 0" class="muted">—</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('access.colCreatedAt')" min-width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column :label="t('access.colToken')" min-width="110">
        <template #default="{ row }">
          <code class="token-text">{{ maskToken(row.token_hint) }}</code>
        </template>
      </el-table-column>
      <el-table-column :label="t('access.colActions')" width="130" fixed="right">
        <template #default="{ row }">
          <el-tooltip :content="t('common.edit')" placement="top" :enterable="false">
            <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="emit('edit', row)" />
          </el-tooltip>
          <el-tooltip :content="t('access.resetToken')" placement="top" :enterable="false">
            <el-button
              link
              type="primary"
              :icon="RefreshRight"
              :aria-label="t('access.resetToken')"
              @click="emit('reset-token', row)"
            />
          </el-tooltip>
          <el-tooltip :content="t('common.delete')" placement="top" :enterable="false">
            <el-button
              link
              type="danger"
              :icon="Delete"
              :aria-label="t('common.delete')"
              @click="emit('delete', row)"
            />
          </el-tooltip>
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { Delete, Edit, InfoFilled, Plus, RefreshRight } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { useMemoryConfig } from '../../config'
import { formatTime } from '../../format'
import { toastError } from '../../toast'
import {
  CAPS,
  capLabel,
  enabledCaps,
  emptyCaps,
  isEmptyCaps,
  maskToken,
  run,
  type IdentityRow,
  type PresetKey,
} from './caps'
import CapsEditor from './CapsEditor.vue'

const props = defineProps<{
  /** The server's current auth toggle state (refreshed by the parent's load) */
  authRequired: boolean
  /** Whether an admin identity exists in the database (precondition for enabling) */
  hasAdminIdentity: boolean
  /** The server's current anonymous capability set (all-keys object); null = unset (anonymous denied) */
  anonymousPermissions: Record<string, boolean> | null
  identities: IdentityRow[]
  /** In compact mode (<960px) a card list replaces the table (created-at/token columns are dropped) */
  compact: boolean
}>()

const emit = defineEmits<{
  /** The toggle state actually changed (the parent refreshes settings accordingly, and anonymous access availability follows) */
  changed: [enabled: boolean]
  /** The anonymous capability set was saved (the parent refreshes settings) */
  saved: []
  create: []
  edit: [row: IdentityRow]
  delete: [row: IdentityRow]
  'reset-token': [row: IdentityRow]
}>()

// ---- Auth toggle: confirm before changing (el-switch before-change).
// The switch is disabled without an admin identity (disabled + tooltip); the precheck here is
// just a backstop.
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

async function confirmAuthToggle(): Promise<boolean> {
  const target = !authOn.value
  // Backstop precheck: enabling requires an admin identity to exist in the database (same rule
  // as the server-side guard). Don't flip the switch before you have the token in hand.
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

// ---- Anonymous access: rebuild the local draft once server data arrives/refreshes (the load
// after a successful save also flows back through here)
const caps = reactive<Record<string, boolean>>(emptyCaps())
const preset = ref<PresetKey>('custom')
const anonOn = ref(false)
const saving = ref(false)

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

// Disabled state: a save is only needed when it differs from the server (the server has a config);
// enabled state: the server has no config, or the checked boxes differ from the server
const dirty = computed(() =>
  anonOn.value
    ? props.anonymousPermissions === null ||
      CAPS.some((c) => props.anonymousPermissions?.[c.key] !== (caps[c.key] === true))
    : props.anonymousPermissions !== null,
)

async function save(): Promise<void> {
  saving.value = true
  try {
    // Disabled = clear the config (anonymous access denied entirely); enabled = save the checked all-keys object
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
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.card-title {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.section-title {
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.anon-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.anon-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.anon-actions {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
.id-header {
  margin-bottom: 12px;
}
.token-text {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.cap-tag {
  margin-right: 4px;
}
.muted {
  color: var(--el-text-color-secondary);
}
/* Identity card list for narrow containers */
.id-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.id-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: var(--el-border-radius-base);
}
.id-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.id-card-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.id-card-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}
.id-card-caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.id-card-caps .cap-tag {
  margin-right: 0;
}
</style>
