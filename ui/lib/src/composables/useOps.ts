// 运维面板的状态：统计、体检、导入导出。动作失败时抛出 Error，由面板层统一 toast。
import { computed, onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { exportBackup, getHealth, getStats, importBackup, runDoctorRemote } from '../api/ops'
import { formatSize } from '../format'
import { t } from '../i18n'
import type { DoctorResp, ImportResp, StatsInfo } from '../types'

export function useOps() {
  const client = useApiClient()

  const stats = ref<Partial<StatsInfo>>({})
  const version = ref('')
  const doctor = ref<DoctorResp>({ ok: true, issues: [] })
  const doctorRan = ref(false)
  const doctorLoading = ref(false)
  const exporting = ref(false)
  const importing = ref(false)

  const sizeText = computed(() => formatSize(stats.value.file_size))

  async function reload() {
    stats.value = await getStats(client)
    stats.value.version = version.value
  }

  async function runDoctor() {
    doctorLoading.value = true
    try {
      doctor.value = await runDoctorRemote(client)
      doctorRan.value = true
    } finally {
      doctorLoading.value = false
    }
  }

  /** 导出备份 JSON 并触发浏览器下载 */
  async function exportData() {
    exporting.value = true
    try {
      const blob = await exportBackup(client)
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'agent-memory-export.json'
      a.click()
      URL.revokeObjectURL(url)
    } finally {
      exporting.value = false
    }
  }

  /** 解析备份文件并导入（仅空库可用，服务端把关）。成功后刷新统计，返回导入计数。 */
  async function importFile(file: File): Promise<ImportResp> {
    importing.value = true
    try {
      const text = await file.text()
      let dump: unknown
      try {
        dump = JSON.parse(text)
      } catch {
        throw new Error(t('errors.invalidBackup'))
      }
      const imported = await importBackup(client, dump)
      await reload()
      return imported
    } finally {
      importing.value = false
    }
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
    doctor,
    doctorRan,
    doctorLoading,
    exporting,
    importing,
    sizeText,
    reload,
    runDoctor,
    exportData,
    importFile,
  }
}

export type OpsStore = ReturnType<typeof useOps>
