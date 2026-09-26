// 运维面板的状态：统计与版本概况（只读，无需 admin）。动作失败时抛出 Error，由面板层统一 toast。
// admin 专属动作（体检/导入导出）在 useAdmin。
import { computed, onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { getHealth, getStats } from '../api/ops'
import { formatSize } from '../format'
import type { StatsInfo } from '../types'

export function useOps() {
  const client = useApiClient()

  const stats = ref<Partial<StatsInfo>>({})
  const version = ref('')

  const sizeText = computed(() => formatSize(stats.value.file_size))

  async function reload() {
    stats.value = await getStats(client)
    stats.value.version = version.value
  }

  onMounted(async () => {
    try {
      version.value = (await getHealth(client)).version ?? ''
    } catch {
      /* 版本号取不到不阻塞统计 */
    }
    await reload().catch(() => {}) // 首屏错误由调用方 toast（面板层包装）
  })

  return {
    stats,
    version,
    sizeText,
    reload,
  }
}

export type OpsStore = ReturnType<typeof useOps>
