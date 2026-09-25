import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import type { Component } from 'vue'
import App from './App.vue'

const app = createApp(App)
app.use(ElementPlus, { locale: zhCn })
for (const [name, component] of Object.entries(ElementPlusIconsVue as Record<string, Component>)) {
  app.component(name, component)
}
app.mount('#app')
