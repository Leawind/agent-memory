// 管理面板的状态：数据体检、备份导入导出（admin 专属端点）。动作失败时抛出 Error，由面板层统一 toast。
import { ref } from 'vue'
import { useApiClient } from '../api/client'
import { exportBackup, importBackup, runDoctorRemote } from '../api/ops'
import { t } from '../i18n'
import type { DoctorResp, ImportResp } from '../types'

export function useAdmin() {
  const client = useApiClient()

  const doctor = ref<DoctorResp>({ ok: true, issues: [] })
  const doctorRan = ref(false)
  const doctorLoading = ref(false)
  const exporting = ref(false)
  const importing = ref(false)

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

  return {
    doctor,
    doctorRan,
    doctorLoading,
    exporting,
    importing,
    runDoctor,
    exportData,
    importFile,
  }
}

export type AdminStore = ReturnType<typeof useAdmin>
