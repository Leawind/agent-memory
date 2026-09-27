// 服务概况的状态：统计与版本（只读，无需 admin）。reload 由 OpsDialog 打开时调用，
// 动作失败时抛出 Error，由调用层统一 toast。admin 专属动作（体检/导入导出）在 useAdmin。
import { computed, ref } from 'vue'
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
    try {
      version.value = (await getHealth(client)).version ?? ''
    } catch {
      /* 版本号取不到不阻塞统计 */
    }
    stats.value = await getStats(client)
    stats.value.version = version.value
  }

  return {
    stats,
    version,
    sizeText,
    reload,
  }
}

export type OpsStore = ReturnType<typeof useOps>
