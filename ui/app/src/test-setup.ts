// happy-dom 没有 ResizeObserver，Element Plus 的 el-table 等组件依赖它
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
globalThis.ResizeObserver = ResizeObserverStub
