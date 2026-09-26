// 容器宽度观测：面板被嵌入宿主任意尺寸容器时，用 ResizeObserver 感知实际宽度，
// 按阈值切换布局。compact（<960px）隐藏低优先级列；narrow（<720px）弹层收窄。
import { computed, onMounted, onUnmounted, watch, ref } from 'vue'
import type { Ref } from 'vue'

export function useContainerWidth(target: Ref<HTMLElement | null>) {
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

  // width 为 0 表示从未观测到（如测试环境 stub）：不进入任何紧凑布局
  const compact = computed(() => width.value > 0 && width.value < 960)
  const narrow = computed(() => width.value > 0 && width.value < 720)
  return { width, compact, narrow }
}
