// Standalone site entry: full Element Plus registration + light/dark theme variables + component library styles.
// All business components come from @agent-memory/ui; this package only does host assembly.
import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './theme.css'
import '@agent-memory/ui/style.css'
import App from './App.vue'

createApp(App).use(ElementPlus, { locale: zhCn }).mount('#app')
