// happy-dom 没有 ResizeObserver，Element Plus 的 el-table 等组件依赖它
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
globalThis.ResizeObserver = ResizeObserverStub

// 测试断言以中文文案为准：固定库语言，避免宿主环境 navigator.language 影响探测结果
import { setMemoryUILocale } from './i18n'
setMemoryUILocale('zh')
