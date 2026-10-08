// Run from main/: node scripts/search-eval.ts [fixture.json]
// AGENT_MEMORY_BIN selects a built executable; otherwise cargo metadata locates debug output.
// Always creates its own temporary database/server. No production endpoint or model credentials.
import { execFileSync, spawn } from 'node:child_process'
import { once } from 'node:events'
import { mkdtemp, readFile, realpath, rm } from 'node:fs/promises'
import { createServer } from 'node:net'
import { tmpdir } from 'node:os'
import { basename, dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

interface Memory {
  key: string
  summary: string
  content: string
  tags: string[]
  age_days: number
  kind?: string
  archived?: boolean
  expired?: boolean
  uses?: number
}
interface Case {
  name: string
  query: string
  tag_expr?: string
  state?: string
  relevance: Record<string, number>
}
interface Fixture {
  description: string
  memories: Memory[]
  derivations: { name: string; expression: string }[]
  cases: Case[]
}
interface SearchResponse {
  results: { id: string; summary: string; content?: unknown; lifecycle: { state: string } }[]
}

const budgets = [2, 5, 10, 20]
const repetitions = 5
const at = 5
const mean = (values: number[]) => values.reduce((sum, value) => sum + value, 0) / Math.max(1, values.length)
function quality(keys: string[], relevance: Record<string, number>, k: number) {
  const positives = Object.values(relevance).filter((value) => value > 0).length
  const grades = keys.slice(0, k).map((key) => relevance[key] ?? 0)
  const dcg = (values: number[]) =>
    values.reduce((sum, grade, index) => sum + (2 ** grade - 1) / Math.log2(index + 2), 0)
  const ideal = dcg(
    Object.values(relevance)
      .sort((a, b) => b - a)
      .slice(0, k),
  )
  return {
    recall: positives ? grades.filter((grade) => grade > 0).length / positives : null,
    ndcg: ideal ? dcg(grades) / ideal : null,
  }
}
function percentile(values: number[], proportion: number) {
  const sorted = [...values].sort((a, b) => a - b)
  return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * proportion) - 1)] ?? 0
}
function checkMetrics() {
  const perfect = quality(['a', 'b'], { a: 3, b: 1 }, 2)
  if (perfect.recall !== 1 || perfect.ndcg !== 1) throw new Error('metric sanity check failed')
  const missing = quality(['b'], { a: 3, b: 1 }, 1)
  if (missing.recall !== 0.5 || missing.ndcg !== 1 / 7) throw new Error('metric cutoff sanity check failed')
}

async function run() {
  checkMetrics()
  const fixturePath = process.argv[2] ?? fileURLToPath(new URL('./search-eval-fixture.json', import.meta.url))
  const fixture = JSON.parse(await readFile(fixturePath, 'utf8')) as Fixture
  if (
    fixture.memories.length > 50 ||
    new Set(fixture.memories.map((memory) => memory.summary)).size !== fixture.memories.length
  ) {
    throw new Error('fixture must have at most 50 memories with unique summaries')
  }
  let binary = process.env.AGENT_MEMORY_BIN
  if (!binary) {
    const metadata = JSON.parse(
      execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], { encoding: 'utf8' }),
    ) as { target_directory: string }
    binary = join(
      metadata.target_directory,
      'debug',
      process.platform === 'win32' ? 'agent-memory.exe' : 'agent-memory',
    )
  }
  binary = await realpath(binary)
  const temp = await mkdtemp(join(tmpdir(), 'agent-memory-eval-'))
  const listener = createServer()
  listener.listen(0, '127.0.0.1')
  await once(listener, 'listening')
  const address = listener.address()
  if (!address || typeof address === 'string') throw new Error('port allocation failed')
  const port = address.port
  await new Promise<void>((done) => listener.close(() => done()))
  const server = spawn(
    binary,
    ['serve', '--host', '127.0.0.1', '--port', String(port), '--db', join(temp, 'eval.db')],
    { windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] },
  )
  let diagnostics = ''
  server.stderr.on('data', (chunk: Buffer) => {
    diagnostics = (diagnostics + chunk.toString()).slice(-32768)
  })
  let spawnError: Error | undefined
  server.on('error', (error) => {
    spawnError = error
  })
  const base = `http://127.0.0.1:${port}`
  async function api<T>(method: string, path: string, body?: unknown): Promise<T> {
    const response = await fetch(base + path, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: AbortSignal.timeout(15000),
    })
    const text = await response.text()
    if (!response.ok) throw new Error(`${method} ${path}: HTTP ${response.status} ${text}`)
    return JSON.parse(text) as T
  }
  try {
    let ready = false
    for (let attempt = 0; attempt < 100; attempt++) {
      if (spawnError || server.exitCode !== null)
        throw new Error(`test server did not start: ${spawnError?.message ?? diagnostics}`)
      try {
        await api('GET', '/health')
        ready = true
        break
      } catch {
        await new Promise((done) => setTimeout(done, 50))
      }
    }
    if (!ready) throw new Error(`test server readiness timed out: ${diagnostics}`)
    const now = Math.floor(Date.now() / 1000)
    const names = [...new Set(fixture.memories.flatMap((memory) => memory.tags))]
    const tagIds = Object.fromEntries(names.map((name, index) => [name, String(index + 1)]))
    const memories = Object.fromEntries(
      fixture.memories.map((memory, index) => {
        const time = now - Math.round(memory.age_days * 86400)
        const events = Array.from({ length: memory.uses ?? 0 }, (_, event) => ({
          actor: 'evaluation',
          event_key: `use-${event}`,
          kind: 'use',
          occurred_at: now - event * 61,
          weight: 1,
        }))
        return [
          `m${index + 1}`,
          {
            summary: memory.summary,
            content: memory.content,
            tags: memory.tags.map((name) => tagIds[name]),
            created_at: time,
            updated_at: time,
            lifecycle: {
              kind: memory.kind ?? 'fact',
              pinned: false,
              archived_at: memory.archived ? now : null,
              expires_at: memory.expired ? now - 1 : null,
            },
            access: { scoring_version: 1, events, buckets: [] },
          },
        ]
      }),
    )
    await api('POST', '/api/import', {
      tags: Object.fromEntries(names.map((name) => [tagIds[name], { name }])),
      memories,
      tag_rules: { constraints: [], derivations: fixture.derivations },
    })
    const listing = await api<{ memories: { id: string; summary: string }[] }>(
      'GET',
      '/api/memories?state=all&limit=50',
    )
    const bySummary = new Map(fixture.memories.map((memory) => [memory.summary, memory.key]))
    const byId = new Map(listing.memories.map((memory) => [memory.id, bySummary.get(memory.summary)!]))
    const variants = [
      { name: 'content_tag_rrf', freshness_weight: 0, reinforcement_weight: 0 },
      { name: 'content_tag_lifecycle_rrf', freshness_weight: 0.2, reinforcement_weight: 0.1 },
    ]
    const report: unknown[] = []
    for (const variant of variants) {
      await api('PUT', '/api/settings', {
        lifecycle_policy: {
          freshness_weight: variant.freshness_weight,
          reinforcement_weight: variant.reinforcement_weight,
        },
      })
      const cases = []
      const timings: number[] = []
      for (const entry of fixture.cases) {
        const params = new URLSearchParams({
          query: entry.query,
          mode: 'keyword',
          limit: '50',
          state: entry.state ?? 'active',
        })
        if (entry.tag_expr) params.set('tag_expr', entry.tag_expr)
        // This server parses URL values without form-style '+' decoding.
        const path = `/api/memories?${params.toString().replaceAll('+', '%20')}`
        await api('GET', path) // warm-up; every timed search recomputes, never sends a cursor
        let result!: SearchResponse
        for (let sample = 0; sample < repetitions; sample++) {
          const start = performance.now()
          result = await api<SearchResponse>('GET', path)
          timings.push(performance.now() - start)
        }
        for (const hit of result.results) {
          if (hit.content !== undefined || hit.lifecycle.state !== (entry.state ?? 'active'))
            throw new Error(`${entry.name}: disclosure/state boundary violated`)
        }
        const keys = result.results.map((memory) => byId.get(memory.id)!)
        if (keys.includes('popular-unrelated') || (Object.keys(entry.relevance).length === 0 && keys.length))
          throw new Error(`${entry.name}: unrelated memory recalled`)
        const metrics = quality(keys, entry.relevance, at)
        if (metrics.recall !== null && metrics.recall < 1)
          throw new Error(`${entry.name}: regression fixture recall@${at} fell below 1`)
        cases.push({
          name: entry.name,
          ...metrics,
          top_keys: keys.slice(0, at),
          pool_coverage: Object.fromEntries(
            budgets.map((budget) => [budget, quality(keys, entry.relevance, budget).recall]),
          ),
        })
      }
      const scored = cases.filter((entry) => entry.recall !== null)
      report.push({
        variant: variant.name,
        recall_at_5: mean(scored.map((entry) => entry.recall!)),
        ndcg_at_5: mean(scored.map((entry) => entry.ndcg!)),
        latency_ms: { p50: percentile(timings, 0.5), p95: percentile(timings, 0.95), samples: timings.length },
        pool_coverage_recall: Object.fromEntries(
          budgets.map((budget) => [budget, mean(scored.map((entry) => entry.pool_coverage[budget]!))]),
        ),
        cases,
      })
    }
    console.log(
      JSON.stringify(
        {
          description: fixture.description,
          corpus_size: fixture.memories.length,
          case_count: fixture.cases.length,
          note: 'Pool coverage measures candidate availability before a reranker. Timings are local warm synthetic measurements; no model calls. Scores are not evidence of production model quality.',
          results: report,
        },
        null,
        2,
      ),
    )
  } finally {
    if (server.exitCode === null && !spawnError) {
      const exited = once(server, 'exit')
      server.kill()
      await exited
    }
    if (dirname(resolve(temp)) !== resolve(tmpdir()) || !basename(temp).startsWith('agent-memory-eval-'))
      throw new Error('refusing to remove an unexpected path')
    await rm(temp, { recursive: true, force: true })
  }
}

run().catch((error: Error) => {
  console.error(error.stack)
  process.exitCode = 1
})
