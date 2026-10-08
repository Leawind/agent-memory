interface CandidateFields {
  modelId: string
  baseUrl: string
  model: string
  minSimilarity?: string
}

/** Only ID, endpoint and API model are required; empty optional fields are valid. */
export function candidateFieldErrors(row: CandidateFields, rows: CandidateFields[]) {
  const id = row.modelId.trim()
  const baseUrl = row.baseUrl.trim()
  let invalidUrl = !baseUrl
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
    baseUrl: !!invalidUrl,
    model: !row.model.trim(),
    minSimilarity: floor !== '' && (!Number.isFinite(similarity) || similarity < 0 || similarity > 1),
  }
}
