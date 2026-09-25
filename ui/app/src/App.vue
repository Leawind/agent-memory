<template>
  <el-config-provider :locale="epLocale">
    <div class="shell">
      <header class="topbar">
        <div class="topbar-inner">
          <div class="brand">
            <span class="brand-mark">
              <el-icon :size="16"><Collection /></el-icon>
            </span>
            <span class="brand-name">agent-memory</span>
          </div>

          <!-- 顶部导航：面板常驻挂载，切换不销毁、不重新请求 -->
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
          <MemoriesPanel v-show="active === 'memories'" />
          <TagsPanel v-show="active === 'tags'" />
          <OpsPanel v-show="active === 'ops'" />
        </div>
      </main>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  ArrowDown,
  Check,
  Collection,
  Monitor,
  Moon,
  Notebook,
  Odometer,
  PriceTag,
  Sunny,
} from '@element-plus/icons-vue'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import en from 'element-plus/es/locale/lang/en'
import { MemoriesPanel, OpsPanel, TagsPanel, provideMemoryUI, setMemoryUILocale, t } from '@agent-memory/ui'

// 同源部署：API 走默认配置（baseUrl = ''）
provideMemoryUI({})

type AdminTab = 'memories' | 'tags' | 'ops'
const active = ref<AdminTab>('memories')
const navItems = computed(() => [
  { key: 'memories' as const, label: t('nav.memories'), icon: Notebook },
  { key: 'tags' as const, label: t('nav.tags'), icon: PriceTag },
  { key: 'ops' as const, label: t('nav.ops'), icon: Odometer },
])

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
