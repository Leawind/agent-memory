import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// 开发时代理到本机运行中的服务器；构建产物由服务器二进制直接内嵌托管
export default defineConfig({
  plugins: [vue()],
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
