// Service overview state: stats and version (read-only, no admin needed). reload is called by
// OpsDialog when it opens; failed actions throw an Error, toasted uniformly at the calling layer.
// Admin-only actions (doctor / import & export) live in useAdmin.
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
      /* A missing version must not block the stats */
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
