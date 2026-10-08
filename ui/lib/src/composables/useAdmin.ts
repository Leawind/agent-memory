// Admin panel state: data doctor, backup import/export, semantic search backfill (admin-only
// endpoints). Failed actions throw an Error; the panel layer shows the toast uniformly.
import { ref } from 'vue'
import { useApiClient } from '../api/client'
import { backfillEmbeddings, exportBackup, importBackup, runDoctorRemote } from '../api/ops'
import { t } from '../i18n'
import type { DoctorResp, ImportResp } from '../types'

export function useAdmin() {
  const client = useApiClient()

  const doctor = ref<DoctorResp>({ ok: true, checks: [] })
  const doctorRan = ref(false)
  const doctorLoading = ref(false)
  const exporting = ref(false)
  const importing = ref(false)
  const backfilling = ref(false)
  /** Set by `cancelBackfill`: the drain loop checks it between batches */
  const backfillCancelled = ref(false)

  /** Interrupt a running backfill loop (the batch in flight still finishes and is stored) */
  function cancelBackfill(): void {
    backfillCancelled.value = true
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

  /** Export the backup JSON and trigger a browser download */
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

  /** Parse a backup file and import it (only allowed on an empty database, enforced server-side). Returns the import counts. */
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
   * Loop backfilling semantic search vectors until the queue drains (a small batch per request;
   * bounded server-side, serial on the frontend). With `modelKey`, drains that specific cache
   * identity instead of the active candidate's.
   * `onProgress` reports cumulative done and the estimated total after each batch, so callers
   * can show real progress instead of a bare spinner.
   * `cancelBackfill` stops the loop after the batch already in flight (that batch always lands —
   * interruption is a client-side stop, never a half-written batch).
   * Returns the number backfilled plus whether the loop was interrupted; unconfigured / server
   * errors are thrown directly, toasted at the panel layer.
   */
  async function backfill(
    modelKey?: string,
    onProgress?: (done: number, total: number) => void,
  ): Promise<{ done: number; cancelled: boolean }> {
    backfilling.value = true
    backfillCancelled.value = false
    try {
      let done = 0
      for (;;) {
        const out = await backfillEmbeddings(client, modelKey)
        if (!out.configured) throw new Error(t('access.embeddingNotConfigured'))
        if (out.error) throw new Error(out.error)
        done += out.processed ?? 0
        onProgress?.(done, done + (out.remaining ?? 0))
        if (backfillCancelled.value) return { done, cancelled: true }
        if ((out.processed ?? 0) === 0) break
      }
      return { done, cancelled: false }
    } finally {
      backfilling.value = false
      backfillCancelled.value = false
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
    cancelBackfill,
  }
}

export type AdminStore = ReturnType<typeof useAdmin>
