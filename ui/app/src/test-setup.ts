// happy-dom has no ResizeObserver, which Element Plus components such as el-table rely on
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
globalThis.ResizeObserver = ResizeObserverStub
