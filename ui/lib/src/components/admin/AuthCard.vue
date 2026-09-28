<template>
  <!-- 鉴权卡：鉴权开关（标题行右侧）+ 匿名访问 + 身份表合并为一张卡。
       鉴权边界由显式开关决定，与是否已创建身份无关。 -->
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

    <!-- 身份：新建入口紧贴身份表（小节行右侧） -->
    <div class="card-header id-header">
      <span class="section-title">{{ t('access.identityCard') }}</span>
      <el-button type="primary" :icon="Plus" @click="emit('create')">
        {{ t('access.create') }}
      </el-button>
    </div>
    <!-- 空身份时不占正文空间（新建入口在小节行）；窄容器：卡片列表替代表格——
         el-table 的固定操作列在手机宽度会把能力标签挤碎 -->
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
  /** 服务器当前的鉴权开关状态（随父层 load 刷新） */
  authRequired: boolean
  /** 库里是否存在 admin 身份（开启的前置条件） */
  hasAdminIdentity: boolean
  /** 服务器当前配置的匿名能力集（全键对象）；null = 未设置（匿名被拒绝） */
  anonymousPermissions: Record<string, boolean> | null
  identities: IdentityRow[]
  /** compact（<960px）时以卡片列表替代表格（创建时间/token 列随之收起） */
  compact: boolean
}>()

const emit = defineEmits<{
  /** 开关状态实际改变（父层据此刷新 settings，匿名访问的可用性随之联动） */
  changed: [enabled: boolean]
  /** 匿名能力集保存成功（父层刷新设置） */
  saved: []
  create: []
  edit: [row: IdentityRow]
  delete: [row: IdentityRow]
  'reset-token': [row: IdentityRow]
}>()

// ---- 鉴权开关：先确认再变更（el-switch before-change）。
// 无 admin 身份时开关禁用（disabled + tooltip 说明），这里的预检只作兜底。
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

// ---- 匿名访问：服务器数据到达/刷新后重建本地草稿（保存成功后的 load 也会走回这里）
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
/* 窄容器的身份卡片列表 */
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
