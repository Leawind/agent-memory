// Element Plus component texts (pagination, dialog buttons, table empty state, ...)
// follow the same locale as the library's own dictionaries. Top-level components wrap
// themselves in an ElConfigProvider, so hosts only need this library — no EP locale setup.
import { computed } from 'vue'
import en from 'element-plus/es/locale/lang/en'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import { currentMemoryUILocale } from './index'

export const elementPlusLocale = computed(() => (currentMemoryUILocale() === 'zh' ? zhCn : en))
