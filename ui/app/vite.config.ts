import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

// 独立站薄壳：业务组件全部来自 @agent-memory/ui（workspace 依赖，须先构建 lib）。
// 产物输出到 ui/dist，由 Rust 侧 rust-embed 编译期嵌入二进制。
// 开发时代理到本机运行中的服务器。
export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
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
