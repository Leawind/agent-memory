<template>
  <div class="access-panel">
    <div class="panel-head">
      <div>
        <h2 class="panel-title">{{ t('access.title') }}</h2>
        <p class="panel-subtitle">{{ t('access.subtitle') }}</p>
      </div>
      <el-button v-if="isAdmin" type="primary" @click="openCreate">
        {{ t('access.create') }}
      </el-button>
    </div>

    <el-alert v-if="!who" type="info" :title="t('access.needAdmin')" :closable="false" class="block" />
    <el-alert
      v-else-if="who.mode === 'open'"
      type="warning"
      :title="t('access.openMode')"
      :closable="false"
      class="block"
    />
    <el-alert v-else-if="!isAdmin" type="info" :title="t('access.needAdmin')" :closable="false" class="block" />

    <template v-if="who && isAdmin">
      <el-card shadow="never" class="block">
        <el-empty v-if="identities.length === 0" :description="t('access.empty')" />
        <el-table v-else :data="identities">
          <el-table-column prop="name" :label="t('access.colName')" min-width="120" />
          <el-table-column :label="t('access.colToken')" min-width="220">
            <template #default="{ row }">
              <div class="token-cell">
                <code class="token-text">{{ maskToken(row.token) }}</code>
                <el-button link type="primary" @click="copyToken(row.token)">
                  {{ t('access.copyToken') }}
                </el-button>
              </div>
            </template>
          </el-table-column>
          <el-table-column :label="t('access.colPermissions')" min-width="260">
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
          <el-table-column :label="t('access.colCreatedAt')" width="170">
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

      <el-card shadow="never" class="block">
        <template #header>{{ t('access.settingsTitle') }}</template>
        <p class="hint">{{ t('access.settingsHint') }}</p>
        <el-form label-position="top" @submit.prevent>
          <el-form-item>
            <template #label>
              <div class="field-label">
                <span>{{ t('access.instructionsLabel') }}</span>
                <el-button link type="primary" @click="restoreInstructions">
                  {{ t('access.restoreDefault') }}
                </el-button>
              </div>
            </template>
            <el-input
              v-model="instructions"
              type="textarea"
              :rows="5"
              :placeholder="t('access.instructionsPlaceholder')"
            />
            <details v-if="instructions" class="default-view">
              <summary>{{ t('access.viewDefault') }}</summary>
              <pre class="default-text">{{ defaultInstructions }}</pre>
            </details>
          </el-form-item>
          <el-form-item>
            <template #label>
              <div class="field-label">
                <span>{{ t('access.conventionsLabel') }}</span>
                <el-button link type="primary" @click="restoreConventions">
                  {{ t('access.restoreDefault') }}
                </el-button>
              </div>
            </template>
            <el-input
              v-model="conventions"
              type="textarea"
              :rows="4"
              :placeholder="t('access.conventionsPlaceholder')"
            />
          </el-form-item>
        </el-form>
        <div class="save-row">
          <el-button type="primary" :loading="savingSettings" @click="saveSettings">
            {{ t('common.save') }}
          </el-button>
        </div>
      </el-card>
    </template>

    <!-- 新建 / 编辑能力 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editing ? t('access.editTitle', { name: editing.name }) : t('access.createTitle')"
      width="480px"
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

    <!-- 新建成功：展示可复制的 token -->
    <el-dialog v-model="tokenShown" :title="t('access.createTitle')" width="560px">
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
import { computed, onMounted, reactive, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { formatTime, t, useApiClient } from '@agent-memory/ui'
import { can, type WhoAmI } from './auth'

const props = defineProps<{ who: WhoAmI | null }>()

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
  token: string
  permissions: Record<string, boolean>
  created_at: number
}

const identities = ref<IdentityRow[]>([])
const instructions = ref('')
const conventions = ref('')
const defaultInstructions = ref('')
const savingSettings = ref(false)

const isAdmin = computed(() => can(props.who, 'admin'))

async function run<T>(action: () => Promise<T>, successMsg?: string): Promise<T | undefined> {
  try {
    const out = await action()
    if (successMsg) ElMessage.success(successMsg)
    return out
  } catch (e) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
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
        default_instructions?: string | null
      }>('/api/settings'),
    ])
    identities.value = Array.isArray(list?.identities) ? list.identities : []
    instructions.value = settings?.instructions ?? ''
    conventions.value = settings?.conventions ?? ''
    defaultInstructions.value = settings?.default_instructions ?? ''
  })
}

onMounted(() => {
  // 无 admin 权限时服务端会 403，静默跳过加载（页面提示已足够）
  if (isAdmin.value) void load()
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
      ElMessage.success(t('access.saved'))
    } else {
      const created = await run(() =>
        api.post<IdentityRow>('/api/identities', {
          name: form.name,
          permissions: { ...form.caps },
        }),
      )
      if (created?.token) {
        createdToken.value = created.token
        tokenShown.value = true
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
  await run(() => api.del(`/api/identities/${encodeURIComponent(row.name)}`))
  ElMessage.success(t('access.deleted'))
  await load()
}

// ---- 设置：恢复默认 = 清空字段（服务端对空值回退内置文案），保存时一并提交 ----
function restoreInstructions(): void {
  instructions.value = ''
}

function restoreConventions(): void {
  conventions.value = ''
}

async function saveSettings(): Promise<void> {
  savingSettings.value = true
  try {
    await run(() =>
      api.put('/api/settings', {
        instructions: instructions.value,
        conventions: conventions.value,
      }),
    )
    ElMessage.success(t('access.saved'))
  } finally {
    savingSettings.value = false
  }
}

// ---- 展示辅助 ----
const tokenShown = ref(false)
const createdToken = ref('')

function maskToken(token: string): string {
  if (typeof token !== 'string' || token.length <= 12) return token
  return `${token.slice(0, 6)}…${token.slice(-4)}`
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
    ElMessage.success(t('access.copied'))
  } catch {
    ElMessage.error(token)
  }
}
</script>

<style scoped>
.access-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.panel-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
}
.panel-title {
  margin: 0;
  font-size: 18px;
  color: var(--el-text-color-primary);
}
.panel-subtitle {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.block {
  margin: 0;
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
.hint {
  margin: 0 0 10px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.field-label {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
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
