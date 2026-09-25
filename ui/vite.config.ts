import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// 开发时代理到本机运行中的服务器；构建产物由服务器二进制直接内嵌托管
export default defineConfig({
  plugins: [vue()],
  build: {
    // 单文件内嵌的管理界面（Element Plus 全量 + marked/dompurify），
    // 一次性加载，不做代码分割，只压掉无意义的 chunk 警告
    chunkSizeWarningLimit: 1500,
  },
  server: {
    proxy: {
      '/api': `http://127.0.0.1:8899`,
      '/mcp': `http://127.0.0.1:8899`,
    },
  },
  test: {
    environment: 'happy-dom',
    setupFiles: ['./src/test-setup.ts'],
  },
})
