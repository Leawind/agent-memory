<template>
  <el-config-provider :locale="epLocale">
    <div class="shell">
      <header class="topbar">
        <div class="topbar-inner">
          <div class="brand">
            <span class="brand-mark">
              <el-icon :size="16"><Collection /></el-icon>
            </span>
            <span class="brand-name">Agent Memory</span>
            <!-- 身份切换在标题处：Agent Memory / 身份名；开放模式无身份区 -->
            <template v-if="who && who.mode === 'token'">
              <span class="brand-sep">/</span>
              <el-dropdown trigger="click" @command="onIdentityCommand">
                <button type="button" class="identity-btn" :aria-label="t('shell.identity')">
                  <span class="identity-name">{{ who.name }}</span>
                  <el-icon :size="12" class="identity-caret"><ArrowDown /></el-icon>
                </button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item
                      v-for="it in identities"
                      :key="it.name"
                      :command="{ type: 'switch', name: it.name }"
                      :data-checked="it.name === who?.name"
                    >
                      <span class="id-option">
                        <span class="id-name">{{ it.name }}</span>
                        <span class="id-hint">{{ it.hint }}</span>
                      </span>
                      <el-icon v-if="it.name === who?.name" class="id-check"><Check /></el-icon>
                      <el-icon
                        class="id-remove"
                        :title="t('shell.removeIdentity')"
                        @click.stop="removeIdentityClick(it.name)"
                      >
                        <Delete />
                      </el-icon>
                    </el-dropdown-item>
                    <!-- 添加身份：整行只有一个加号图标，点击弹令牌输入框 -->
                    <el-dropdown-item divided command="add" class="id-add" :aria-label="t('shell.addIdentity')">
                      <el-icon :size="14"><Plus /></el-icon>
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
          </div>

          <!-- 顶部导航：面板常驻挂载，切换不销毁；切回时经 refresh 拉最新数据。
               管理标签页仅对具备 admin 能力的身份显示 -->
          <nav class="nav">
            <button
              v-for="item in navItems"
              :key="item.key"
              type="button"
              class="nav-item"
              :class="{ active: active === item.key }"
              @click="active = item.key"
            >
              <el-icon :size="15"><component :is="item.icon" /></el-icon>
              <span>{{ item.label }}</span>
            </button>
          </nav>

          <div class="actions">
            <!-- 主题：三态图标分段（浅色/深色/跟随系统） -->
            <div class="seg" role="group" :aria-label="t('shell.theme')">
              <button
                v-for="opt in themeOptions"
                :key="opt.value"
                type="button"
                class="seg-item"
                :class="{ active: theme === opt.value }"
                :title="opt.label"
                :aria-label="opt.label"
                @click="theme = opt.value"
              >
                <el-icon :size="15"><component :is="opt.icon" /></el-icon>
              </button>
            </div>

            <!-- 语言：下拉菜单（nativeName 不随界面语言变；新增语言 = 新字典 + 一项菜单） -->
            <el-dropdown trigger="click" @command="setLang">
              <button type="button" class="lang-btn" :aria-label="t('shell.language')">
                <span>{{ currentLanguage.nativeName }}</span>
                <el-icon :size="12"><ArrowDown /></el-icon>
              </button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item
                    v-for="l in languages"
                    :key="l.value"
                    :command="l.value"
                    :data-checked="lang === l.value"
                  >
                    <span class="lang-option">{{ l.nativeName }}</span>
                    <el-icon v-if="lang === l.value" class="lang-check"><Check /></el-icon>
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </header>

      <main class="content">
        <!-- .memory-ui 命名空间：组件库令牌样式（tokens.css）的选择器前缀 -->
        <div class="memory-ui">
          <MemoriesPanel ref="memoriesPanel" v-show="active === 'memories'" />
          <TagsPanel ref="tagsPanel" v-show="active === 'tags'" />
          <OpsPanel ref="opsPanel" v-show="active === 'ops'" />
          <!-- 管理面板：只对 admin 身份挂载（v-if），非 admin 身份不产生任何管理请求 -->
          <AdminPanel v-if="isAdmin" ref="adminPanel" v-show="active === 'admin'" :who="who" />
        </div>
      </main>

      <!-- 令牌输入：401 或「添加身份」时弹出；保存前先验证再入库 -->
      <el-dialog
        v-model="tokenDialog"
        :title="tokenDialogMode === 'add' ? t('shell.addIdentity') : t('shell.tokenPromptTitle')"
        width="440px"
        :close-on-click-modal="false"
      >
        <p class="token-desc">
          {{ tokenDialogMode === 'add' ? t('shell.addIdentityDesc') : t('shell.tokenPromptDesc') }}
        </p>
        <el-input
          v-model="tokenInput"
          :placeholder="t('shell.tokenPlaceholder')"
          show-password
          clearable
          @keyup.enter="saveToken"
        />
        <p v-if="tokenError" class="token-error">{{ t('shell.tokenInvalid') }}</p>
        <template #footer>
          <el-button type="primary" :loading="checkingToken" @click="saveToken">
            {{ t('shell.tokenConfirm') }}
          </el-button>
        </template>
      </el-dialog>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import {
  ArrowDown,
  Check,
  Collection,
  Delete,
  Key,
  Monitor,
  Moon,
  Notebook,
  Odometer,
  Plus,
  PriceTag,
  Sunny,
} from '@element-plus/icons-vue'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import en from 'element-plus/es/locale/lang/en'
import { AdminPanel, MemoriesPanel, OpsPanel, TagsPanel, provideMemoryUI, setMemoryUILocale, t } from '@agent-memory/ui'
import {
  UNAUTHORIZED_EVENT,
  addIdentity,
  authFetch,
  can,
  currentIdentityName,
  fetchWhoAmI,
  listIdentities,
  listIdentityNames,
  removeIdentity,
  switchIdentity,
  type WhoAmI,
} from './auth'

// 同源部署：API 经带鉴权头的 fetch（token 缺省时与原生 fetch 等价）。
// onIdentityToken：管理面板创建/重置身份后「保存到本浏览器」的落点——
// 记入多身份令牌表并立即切换到该身份，避免 token 只展示一次而错失保存。
provideMemoryUI({
  fetch: authFetch,
  onIdentityToken: async (name, token) => {
    addIdentity(name, token)
    refreshIdentities()
    await resolveIdentity()
    refreshActivePanel()
  },
  // 鉴权开关切换后重新解析身份：开放 ↔ token 模式切换时标题处身份区即时反映
  onAuthChanged: () => resolveIdentity(),
})

type AdminTab = 'memories' | 'tags' | 'ops' | 'admin'
const active = ref<AdminTab>('memories')

// ---- 身份与令牌：本浏览器可保存多个身份，标题处下拉切换/删除/添加 ----
const who = ref<WhoAmI | null>(null)
const identities = ref<{ name: string; hint: string }[]>([])

const isAdmin = computed(() => can(who.value, 'admin'))

// 管理标签页仅 admin 可见；当前页失去可见性时（如切到低权限身份）退回记忆管理
const navItems = computed(() => {
  const items: { key: AdminTab; label: string; icon: typeof Notebook }[] = [
    { key: 'memories', label: t('nav.memories'), icon: Notebook },
    { key: 'tags', label: t('nav.tags'), icon: PriceTag },
    { key: 'ops', label: t('nav.ops'), icon: Odometer },
  ]
  if (isAdmin.value) items.push({ key: 'admin', label: t('nav.admin'), icon: Key })
  return items
})

watch(navItems, (items) => {
  if (!items.some((item) => item.key === active.value)) active.value = 'memories'
})

// 面板常驻挂载（v-show）不会重新挂载，切回时数据可能陈旧：切换时刷新目标面板
interface RefreshablePanel {
  refresh: () => void
}
const memoriesPanel = ref<RefreshablePanel | null>(null)
const tagsPanel = ref<RefreshablePanel | null>(null)
const opsPanel = ref<RefreshablePanel | null>(null)
const adminPanel = ref<RefreshablePanel | null>(null)

function refreshActivePanel(): void {
  const panel = {
    memories: memoriesPanel.value,
    tags: tagsPanel.value,
    ops: opsPanel.value,
    admin: adminPanel.value,
  }[active.value]
  panel?.refresh()
}

watch(active, () => refreshActivePanel())

// ---- 身份下拉与令牌弹窗状态 ----
function refreshIdentities(): void {
  identities.value = listIdentities()
}

const tokenDialog = ref(false)
const tokenDialogMode = ref<'required' | 'add'>('required')
const tokenInput = ref('')
const tokenError = ref(false)
const checkingToken = ref(false)

async function resolveIdentity(): Promise<void> {
  const r = await fetchWhoAmI()
  if (r.ok) {
    who.value = r.who
  } else if (r.needToken) {
    // 当前 token 已失效：移除这个失效身份（若有效 token 在请求中，服务端不会 401）
    const name = currentIdentityName()
    if (name) removeIdentity(name)
    refreshIdentities()
    who.value = null
    tokenDialogMode.value = 'required'
    tokenDialog.value = true
  }
}

async function switchTo(name: string): Promise<void> {
  switchIdentity(name)
  refreshIdentities()
  who.value = null
  await resolveIdentity()
  refreshActivePanel()
}

async function saveToken(): Promise<void> {
  const token = tokenInput.value.trim()
  if (!token) return
  checkingToken.value = true
  tokenError.value = false
  // 先验证候选 token 再入库，无效时不污染已保存的身份表
  const r = await fetchWhoAmI(token)
  checkingToken.value = false
  if (r.ok) {
    addIdentity(r.who.name, token)
    refreshIdentities()
    who.value = r.who
    tokenDialog.value = false
    tokenInput.value = ''
    refreshActivePanel()
  } else {
    tokenError.value = true
  }
}

async function onIdentityCommand(command: 'add' | { type: 'switch'; name: string }): Promise<void> {
  if (command === 'add') {
    tokenDialogMode.value = 'add'
    tokenInput.value = ''
    tokenError.value = false
    tokenDialog.value = true
    return
  }
  if (command.name !== who.value?.name) {
    await switchTo(command.name)
  }
}

async function removeIdentityClick(name: string): Promise<void> {
  const wasCurrent = currentIdentityName() === name
  removeIdentity(name)
  refreshIdentities()
  if (!wasCurrent) return
  // 删除的是当前身份：自动切到剩余的第一个；一个不剩时回到未连接状态
  who.value = null
  const next = listIdentityNames()[0]
  if (next) {
    await switchTo(next)
  } else {
    await resolveIdentity()
    refreshActivePanel()
  }
}

function onUnauthorized(): void {
  // 401 说明当前 token 已被服务端拒绝（重置/删除）：移除并要求重新输入
  void resolveIdentity()
}

onMounted(() => {
  refreshIdentities()
  void resolveIdentity()
  window.addEventListener(UNAUTHORIZED_EVENT, onUnauthorized)
})
onUnmounted(() => window.removeEventListener(UNAUTHORIZED_EVENT, onUnauthorized))

// ---- 语言（界面文案 + Element Plus 内置文案同步切换）----
// 新增语言两步走：lib 里加字典文件并在 messages 注册 + 在 languages 列表加一项
type LangChoice = 'zh' | 'en'
const LANG_KEY = 'agent-memory-locale'
// nativeName 用语言本名显示，不随界面语言翻译
const languages = [
  { value: 'zh' as LangChoice, nativeName: '中文' },
  { value: 'en' as LangChoice, nativeName: 'English' },
]
const lang = ref<LangChoice>(readStoredLang())
const epLocale = computed(() => (lang.value === 'zh' ? zhCn : en))
const currentLanguage = computed(() => languages.find((l) => l.value === lang.value) ?? languages[0])
// 初始偏好（持久化值或浏览器语言）同步进库内 i18n 实例
setMemoryUILocale(lang.value)

function setLang(value: LangChoice) {
  lang.value = value
}

// 无持久化偏好时按浏览器语言选择（与 lib 的 auto 探测同规则）
function detectLang(): LangChoice {
  return navigator.language?.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

function readStoredLang(): LangChoice {
  try {
    const v = localStorage.getItem(LANG_KEY)
    if (v === 'zh' || v === 'en') return v
  } catch {
    /* 隐私模式等场景读不了就算了 */
  }
  return detectLang()
}

watch(lang, (value) => {
  try {
    localStorage.setItem(LANG_KEY, value)
  } catch {
    /* 隐私模式等场景存不了就算了 */
  }
  setMemoryUILocale(value)
})

// ---- 主题：浅色 / 深色 / 跟随系统 ----
type ThemeChoice = 'light' | 'dark' | 'system'
const themeOptions = computed(() => [
  { label: t('shell.themeLight'), value: 'light' as const, icon: Sunny },
  { label: t('shell.themeDark'), value: 'dark' as const, icon: Moon },
  { label: t('shell.themeSystem'), value: 'system' as const, icon: Monitor },
])
const STORAGE_KEY = 'agent-memory-theme'

const theme = ref<ThemeChoice>(readStoredTheme())
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)')

function readStoredTheme(): ThemeChoice {
  try {
    const v = localStorage.getItem(STORAGE_KEY)
    return v === 'light' || v === 'dark' || v === 'system' ? v : 'system'
  } catch {
    return 'system'
  }
}

function applyTheme() {
  const dark = theme.value === 'dark' || (theme.value === 'system' && prefersDark.matches)
  document.documentElement.classList.toggle('dark', dark)
}

watch(theme, () => {
  try {
    localStorage.setItem(STORAGE_KEY, theme.value)
  } catch {
    /* 隐私模式等场景存不了就算了 */
  }
  applyTheme()
})

onMounted(() => {
  applyTheme()
  // 跟随系统时监听系统主题变化
  prefersDark.addEventListener?.('change', () => {
    if (theme.value === 'system') applyTheme()
  })
})
</script>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
}
body {
  background: var(--el-bg-color-page);
}
</style>

<style scoped>
.shell {
  min-height: 100%;
  display: flex;
  flex-direction: column;
}

/* 顶栏：品牌 + 导航 + 偏好控件 */
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.topbar-inner {
  max-width: 1240px;
  margin: 0 auto;
  padding: 0 24px;
  height: 60px;
  display: flex;
  align-items: center;
  gap: 20px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: -0.01em;
  color: var(--el-text-color-primary);
  flex-shrink: 0;
}
/* 品牌图标：品牌绿渐变圆角块（Modrinth 的 --brand-gradient-bg） */
.brand-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  color: var(--el-color-primary);
  background: linear-gradient(135deg, rgba(68, 182, 138, 0.25) 0%, rgba(58, 250, 112, 0.18) 100%);
}
.nav {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  min-width: 0;
}
/* 导航项：胶囊形，悬浮换表面色，选中铺品牌绿软底 */
.nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 36px;
  padding: 0 14px;
  border: none;
  border-radius: var(--el-border-radius-base);
  background: transparent;
  color: var(--el-text-color-regular);
  font-size: 14px;
  font-family: inherit;
  cursor: pointer;
}
.nav-item:hover {
  background: var(--el-fill-color-light);
}
.nav-item.active {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  font-weight: 600;
}
.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

/* 主题分段：iOS 风格胶囊，激活项浮起 */
.seg {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px;
  border-radius: 999px;
  background: var(--el-fill-color);
}
.seg-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 24px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--el-text-color-secondary);
  cursor: pointer;
}
.seg-item:hover {
  color: var(--el-text-color-primary);
}
.seg-item.active {
  background: var(--el-bg-color);
  color: var(--el-color-primary);
  box-shadow: var(--el-box-shadow-light);
}

/* 语言下拉触发器：与主题分段同风格的胶囊 */
.lang-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 12px;
  border: none;
  border-radius: 999px;
  background: var(--el-fill-color);
  color: var(--el-text-color-regular);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}
.lang-btn:hover {
  color: var(--el-text-color-primary);
  background: var(--el-fill-color-dark);
}
.lang-option {
  flex: 1;
  margin-right: 12px;
}
.lang-check {
  color: var(--el-color-primary);
}

/* 身份切换：品牌行内的可点击身份名（Agent Memory / 身份名） */
.brand-sep {
  color: var(--el-text-color-secondary);
  font-weight: 400;
}
.identity-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
  max-width: 240px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--el-border-radius-base);
  background: transparent;
  color: var(--el-text-color-primary);
  font-size: inherit;
  font-weight: inherit;
  font-family: inherit;
  letter-spacing: inherit;
  cursor: pointer;
}
.identity-btn:hover {
  background: var(--el-fill-color);
}
.identity-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.identity-caret {
  flex-shrink: 0;
  color: var(--el-text-color-secondary);
}
/* 下拉行：身份名 + token 首尾提示靠左，勾选与删除靠右 */
.id-option {
  display: flex;
  align-items: baseline;
  gap: 8px;
  flex: 1;
  min-width: 0;
  margin-right: 12px;
}
.id-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.id-hint {
  flex-shrink: 0;
  font-family: var(--el-font-family-mono, ui-monospace, monospace);
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.id-check {
  flex-shrink: 0;
  color: var(--el-color-primary);
}
.id-remove {
  flex-shrink: 0;
  margin-left: 10px;
  color: var(--el-text-color-secondary);
  cursor: pointer;
}
.id-remove:hover {
  color: var(--el-color-danger);
}
/* 添加身份：整行居中一个加号 */
.id-add {
  justify-content: center;
}

/* 令牌弹窗文案 */
.token-desc {
  margin: 0 0 12px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  line-height: 1.6;
}
.token-error {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--el-color-danger);
}

/* 内容区：通栏留白 + 居中容器 */
.content {
  flex: 1;
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
  padding: 24px;
  box-sizing: border-box;
}
</style>
