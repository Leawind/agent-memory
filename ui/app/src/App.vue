<template>
  <el-config-provider :locale="epLocale">
    <div class="shell">
      <header class="topbar">
        <div class="topbar-inner">
          <div class="brand">
            <!-- Click the title to open the server overview dialog (OpsDialog) -->
            <button type="button" class="brand-btn" @click="opsDialogVisible = true">
              <span class="brand-mark">
                <el-icon :size="16"><Collection /></el-icon>
              </span>
              <span class="brand-name">Agent Memory</span>
            </button>
            <!-- Identity switcher sits in the title: Agent Memory / identity name. Hidden only when open mode is confirmed
                 (who.mode === 'open'); when the current identity is invalid (who empty) the dropdown still renders so you can switch or add -->
            <template v-if="showIdentities">
              <span class="brand-sep">/</span>
              <el-dropdown trigger="click" @command="onIdentityCommand">
                <button type="button" class="identity-btn" :aria-label="t('shell.identity')">
                  <span class="identity-name">{{ identityLabel }}</span>
                  <el-icon :size="12" class="identity-caret"><ArrowDown /></el-icon>
                </button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item
                      v-for="it in identities"
                      :key="it.name"
                      :command="{ type: 'switch', name: it.name }"
                      :data-checked="it.name === currentName"
                    >
                      <span class="id-option">
                        <span class="id-name">{{ it.name }}</span>
                        <span class="id-hint">{{ it.hint }}</span>
                      </span>
                      <el-icon v-if="it.name === currentName" class="id-check"><Check /></el-icon>
                      <el-icon
                        class="id-remove"
                        :title="t('shell.removeIdentity')"
                        @click.stop="removeIdentityClick(it.name)"
                      >
                        <Delete />
                      </el-icon>
                    </el-dropdown-item>
                    <!-- Add identity: a single plus icon fills the row; clicking opens the token input -->
                    <el-dropdown-item divided command="add" class="id-add" :aria-label="t('shell.addIdentity')">
                      <el-icon :size="14"><Plus /></el-icon>
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </template>
          </div>

          <!-- Top navigation: panels stay mounted, switching keeps state; a refresh pulls the latest data on return.
               The admin tab only shows for identities with the admin capability -->
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
            <!-- Theme: three-state segmented icon control (light / dark / follow system) -->
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

            <!-- Language: dropdown. The trigger is a fixed translate icon, never the current language
                 text (nativeName in menu items never follows the UI language; adding a language =
                 a new dictionary + one menu item). Element Plus icons have no translate glyph and
                 pulling in a second icon set for one glyph is not worth it, so the standard
                 "languages" mark is inlined here (same as the de-agent-bug shell). -->
            <el-dropdown trigger="click" @command="setLang">
              <button type="button" class="lang-btn" :aria-label="t('shell.language')">
                <svg
                  class="lang-icon"
                  viewBox="0 0 24 24"
                  width="15"
                  height="15"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d="m5 8 6 6" />
                  <path d="m4 14 6-6 2-3" />
                  <path d="M2 5h12" />
                  <path d="M7 2h1" />
                  <path d="m22 22-5-10-5 10" />
                  <path d="M14 18h6" />
                </svg>
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
        <!-- .memory-ui namespace: selector prefix for the library's token styles (tokens.css) -->
        <div class="memory-ui">
          <MemoriesPanel ref="memoriesPanel" v-show="active === 'memories'" />
          <TagsPanel ref="tagsPanel" v-show="active === 'tags'" />
          <!-- Admin panel: mounted only for admin identities (v-if); non-admin identities make no admin requests -->
          <AdminPanel v-if="isAdmin" ref="adminPanel" v-show="active === 'admin'" :who="who" />
        </div>
      </main>

      <!-- Server overview: opened by clicking the header title; OpsDialog refetches each time it opens -->
      <OpsDialog :visible="opsDialogVisible" @update:visible="opsDialogVisible = $event" />

      <!-- Token input: shown on 401 or on "add identity"; validated before it is stored.
           Non-modal + click-through: the 401 prompt doesn't lock the page, and the header identity dropdown stays usable (switching to another saved identity recovers).
           modal=false alone isn't enough - EP's fullscreen scroll container still blocks clicks; penetrable turns it into pointer-events:none -->
      <el-dialog
        v-model="tokenDialog"
        :modal="false"
        modal-penetrable
        :title="tokenDialogMode === 'add' ? t('shell.addIdentity') : t('shell.tokenPromptTitle')"
        width="440px"
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
  Plus,
  PriceTag,
  Sunny,
} from '@element-plus/icons-vue'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import en from 'element-plus/es/locale/lang/en'
import {
  AdminPanel,
  MemoriesPanel,
  OpsDialog,
  TagsPanel,
  provideMemoryUI,
  setMemoryUILocale,
  t,
} from '@agent-memory/ui'
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

// Same-origin deployment: API goes through a fetch carrying the auth header (identical to native
// fetch when no token is set).
// onIdentityToken: where the admin panel's "save to this browser" lands after creating/resetting
// an identity — record it in the multi-identity token table and switch to that identity
// immediately, so a token shown only once is not lost.
provideMemoryUI({
  fetch: authFetch,
  onIdentityToken: async (name, token) => {
    addIdentity(name, token)
    refreshIdentities()
    await resolveIdentity()
    refreshActivePanel()
  },
  // Re-resolve the identity after the auth toggle flips: the top-bar identity area reflects the open <-> token mode switch immediately
  onAuthChanged: () => resolveIdentity(),
})

type AdminTab = 'memories' | 'tags' | 'admin'
const active = ref<AdminTab>('memories')
// Service overview dialog: opened by clicking the top-bar title
const opsDialogVisible = ref(false)

// ---- Identities and tokens: this browser can store multiple identities, switched/removed/added
// from the dropdown at the title ----
const who = ref<WhoAmI | null>(null)
const identities = ref<{ name: string; hint: string }[]>([])
// Local current-identity pointer (localStorage), decoupled from who (the server-verified result):
// who only reflects "the most recent whoami"; the dropdown must stay usable even when that
// verification fails
const currentName = ref<string | null>(null)
// The most recent whoami explicitly returned 401 (the server has token auth on) — even with an
// empty who this counts as token mode
const needToken = ref(false)

const isAdmin = computed(() => can(who.value, 'admin'))

// Identity area visibility: hidden only in confirmed open mode (who.mode === 'open'); whether the
// current identity is valid does not matter — after a 401 failure (empty who), as long as there
// are stored identities or the server explicitly requires a token, the dropdown must stay
// usable or there is no way to recover;
// anonymous mode (anonymous capabilities but no token stored in this browser) must also open the
// dropdown to add an identity and upgrade
const showIdentities = computed(
  () =>
    who.value?.mode !== 'open' && (identities.value.length > 0 || needToken.value || who.value?.mode === 'anonymous'),
)
// Trigger label: prefer the verified identity name, then the local current pointer, otherwise
// not connected; the anonymous identity gets its own label
const identityLabel = computed(() => {
  if (who.value?.mode === 'anonymous') return t('shell.anonymous')
  return who.value?.name ?? currentName.value ?? t('shell.identityNone')
})

// The admin tab is visible to admins only; when the current tab loses visibility (e.g. switching to a lower-privileged identity), fall back to memories
const navItems = computed(() => {
  const items: { key: AdminTab; label: string; icon: typeof Notebook }[] = [
    { key: 'memories', label: t('nav.memories'), icon: Notebook },
    { key: 'tags', label: t('nav.tags'), icon: PriceTag },
  ]
  if (isAdmin.value) items.push({ key: 'admin', label: t('nav.admin'), icon: Key })
  return items
})

watch(navItems, (items) => {
  if (!items.some((item) => item.key === active.value)) active.value = 'memories'
})

// Permanently mounted panels (v-show) are not remounted, so data may be stale when switching
// back: refresh the target panel on switch
interface RefreshablePanel {
  refresh: () => void
}
const memoriesPanel = ref<RefreshablePanel | null>(null)
const tagsPanel = ref<RefreshablePanel | null>(null)
const adminPanel = ref<RefreshablePanel | null>(null)

function refreshActivePanel(): void {
  const panel = {
    memories: memoriesPanel.value,
    tags: tagsPanel.value,
    admin: adminPanel.value,
  }[active.value]
  panel?.refresh()
}

watch(active, () => refreshActivePanel())

// ---- Identity dropdown and token dialog state ----
function refreshIdentities(): void {
  identities.value = listIdentities()
  currentName.value = currentIdentityName()
}

const tokenDialog = ref(false)
const tokenDialogMode = ref<'required' | 'add'>('required')
const tokenInput = ref('')
const tokenError = ref(false)
const checkingToken = ref(false)

/** Resolve the current identity, returning whether a valid identity was obtained. On 401, only
 * the one invalid identity is removed and the token dialog pops — never clear the whole identity
 * table; the remaining identities stay in the dropdown for self-recovery. */
async function resolveIdentity(): Promise<boolean> {
  const r = await fetchWhoAmI()
  if (r.ok) {
    needToken.value = false
    who.value = r.who
    // fetchWhoAmI may have folded a legacy single token into the identity table; sync the local list
    refreshIdentities()
    return true
  }
  if (r.needToken) {
    // The current token was rejected by the server (reset/deleted): remove this invalid identity
    // (with a valid token in the request, the server would not 401)
    needToken.value = true
    const name = currentIdentityName()
    if (name) removeIdentity(name)
    refreshIdentities()
    who.value = null
    tokenDialogMode.value = 'required'
    tokenDialog.value = true
  }
  return false
}

async function switchTo(name: string): Promise<void> {
  switchIdentity(name)
  refreshIdentities()
  who.value = null
  if (await resolveIdentity()) {
    // A successful switch restores the page: collapse the token dialog (whether it is a 401 notice or an unfinished add)
    tokenDialog.value = false
    tokenInput.value = ''
    tokenError.value = false
  }
  refreshActivePanel()
}

async function saveToken(): Promise<void> {
  const token = tokenInput.value.trim()
  if (!token) return
  checkingToken.value = true
  tokenError.value = false
  // Validate the candidate token before storing it, so an invalid token never pollutes the saved identity table
  const r = await fetchWhoAmI(token)
  checkingToken.value = false
  if (r.ok) {
    addIdentity(r.who.name, token)
    refreshIdentities()
    needToken.value = false
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
  // The current identity was deleted: automatically switch to the first remaining one; when none remain, return to the not-connected state
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
  // A 401 means the current token was rejected by the server (reset/deleted): remove it and ask for re-entry
  void resolveIdentity()
}

onMounted(() => {
  refreshIdentities()
  void resolveIdentity()
  window.addEventListener(UNAUTHORIZED_EVENT, onUnauthorized)
})
onUnmounted(() => window.removeEventListener(UNAUTHORIZED_EVENT, onUnauthorized))

// ---- Language (UI copy and Element Plus built-in copy switch together) ----
// Adding a language takes two steps: add a dictionary file in the lib and register it in messages
// + add one entry to the languages list
type LangChoice = 'zh' | 'en'
const LANG_KEY = 'agent-memory-locale'
// nativeName is shown in the language's own name and is not translated with the UI language
const languages = [
  { value: 'zh' as LangChoice, nativeName: '中文' },
  { value: 'en' as LangChoice, nativeName: 'English' },
]
const lang = ref<LangChoice>(readStoredLang())
const epLocale = computed(() => (lang.value === 'zh' ? zhCn : en))
// Sync the initial preference (persisted value or browser language) into the library's i18n instance
setMemoryUILocale(lang.value)

function setLang(value: LangChoice) {
  lang.value = value
}

// With no persisted preference, choose by browser language (same rule as the lib's auto detection)
function detectLang(): LangChoice {
  return navigator.language?.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

function readStoredLang(): LangChoice {
  try {
    const v = localStorage.getItem(LANG_KEY)
    if (v === 'zh' || v === 'en') return v
  } catch {
    /* If storage is unreadable (e.g. private mode), fall through */
  }
  return detectLang()
}

watch(lang, (value) => {
  try {
    localStorage.setItem(LANG_KEY, value)
  } catch {
    /* If storage is unavailable (e.g. private mode), just give up */
  }
  setMemoryUILocale(value)
})

// ---- Theme: light / dark / follow system ----
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
    /* If storage is unavailable (e.g. private mode), just give up */
  }
  applyTheme()
})

onMounted(() => {
  applyTheme()
  // In follow-system mode, listen for system theme changes
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
/* Clamp overlay widths: fixed pixel widths (the 440px token dialog etc.) must not overflow the viewport on phones */
@media (max-width: 720px) {
  .el-dialog {
    max-width: calc(100vw - 24px);
  }
  .el-message-box {
    max-width: calc(100vw - 24px);
  }
}
</style>

<style scoped>
.shell {
  min-height: 100%;
  display: flex;
  flex-direction: column;
}

/* Header: brand + navigation + preference controls. Same color as the page body (no separate band), no border */
.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--el-bg-color-page);
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
/* Title button: opens the server overview; negative margin cancels button padding to keep the nav baseline aligned */
.brand-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: -4px -8px;
  padding: 4px 8px;
  border: none;
  border-radius: var(--el-border-radius-base);
  background: transparent;
  color: inherit;
  font: inherit;
  letter-spacing: inherit;
  cursor: pointer;
}
.brand-btn:hover {
  background: var(--el-fill-color);
}
/* Brand icon: rounded block with the brand-green gradient (Modrinth's --brand-gradient-bg) */
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
/* Navigation: segmented pill (same as the theme switch on the right) - the container gets a recessed fill, the active item floats */
.nav {
  display: flex;
  align-items: center;
  gap: 2px;
  min-width: 0;
  padding: 2px;
  border-radius: 999px;
  background: var(--el-fill-color);
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 14px;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--el-text-color-regular);
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
}
.nav-item:hover {
  color: var(--el-text-color-primary);
}
.nav-item.active {
  background: var(--el-bg-color);
  color: var(--el-color-primary);
  font-weight: 600;
  box-shadow: var(--el-box-shadow-light);
}
.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: auto;
}

/* Theme segment: iOS-style capsule, active item floats */
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

/* Language dropdown trigger: capsule styled like the theme segment */
.lang-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 10px;
  border: none;
  border-radius: 999px;
  background: var(--el-fill-color);
  color: var(--el-text-color-regular);
  cursor: pointer;
}
.lang-btn:hover {
  color: var(--el-text-color-primary);
  background: var(--el-fill-color-dark);
}
.lang-icon {
  display: block;
}
.lang-option {
  flex: 1;
  margin-right: 12px;
}
.lang-check {
  color: var(--el-color-primary);
}

/* Identity switcher: clickable identity name inside the brand row (Agent Memory / identity name) */
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
/* Dropdown rows: identity name + token suffix hint on the left, checkmark and delete on the right */
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
/* Add identity: one centered plus sign fills the row */
.id-add {
  justify-content: center;
}

/* Token dialog copy */
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

/* Content area: full-width padding with a centered container */
.content {
  flex: 1;
  width: 100%;
  max-width: 1240px;
  margin: 0 auto;
  padding: 24px;
  box-sizing: border-box;
}

/* ---- Mobile adaptation ---- */
/* <=720px: header wraps to two rows - first the brand row + preference controls, then navigation (horizontal scroll if it doesn't fit).
   A single-row layout inevitably overflows at phone widths (nav items pushed off-viewport, overlapping the theme controls) */
@media (max-width: 720px) {
  .topbar-inner {
    height: auto;
    flex-wrap: wrap;
    padding: 0 12px;
    gap: 0 12px;
  }
  .brand {
    order: 1;
    flex: 1;
    min-width: 0;
  }
  .actions {
    order: 2;
    padding: 8px 0;
  }
  .nav {
    order: 3;
    flex-basis: 100%;
    height: 44px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .nav::-webkit-scrollbar {
    display: none;
  }
  .nav-item {
    flex-shrink: 0;
    height: 34px;
  }
  .identity-btn {
    max-width: 150px;
  }
  .content {
    padding: 12px;
  }
}
/* <=560px: collapse the brand name to just the icon (still opens the server overview on click); cap the identity name width so it isn't squeezed */
@media (max-width: 560px) {
  .brand-name,
  .brand-sep {
    display: none;
  }
  .identity-btn {
    max-width: 130px;
  }
  .lang-btn {
    padding: 0 9px;
    gap: 3px;
  }
}
</style>
