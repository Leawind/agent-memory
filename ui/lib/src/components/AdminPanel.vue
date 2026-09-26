<template>
  <div ref="rootRef" class="am-panel admin-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div class="am-heading">
        <h2 class="am-panel-title">{{ props.title ?? t('access.title') }}</h2>
        <el-tooltip :content="props.subtitle ?? t('access.subtitle')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
      </div>
      <el-button v-if="isAdmin" type="primary" :icon="Plus" @click="openCreate">
        {{ t('access.create') }}
      </el-button>
    </div>

    <el-alert v-if="!who" type="info" :title="t('access.needAdmin')" :closable="false" />
    <el-alert v-else-if="who.mode === 'open'" type="warning" :title="t('access.openMode')" :closable="false" />
    <el-alert v-else-if="!isAdmin" type="info" :title="t('access.needAdmin')" :closable="false" />

    <template v-if="who && isAdmin">
      <!-- 鉴权开关：鉴权边界由显式开关决定，与是否已创建身份无关 -->
      <el-card shadow="never">
        <div class="auth-row">
          <div class="auth-text">
            <span class="auth-label">{{ t('access.authTitle') }}</span>
            <span class="auth-hint">{{ t('access.authHint') }}</span>
          </div>
          <el-switch v-model="authRequired" :before-change="confirmAuthToggle" :loading="togglingAuth" />
        </div>
      </el-card>

      <el-card shadow="never">
        <el-empty v-if="identities.length === 0" :description="t('access.empty')" />
        <el-table v-else :data="identities">
          <el-table-column prop="name" :label="t('access.colName')" min-width="120" />
          <el-table-column :label="t('access.colToken')" min-width="200">
            <template #default="{ row }">
              <div class="token-cell">
                <code class="token-text">{{ maskToken(row.token_hint) }}</code>
                <el-button link type="primary" @click="askResetToken(row)">
                  {{ t('access.resetToken') }}
                </el-button>
              </div>
            </template>
          </el-table-column>
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
          <el-table-column v-if="!compact" :label="t('access.colCreatedAt')" width="170">
            <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
          </el-table-column>
          <el-table-column :label="t('access.colActions')" width="140" fixed="right">
            <template #default="{ row }">
              <el-button link type="primary" @click="openEdit(row)">
                {{ t('common.edit') }}
              </el-button>
              <el-button link type="danger" @click="askDelete(row)">
                {{ t('common.delete') }}
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <el-card shadow="never">
        <template #header>{{ t('access.settingsTitle') }}</template>
        <!-- 留空即默认：行为说明放在各自字段的提示行里，紧邻输入框 -->
        <el-form label-position="top" @submit.prevent>
          <el-form-item :label="t('access.instructionsLabel')">
            <el-input
              v-model="instructions"
              type="textarea"
              :rows="5"
              :placeholder="t('access.instructionsPlaceholder')"
            />
            <div class="field-hint">{{ t('access.instructionsHint') }}</div>
            <details v-if="instructions" class="default-view">
              <summary>{{ t('access.viewDefault') }}</summary>
              <pre class="default-text">{{ defaultInstructions }}</pre>
            </details>
          </el-form-item>
          <el-form-item :label="t('access.conventionsLabel')">
            <el-input
              v-model="conventions"
              type="textarea"
              :rows="4"
              :placeholder="t('access.conventionsPlaceholder')"
            />
            <div class="field-hint">{{ t('access.conventionsHint') }}</div>
          </el-form-item>
        </el-form>
        <div class="save-row">
          <el-button type="primary" :loading="savingSettings" @click="saveSettings">
            {{ t('common.save') }}
          </el-button>
        </div>
      </el-card>

      <!-- 备份导入导出：admin 专属端点（/api/export 由 http 层拦截为附件下载） -->
      <el-card shadow="never">
        <template #header>{{ t('access.backupCard') }}</template>
        <div class="actions">
          <el-button :icon="Download" :loading="exporting" @click="run(exportData)">
            {{ t('access.export') }}
          </el-button>
          <el-button :icon="UploadFilled" :loading="importing" @click="importInput?.click()">
            {{ t('access.import') }}
          </el-button>
          <el-tooltip :content="t('access.importHint')" placement="top">
            <el-icon class="am-info"><InfoFilled /></el-icon>
          </el-tooltip>
          <input
            ref="importInput"
            type="file"
            accept="application/json,.json"
            style="display: none"
            @change="onImportFile"
          />
        </div>
      </el-card>

      <el-card shadow="never">
        <template #header>
          <div class="card-header">
            <span>{{ t('access.doctorCard') }}</span>
            <el-button size="small" :icon="Search" :loading="doctorLoading" @click="run(runDoctor)">
              {{ t('access.runDoctor') }}
            </el-button>
          </div>
        </template>
        <template v-if="doctorRan">
          <el-alert v-if="doctor.ok" :title="t('access.doctorOk')" type="success" show-icon :closable="false" />
          <el-alert
            v-else
            :title="t('access.doctorFail', { count: doctor.issues.length })"
            type="error"
            show-icon
            :closable="false"
          />
          <ul v-if="!doctor.ok" class="issues">
            <li v-for="(issue, i) in doctor.issues" :key="i">{{ issue }}</li>
          </ul>
        </template>
        <el-empty v-else :description="t('access.doctorEmpty')" :image-size="60" />
      </el-card>
    </template>

    <!-- 新建 / 编辑能力 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editing ? t('access.editTitle', { name: editing.name }) : t('access.createTitle')"
      :width="compact ? '96%' : '480px'"
    >
      <el-form label-position="top" @submit.prevent>
        <el-form-item v-if="!editing" :label="t('access.nameLabel')">
          <el-input v-model="form.name" :placeholder="t('access.namePlaceholder')" />
        </el-form-item>
        <el-form-item :label="t('access.permsLabel')">
          <div class="presets">
            <span class="presets-label">{{ t('access.presets') }}</span>
            <el-radio-group v-model="preset" size="small" @change="applyPreset">
              <el-radio-button value="admin">{{ t('access.presetAdmin') }}</el-radio-button>
              <el-radio-button value="member">{{ t('access.presetMember') }}</el-radio-button>
              <el-radio-button value="viewer">{{ t('access.presetViewer') }}</el-radio-button>
              <el-radio-button value="custom">{{ t('access.presetCustom') }}</el-radio-button>
            </el-radio-group>
          </div>
          <div class="caps">
            <el-checkbox v-for="c in CAPS" :key="c.key" v-model="form.caps[c.key]" @change="onManualToggle">
              {{ capLabel(c.key) }}
            </el-checkbox>
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="submitting" @click="submit">
          {{ t('common.save') }}
        </el-button>
      </template>
    </el-dialog>

    <!-- 新建 / 重置成功：token 明文仅此一次展示 -->
    <el-dialog v-model="tokenShown" :title="tokenShownTitle" :width="compact ? '96%' : '560px'">
      <p>{{ t('access.created') }}</p>
      <code class="new-token">{{ createdToken }}</code>
      <template #footer>
        <el-button
          type="primary"
          @click="
            () => {
              copyToken(createdToken)
              tokenShown = false
            }
          "
        >
          {{ t('access.copyToken') }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { Download, InfoFilled, Plus, Search, UploadFilled } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { t } from '../i18n'
import { useApiClient } from '../api/client'
import { useAdmin } from '../composables/useAdmin'
import { useContainerWidth } from '../composables/useContainerWidth'
import { toastError, toastSuccess } from '../toast'
import type { WhoAmI } from '../types'

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

// 能力清单：key 与服务端 auth::Cap 的 JSON 键一致（唯一登记表）
const CAPS = [
  { key: 'read', labelKey: 'capRead' },
  { key: 'create', labelKey: 'capCreate' },
  { key: 'update', labelKey: 'capUpdate' },
  { key: 'delete', labelKey: 'capDelete' },
  { key: 'tag_manage', labelKey: 'capTagManage' },
  { key: 'admin', labelKey: 'capAdmin' },
] as const

interface IdentityRow {
  name: string
  token_hint: string
  permissions: Record<string, boolean>
  created_at: number
}

const identities = ref<IdentityRow[]>([])
const instructions = ref('')
const conventions = ref('')
const defaultInstructions = ref('')
const savingSettings = ref(false)
const authRequired = ref(false)
const togglingAuth = ref(false)

const isAdmin = computed(() => !!props.who && props.who.permissions?.admin === true)

const { doctor, doctorRan, doctorLoading, exporting, importing, runDoctor, exportData, importFile } = useAdmin()

const rootRef = ref<HTMLElement | null>(null)
// compact（<960px）时创建时间列收起、对话框加宽到 96%
const { compact } = useContainerWidth(rootRef)

const importInput = ref<HTMLInputElement | null>(null)

async function run<T>(action: () => Promise<T>, successMsg?: string): Promise<T | undefined> {
  try {
    const out = await action()
    if (successMsg) toastSuccess(successMsg)
    return out
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
    return undefined
  }
}

async function load(): Promise<void> {
  await run(async () => {
    const [list, settings] = await Promise.all([
      api.get<{ identities?: IdentityRow[] }>('/api/identities'),
      api.get<{
        instructions?: string | null
        conventions?: string | null
        auth_required?: boolean
        default_instructions?: string | null
      }>('/api/settings'),
    ])
    identities.value = Array.isArray(list?.identities) ? list.identities : []
    instructions.value = settings?.instructions ?? ''
    conventions.value = settings?.conventions ?? ''
    defaultInstructions.value = settings?.default_instructions ?? ''
    authRequired.value = settings?.auth_required === true
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

// ---- 新建 / 编辑 ----
const dialogVisible = ref(false)
const submitting = ref(false)
const editing = ref<IdentityRow | null>(null)
const preset = ref<'admin' | 'member' | 'viewer' | 'custom'>('custom')
const form = reactive<{ name: string; caps: Record<string, boolean> }>({
  name: '',
  caps: emptyCaps(),
})

function emptyCaps(): Record<string, boolean> {
  return Object.fromEntries(CAPS.map((c) => [c.key, false]))
}

const PRESETS: Record<string, string[]> = {
  admin: CAPS.map((c) => c.key),
  member: ['read', 'create', 'update', 'tag_manage'],
  viewer: ['read'],
}

function applyPreset(): void {
  if (preset.value === 'custom') return
  const keys = new Set(PRESETS[preset.value] ?? [])
  for (const c of CAPS) form.caps[c.key] = keys.has(c.key)
}

function onManualToggle(): void {
  preset.value = 'custom'
}

function openCreate(): void {
  editing.value = null
  form.name = ''
  Object.assign(form.caps, emptyCaps())
  preset.value = 'member'
  applyPreset()
  dialogVisible.value = true
}

function openEdit(row: IdentityRow): void {
  editing.value = row
  form.name = row.name
  for (const c of CAPS) form.caps[c.key] = row.permissions?.[c.key] === true
  preset.value = 'custom'
  dialogVisible.value = true
}

async function submit(): Promise<void> {
  submitting.value = true
  try {
    if (editing.value) {
      await run(() =>
        api.put(`/api/identities/${encodeURIComponent(editing.value!.name)}`, {
          permissions: { ...form.caps },
        }),
      )
      toastSuccess(t('access.saved'))
    } else {
      const created = await run(() =>
        api.post<IdentityRow & { token?: string }>('/api/identities', {
          name: form.name,
          permissions: { ...form.caps },
        }),
      )
      if (created?.token) {
        showTokenOnce(t('access.createTitle'), created.token)
      }
    }
    dialogVisible.value = false
    await load()
  } finally {
    submitting.value = false
  }
}

// ---- 删除 ----
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

// ---- 鉴权开关：先确认再变更（el-switch before-change）----
async function confirmAuthToggle(): Promise<boolean> {
  const target = !authRequired.value
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
    await run(() => api.put('/api/settings', { auth_required: target }))
    return true
  } finally {
    togglingAuth.value = false
  }
}

// ---- 设置：留空即默认（服务端对空值回退内置文案），保存时一并提交 ----
async function saveSettings(): Promise<void> {
  savingSettings.value = true
  try {
    await run(() =>
      api.put('/api/settings', {
        instructions: instructions.value,
        conventions: conventions.value,
      }),
    )
    toastSuccess(t('access.saved'))
  } finally {
    savingSettings.value = false
  }
}

// ---- 备份 ----
function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importFile(file)
    .then((imported) => {
      toastSuccess(t('access.imported', { memories: imported.imported_memories, tags: imported.imported_tags }))
    })
    .catch((err: unknown) => {
      toastError(err instanceof Error ? err.message : String(err))
    })
}

// ---- 展示辅助 ----
const tokenShown = ref(false)
const tokenShownTitle = ref('')
const createdToken = ref('')

// token 只存哈希，列表只有尾缀提示
function maskToken(hint: string): string {
  return hint ? `…${hint}` : '—'
}

function showTokenOnce(title: string, token: string): void {
  tokenShownTitle.value = title
  createdToken.value = token
  tokenShown.value = true
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
  if (reset?.token) showTokenOnce(t('access.resetTitle'), reset.token)
  await load()
}

function enabledCaps(row: IdentityRow): string[] {
  return CAPS.map((c) => c.key).filter((k) => row.permissions?.[k] === true)
}

function capLabel(key: string): string {
  const found = CAPS.find((c) => c.key === key)
  return found ? t(`access.${found.labelKey}`) : key
}

async function copyToken(token: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(token)
    toastSuccess(t('access.copied'))
  } catch {
    toastError(token)
  }
}
</script>

<style scoped>
.admin-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
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
.token-cell {
  display: flex;
  align-items: center;
  gap: 8px;
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
.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}
.issues {
  margin: 12px 0 0;
  padding-left: 20px;
  line-height: 1.9;
}
.presets {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.presets-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 16px;
}
.new-token {
  display: block;
  margin-top: 8px;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-fill-color);
  font-size: 13px;
  word-break: break-all;
  user-select: all;
}
</style>
