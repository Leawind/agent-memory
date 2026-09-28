// Container width observation: when a panel is embedded in a host container of any size, use
// ResizeObserver to sense the actual width and switch layouts by thresholds.
// compact (<960px) hides low-priority columns; narrow (<720px) narrows dialogs.
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

  // width of 0 means never observed (e.g. stubbed in tests): never enter a compact layout
  const compact = computed(() => width.value > 0 && width.value < 960)
  const narrow = computed(() => width.value > 0 && width.value < 720)
  return { width, compact, narrow }
}
