// 管理面板的状态：数据体检、备份导入导出、语义搜索补跑（admin 专属端点）。动作失败时抛出 Error，由面板层统一 toast。
import { ref } from 'vue'
import { useApiClient } from '../api/client'
import { backfillEmbeddings, exportBackup, importBackup, runDoctorRemote } from '../api/ops'
import { t } from '../i18n'
import type { DoctorResp, ImportResp } from '../types'

export function useAdmin() {
  const client = useApiClient()

  const doctor = ref<DoctorResp>({ ok: true, issues: [] })
  const doctorRan = ref(false)
  const doctorLoading = ref(false)
  const exporting = ref(false)
  const importing = ref(false)
  const backfilling = ref(false)

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

  /** 解析备份文件并导入（仅空库可用，服务端把关）。返回导入计数。 */
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
      return await importBackup(client, dump)
    } finally {
      importing.value = false
    }
  }

  /**
   * 循环补跑语义搜索向量直到清零（每次请求一小批，服务端有界、前端串行）。
   * 返回补跑总条数；未配置 / 服务报错直接抛出，由面板层 toast。
   */
  async function backfill(): Promise<number> {
    backfilling.value = true
    try {
      let total = 0
      for (;;) {
        const out = await backfillEmbeddings(client)
        if (!out.configured) throw new Error(t('access.embeddingNotConfigured'))
        if (out.error) throw new Error(out.error)
        total += out.processed ?? 0
        if ((out.processed ?? 0) === 0) break
      }
      return total
    } finally {
      backfilling.value = false
    }
  }

  return {
    doctor,
    doctorRan,
    doctorLoading,
    exporting,
    importing,
    backfilling,
    runDoctor,
    exportData,
    importFile,
    backfill,
  }
}

export type AdminStore = ReturnType<typeof useAdmin>
