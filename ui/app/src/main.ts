// 独立站入口：Element Plus 全量注册 + 亮/暗主题变量 + 组件库样式。
// 业务组件全部来自 @agent-memory/ui，本包只做宿主装配。
import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import './theme.css'
import '@agent-memory/ui/style.css'
import App from './App.vue'

createApp(App).use(ElementPlus, { locale: zhCn }).mount('#app')
