import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import dts from 'vite-plugin-dts'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

// 组件库构建（lib mode）：产物 ESM + 单文件 style.css，vue/element-plus/marked 等全部外置，
// 由宿主应用提供（peerDependencies）。CSS 不会自动注入，宿主需另行 import '@agent-memory/ui/style.css'。
// 模板中的 el-* 组件由 unplugin-vue-components 在构建期注入具名导入（不注入 EP 样式，
// 样式由宿主引入），因此宿主无需 app.use(ElementPlus) 全局注册。
export default defineConfig({
  plugins: [
    vue(),
    Components({ resolvers: [ElementPlusResolver({ importStyle: false })], dts: false }),
    dts({
      tsconfigPath: './tsconfig.json',
      cleanVueFileName: true,
      include: ['src'],
      exclude: ['src/**/*.test.ts', 'src/test-setup.ts'],
    }),
  ],
  build: {
    lib: {
      entry: 'src/index.ts',
      formats: ['es'],
      fileName: 'index',
      // CSS 产物名固定为 style.css（与 package.json exports 对应）
      cssFileName: 'style',
    },
    rollupOptions: {
      external: [
        /^vue($|\/)/,
        /^element-plus($|\/)/,
        /^@element-plus\//,
        /^marked($|\/)/,
        /^dompurify($|\/)/,
        /^vue-i18n($|\/)/,
      ],
    },
  },
  test: {
    environment: 'happy-dom',
    setupFiles: ['./src/test-setup.ts'],
    // vitest 默认按 CPU 核数开满线程池，跑测试时其他程序会卡；用例很轻，限两个线程
    pool: 'threads',
    poolOptions: { threads: { maxThreads: 2, minThreads: 1 } },
  },
})
