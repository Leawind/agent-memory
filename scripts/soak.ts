// Mixed-load soak script: writers + readers + tag operations hitting two server
// processes concurrently (sharing one database), verifying long-run stability of the
// read/write transaction split (zero data loss, zero server-side errors).
//
// Usage (start two servers sharing the same --db first; Node >= 24 can run .ts directly):
//   node soak.ts <port1> <port2>      # 60 seconds by default
// When the servers have token auth enabled, inject the token via SOAK_TOKEN
// (all requests automatically carry the Bearer header):
//   SOAK_TOKEN=xxx node soak.ts <port1> <port2>
// Failure classification: an ECONNREFUSED storm = client concurrency exceeds the accept
// backlog (tool overload, not a server defect); HTTP 5xx / SQLITE_BUSY strings = real
// server-side problems.
import http from 'node:http'

const PORTS = [Number(process.argv[2]), Number(process.argv[3])]
const DURATION_MS = 60_000
const MAX_SOCKETS = 4
const ops = { create: 0, search: 0, list: 0, tag: 0 }
const fails = { refused: 0, reset: 0, status4: 0, status5: 0, other: 0 }
const samples: string[] = []

const agent = new http.Agent({ keepAlive: true, maxSockets: MAX_SOCKETS })

interface HttpResult {
  status?: number
  body?: string
  error?: Error
}

function req(method: string, port: number, path: string, body?: unknown): Promise<HttpResult> {
  return new Promise((resolve) => {
    const data = body !== undefined ? JSON.stringify(body) : null
    const headers: Record<string, number | string> = data
      ? { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(data) }
      : {}
    // Authenticated servers: every request carries the Bearer token
    if (process.env.SOAK_TOKEN) headers.Authorization = `Bearer ${process.env.SOAK_TOKEN}`
    const r = http.request(
      {
        host: '127.0.0.1',
        port,
        method,
        path,
        agent,
        headers,
      },
      (res) => {
        const chunks: Buffer[] = []
        res.on('data', (x: Buffer) => chunks.push(x))
        res.on('end', () => resolve({ status: res.statusCode, body: Buffer.concat(chunks).toString() }))
      },
    )
    r.on('error', (e: Error) => resolve({ error: e }))
    if (data) r.write(data)
    r.end()
  })
}

function recordFail(label: string, r: HttpResult): void {
  if (r.error) {
    const code = (r.error as NodeJS.ErrnoException).code ?? 'unknown'
    if (code === 'ECONNREFUSED') fails.refused++
    else if (code === 'ECONNRESET' || code === 'EPIPE') fails.reset++
    else fails.other++
    if (samples.length < 5) samples.push(`${label}: ${code}`)
  } else if ((r.status ?? 0) >= 500) {
    fails.status5++
    if (samples.length < 5) samples.push(`${label}: HTTP ${r.status} ${r.body?.slice(0, 80)}`)
  } else {
    fails.status4++
    if (samples.length < 5) samples.push(`${label}: HTTP ${r.status} ${r.body?.slice(0, 80)}`)
  }
}

async function main(): Promise<void> {
  const deadline = Date.now() + DURATION_MS
  const work: Promise<void>[] = []

  for (const w of [0, 1]) {
    work.push(
      (async () => {
        while (Date.now() < deadline) {
          const r = await req('POST', PORTS[w % 2], '/api/memories', {
            summary: `soak ${w} ${Date.now()}`,
            content: 'soak body',
            tags: ['soak'],
            // the tag is fixed fixture vocabulary; opt into auto-creation explicitly
            create_missing_tags: true,
          })
          if (r.status === 200) ops.create++
          else recordFail('create', r)
        }
      })(),
    )
  }
  for (const w of [0, 1]) {
    work.push(
      (async () => {
        while (Date.now() < deadline) {
          const r = await req('GET', PORTS[w % 2], '/api/memories?query=soak&limit=5')
          if (r.status === 200) ops.search++
          else recordFail('search', r)
        }
      })(),
    )
  }
  work.push(
    (async () => {
      let i = 0
      while (Date.now() < deadline) {
        const name = `soaktag${i % 5}`
        await req('POST', PORTS[i % 2], '/api/tags', { name, description: 'soak' })
        await req('PUT', PORTS[i % 2], `/api/tags/${name}`, { description: `v${i}` })
        await req('DELETE', PORTS[i % 2], `/api/tags/${name}?mode=detach`)
        ops.tag++
        i++
      }
    })(),
  )
  work.push(
    (async () => {
      while (Date.now() < deadline) {
        const r = await req('GET', PORTS[0], '/api/memories?limit=20')
        if (r.status === 200) ops.list++
        else recordFail('list', r)
      }
    })(),
  )

  await Promise.all(work)
  console.log('ops:', JSON.stringify(ops))
  console.log('fails:', JSON.stringify(fails))
  console.log('error samples:')
  for (const s of samples) console.log('  -', s)
}

void main()
