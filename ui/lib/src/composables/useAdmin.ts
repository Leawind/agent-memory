// Admin panel state: data doctor, backup import/export (admin-only endpoints). Failed actions
// throw an Error; the panel layer shows the toast uniformly.
import { ref } from 'vue'
import { useApiClient } from '../api/client'
import { exportBackup, importBackup, runDoctorRemote } from '../api/ops'
import { t } from '../i18n'
import type { DoctorResp, ImportResp } from '../types'

export function useAdmin() {
  const client = useApiClient()

  const doctor = ref<DoctorResp>({ ok: true, checks: [] })
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

  /** Export the backup JSON (optionally limited to memories matching a tag expression) and trigger a browser download */
  async function exportData(predicate?: string) {
    exporting.value = true
    try {
      const blob = await exportBackup(client, predicate)
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

  /**
   * Parse a backup file and import it. Restore (the default) targets an empty database;
   * `merge` imports incrementally and `predicate` filters inside the file — both enforced
   * server-side. Returns the import counts.
   */
  async function importFile(file: File, options?: { merge?: boolean; predicate?: string }): Promise<ImportResp> {
    importing.value = true
    try {
      const text = await file.text()
      let dump: unknown
      try {
        dump = JSON.parse(text)
      } catch {
        throw new Error(t('errors.invalidBackup'))
      }
      return await importBackup(client, dump, options)
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
