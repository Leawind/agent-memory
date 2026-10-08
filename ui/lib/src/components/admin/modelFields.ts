interface CandidateFields {
  enabled: boolean
  modelId: string
  name: string
  baseUrl: string
  model: string
  minSimilarity?: string
}

/** Empty endpoint/model fields are allowed only for disabled drafts, matching settings validation. */
export function candidateFieldErrors(row: CandidateFields, rows: CandidateFields[]) {
  const id = row.modelId.trim()
  const baseUrl = row.baseUrl.trim()
  let invalidUrl = row.enabled && !baseUrl
  if (baseUrl) {
    try {
      const url = new URL(baseUrl)
      invalidUrl = !['http:', 'https:'].includes(url.protocol)
    } catch {
      invalidUrl = true
    }
  }
  const floor = row.minSimilarity?.trim() ?? ''
  const similarity = Number(floor)
  return {
    modelId: !id || rows.filter((r) => r.modelId.trim() === id).length > 1,
    name: !row.name.trim(),
    baseUrl: !!invalidUrl,
    model: row.enabled && !row.model.trim(),
    minSimilarity: floor !== '' && (!Number.isFinite(similarity) || similarity < 0 || similarity > 1),
  }
}
