// 容器宽度观测：面板被嵌入宿主任意尺寸容器时，用 ResizeObserver 感知实际宽度，
// 窄容器切换 compact 布局（隐藏低优先级列、弹层收窄）。
import { computed, onMounted, onUnmounted, watch, ref } from 'vue'
import type { Ref } from 'vue'

export function useContainerWidth(target: Ref<HTMLElement | null>, compactBelow = 720) {
  const width = ref(0)
  let observer: ResizeObserver | null = null

  function observe(el: HTMLElement | null) {
    observer?.disconnect()
    observer = null
    if (!el || typeof ResizeObserver === 'undefined') return
    observer = new ResizeObserver((entries) => {
      width.value = entries[0]?.contentRect.width ?? 0
    })
    observer.observe(el)
  }

  onMounted(() => observe(target.value))
  watch(target, (el) => observe(el))
  onUnmounted(() => observer?.disconnect())

  // width 为 0 表示从未观测到（如测试环境 stub）：不进入 compact
  return { width, compact: computed(() => width.value > 0 && width.value < compactBelow) }
}
