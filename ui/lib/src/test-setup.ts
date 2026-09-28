// happy-dom has no ResizeObserver, which Element Plus components such as el-table rely on
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
globalThis.ResizeObserver = ResizeObserverStub

// Test assertions rely on Chinese UI copy: pin the library language so the host
// environment's navigator.language does not affect language detection
import { setMemoryUILocale } from './i18n'
setMemoryUILocale('zh')
