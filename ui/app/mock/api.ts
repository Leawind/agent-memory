// dev 专用 mock API：Vite 中间件形态的内存假数据源，`npm run dev` 默认启用，
// 让前端开发不依赖 Rust 后端。`--mode live` 时不加载本插件，走真实服务器代理。
// 契约与 src/api.rs / src/tools/defs.rs 一一对应（字段名照抄，勿凭空发明）；
// 行为只模拟 UI 用得到的路径：未知 /api 路由一律 404，方便发现契约漂移。
import { randomBytes } from 'node:crypto'
import type { ServerResponse } from 'node:http'
import type { Connect, Plugin } from 'vite'

// 模拟网络延迟区间（ms）：让 loading 态可见，又不拖慢开发节奏
const LATENCY_MS = [40, 140] as const

const CAPS = ['read', 'create', 'update', 'delete', 'tag_manage', 'admin'] as const
type Cap = (typeof CAPS)[number]

const allCaps = (): Record<Cap, boolean> => Object.fromEntries(CAPS.map((c) => [c, true])) as Record<Cap, boolean>

interface MockMemory {
  id: number
  summary: string
  content: string
  tags: string[]
  created_at: number
  updated_at: number
}

interface MockTag {
  description: string
  created_at: number
}

interface MockIdentity {
  name: string
  token: string
  permissions: Record<Cap, boolean>
  created_at: number
}

interface MockSettings {
  auth_required: boolean
  instructions: string
  conventions: string
  embedding_enabled: boolean
  embedding_base_url: string
  embedding_model: string
  embedding_api_key: string
}

const DAY = 86_400
const now = (): number => Math.floor(Date.now() / 1000)

// ---- 种子数据：中英混合 + Markdown 正文，覆盖列表/搜索/详情/管理各面板 ----
function seedState(): {
  memories: MockMemory[]
  tags: Map<string, MockTag>
  identities: MockIdentity[]
  settings: MockSettings
  nextId: number
} {
  const t = now()
  const memories: MockMemory[] = [
    {
      id: 1,
      summary: '项目结构约定',
      content:
        '# 模块划分\n\n- `src/tools/defs.rs` 是对 agent 的**契约**，唯一权威来源\n- SQL 一律外置到 `sql/*.sql`，Rust 代码不出现 SQL 文本\n- 迁移脚本在 `migrations/`，build.rs 编译期登记\n\n> 改契约前先读 AGENTS.md 的架构不变量。',
      tags: ['rust', '架构'],
      created_at: t - 30 * DAY,
      updated_at: t - 2 * DAY,
    },
    {
      id: 2,
      summary: 'MCP protocol versions supported',
      content:
        '## Protocol compatibility\n\nWe support `2024-11-05`, `2025-03-26` and `2025-06-18`.\n\n- id boundary format is strictly `"m{n}"`\n- notifications never get a response\n- tool results travel the text channel as compact JSON',
      tags: ['mcp', 'rust'],
      created_at: t - 25 * DAY,
      updated_at: t - 3 * DAY,
    },
    {
      id: 3,
      summary: '组件库四面板设计',
      content:
        '# 四个自包含面板\n\n1. `MemoriesPanel`：列表/搜索双模式\n2. `TagsPanel`：标签表 + 正则过滤\n3. `OpsPanel`：只读概况（统计 + 体检）\n4. `AdminPanel`：身份、鉴权开关、提示词、备份\n\n样式全部引用 `--el-*` 变量，跟随宿主主题。',
      tags: ['前端', '设计'],
      created_at: t - 20 * DAY,
      updated_at: t - 1 * DAY,
    },
    {
      id: 4,
      summary: 'SQLite WAL 事务纪律',
      content:
        '读写分离：读请求 `BEGIN DEFERRED`（一致性快照、不抢写锁），写请求 `BEGIN IMMEDIATE`（一开始就取写锁）。\n\nDEFERRED 写事务"先读后升级"在并发下会立即 SQLITE_BUSY，busy_timeout 不重试死锁场景。',
      tags: ['rust', '架构'],
      created_at: t - 18 * DAY,
      updated_at: t - 4 * DAY,
    },
    {
      id: 5,
      summary: '语义搜索回退策略',
      content:
        '# 回退是硬性承诺\n\nembedding 服务不可用只允许降级不允许失败：\n\n- 搜索回退纯关键词，携带 `semantic_fallback` 标记\n- 写入静默留待补跑\n- 混合排序用 RRF 融合，禁止不同量纲直接加权',
      tags: ['mcp', '设计', 'embeddings'],
      created_at: t - 15 * DAY,
      updated_at: t - 6 * DAY,
    },
    {
      id: 6,
      summary: 'e2e 测试基建',
      content:
        'tests/e2e/ 真实 spawn 二进制（serve --port 随机 + --db 临时目录），HTTP 客户端用 std::net 手写。服务器启动用全局锁串行化避免端口竞争。',
      tags: ['测试'],
      created_at: t - 12 * DAY,
      updated_at: t - 8 * DAY,
    },
    {
      id: 7,
      summary: '组合式函数清单',
      content:
        '# composables\n\n| 名称 | 职责 |\n| --- | --- |\n| `useMemories` | 列表/搜索状态机 |\n| `useTags` | 标签表 |\n| `useOps` | 统计与体检 |\n| `useAdmin` | 备份/体检/补跑 |\n\n面板层只渲染与 toast，数据操作全在 composables。',
      tags: ['前端'],
      created_at: t - 10 * DAY,
      updated_at: t - 5 * DAY,
    },
    {
      id: 8,
      summary: '发布前检查清单',
      content:
        '1. `cargo test` 全绿\n2. `cargo clippy --all-targets` 零告警\n3. `npm run typecheck` 通过\n4. `node scripts/sdk-compat-check.ts` 官方 SDK 回归\n5. 更新 CHANGELOG，打 tag',
      tags: ['流程', '测试'],
      created_at: t - 8 * DAY,
      updated_at: t - 10 * 3600,
    },
    {
      id: 9,
      summary: 'i18n 双语字典维护',
      content:
        '# 新增语言三步走\n\n1. lib 里加字典文件并在 messages 注册\n2. languages 列表加一项（nativeName 用语言本名）\n3. 补齐 zh/en 两份字典的新 key\n\nauto 模式跟随浏览器语言，`setMemoryUILocale` 运行时切换。',
      tags: ['前端', '流程'],
      created_at: t - 6 * DAY,
      updated_at: t - 7 * 3600,
    },
    {
      id: 10,
      summary: 'RRF fusion for hybrid search',
      content:
        'Keyword scores and cosine similarity live on different scales, so **never** compare them directly — fuse with Reciprocal Rank Fusion (k = 60).\n\nVectors are derived data: cascade-deleted with memories, never exported, fingerprint-mismatch counts as missing.',
      tags: ['embeddings', 'mcp'],
      created_at: t - 3 * DAY,
      updated_at: t - 2 * 3600,
    },
  ]
  const tags = new Map<string, MockTag>(
    Object.entries({
      rust: { description: 'Rust 后端与工具链', created_at: t - 30 * DAY },
      架构: { description: '模块划分与不变量', created_at: t - 30 * DAY },
      mcp: { description: 'Model Context Protocol', created_at: t - 25 * DAY },
      前端: { description: 'Vue3 组件库与站点', created_at: t - 20 * DAY },
      设计: { description: '交互与视觉决定', created_at: t - 20 * DAY },
      embeddings: { description: '语义搜索与向量', created_at: t - 15 * DAY },
      测试: { description: '单元与端到端测试', created_at: t - 12 * DAY },
      流程: { description: '工作流与检查清单', created_at: t - 8 * DAY },
    }).map(([name, v]) => [name, v]),
  )
  const identities: MockIdentity[] = [
    { name: 'leawind', token: 'mock-token-leawind-admin', permissions: allCaps(), created_at: t - 30 * DAY },
    {
      name: 'guest',
      token: 'mock-token-guest-viewer',
      permissions: { read: true, create: false, update: false, delete: false, tag_manage: false, admin: false },
      created_at: t - 7 * DAY,
    },
  ]
  const settings: MockSettings = {
    auth_required: false,
    instructions: '',
    conventions: '',
    embedding_enabled: false,
    embedding_base_url: '',
    embedding_model: '',
    embedding_api_key: '',
  }
  return { memories, tags, identities, settings, nextId: 11 }
}

// ---- 视图组装（字段名与真实服务端一致）----
const formatId = (id: number): string => `m${id}`

function fullView(m: MockMemory) {
  return {
    id: formatId(m.id),
    tags: [...m.tags],
    summary: m.summary,
    content: m.content,
    created_at: m.created_at,
    updated_at: m.updated_at,
  }
}

function summaryView(m: MockMemory) {
  const { content: _content, ...rest } = fullView(m)
  return rest
}

function tagView(name: string, tag: MockTag, memories: MockMemory[]) {
  const carrying = memories.filter((m) => m.tags.includes(name))
  return {
    name,
    description: tag.description,
    memory_count: carrying.length,
    last_used_at: carrying.length ? Math.max(...carrying.map((m) => m.updated_at)) : null,
    created_at: tag.created_at,
  }
}

const DEFAULT_INSTRUCTIONS = '（mock）内置默认提示词占位：管理语义记忆的服务器使用说明。'

function settingsView(s: MockSettings) {
  return { ...s, default_instructions: DEFAULT_INSTRUCTIONS }
}

// 片段窗口：HTML 转义后用 <mark> 包住命中（UI 会再过一遍 DOMPurify）
function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

function buildSnippet(content: string, query: string): string {
  const lower = content.toLowerCase()
  const q = query.toLowerCase()
  const first = lower.indexOf(q)
  const start = Math.max(0, first - 50)
  const end = Math.min(content.length, first < 0 ? 140 : first + q.length + 90)
  const raw = (start > 0 ? '…' : '') + content.slice(start, end) + (end < content.length ? '…' : '')
  let out = ''
  let rest = raw
  for (;;) {
    const hit = rest.toLowerCase().indexOf(q)
    if (hit < 0 || !q) {
      out += escapeHtml(rest)
      break
    }
    out += escapeHtml(rest.slice(0, hit))
    out += `<mark>${escapeHtml(rest.slice(hit, hit + q.length))}</mark>`
    rest = rest.slice(hit + q.length)
  }
  return out
}

function scoreOf(m: MockMemory, query: string): number {
  const q = query.toLowerCase()
  const hits = (s: string): number => s.toLowerCase().split(q).length - 1
  return Math.min(99, hits(m.summary) * 2 + hits(m.content))
}

function normalizeTag(name: unknown): string {
  return typeof name === 'string' ? name.trim() : ''
}

// ---- HTTP 小工具 ----
type JsonBody = Record<string, unknown>

function sendJson(res: ServerResponse, status: number, body: unknown): void {
  const text = body === null || body === undefined ? '' : JSON.stringify(body)
  res.statusCode = status
  res.setHeader('Content-Type', 'application/json; charset=utf-8')
  res.end(text)
}

function readBody(req: Connect.IncomingMessage): Promise<JsonBody> {
  return new Promise((resolve) => {
    const chunks: Buffer[] = []
    req.on('data', (c: Buffer) => chunks.push(c))
    req.on('end', () => {
      try {
        const parsed: unknown = JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}')
        resolve(parsed && typeof parsed === 'object' ? (parsed as JsonBody) : {})
      } catch {
        resolve({})
      }
    })
  })
}

function bearerToken(req: Connect.IncomingMessage): string | null {
  const header = req.headers.authorization
  return header?.startsWith('Bearer ') ? header.slice(7) : null
}

function err(status: number, message: string): { status: number; body: { error: string } } {
  return { status, body: { error: message } }
}

export function mockApi(): Plugin {
  const state = seedState()

  // 身份解析：与真实服务端同语义——有效 token 决定身份；鉴权开启时无有效 token
  // 一律 401（fail-closed）；未开启时视为开放模式全能力。
  function resolve(
    req: Connect.IncomingMessage,
  ):
    | { ok: true; who: { name: string; mode: 'open' | 'token'; permissions: Record<string, boolean> } }
    | { ok: false; status: number; error: string } {
    const token = bearerToken(req)
    const identity = token ? state.identities.find((i) => i.token === token) : undefined
    if (identity) {
      return { ok: true, who: { name: identity.name, mode: 'token', permissions: identity.permissions } }
    }
    if (state.settings.auth_required) {
      return { ok: false, status: 401, error: 'missing or invalid token' }
    }
    return { ok: true, who: { name: 'local', mode: 'open', permissions: allCaps() } }
  }

  function requireCap(req: Connect.IncomingMessage, cap: Cap): { status: number; body: { error: string } } | null {
    const r = resolve(req)
    if (!r.ok) return err(r.status, r.error)
    if (r.who.permissions[cap] !== true) return err(403, `requires '${cap}' capability`)
    return null
  }

  async function handle(
    req: Connect.IncomingMessage,
    url: URL,
    method: string,
  ): Promise<{ status: number; body: unknown; attachment?: string }> {
    const segs = url.pathname.split('/').filter(Boolean)
    const qs = url.searchParams

    // 探活与 whoami 免鉴权（与真实服务端一致）
    if (url.pathname === '/health') {
      return { status: 200, body: { status: 'ok', version: '0.1.0-mock' } }
    }
    if (url.pathname === '/api/whoami') {
      const r = resolve(req)
      if (!r.ok) return err(r.status, r.error)
      return { status: 200, body: r.who }
    }

    if (segs[0] !== 'api') return err(404, `no such route: ${method} ${url.pathname}`)
    const rest = segs.slice(1)

    // ---- 记忆 ----
    if (rest[0] === 'memories' && rest.length === 1) {
      if (method === 'GET') {
        const guard = requireCap(req, 'read')
        if (guard) return guard
        const query = (qs.get('query') ?? '').trim()
        const offset = Math.max(0, Number(qs.get('offset') ?? 0) || 0)
        const limit = Math.min(100, Math.max(1, Number(qs.get('limit') ?? 20) || 20))
        if (query) {
          const tagFilter = (qs.get('tags') ?? '').split(',').map(normalizeTag).filter(Boolean)
          const explicitHybrid = qs.get('mode') === 'hybrid'
          const configured = state.settings.embedding_enabled && !!state.settings.embedding_model
          let pool = state.memories.filter(
            (m) =>
              (m.summary + '\n' + m.content).toLowerCase().includes(query.toLowerCase()) &&
              tagFilter.every((t) => m.tags.includes(t)),
          )
          const scored = pool
            .map((m) => ({ m, score: scoreOf(m, query) }))
            .sort((a, b) => b.score - a.score || b.m.updated_at - a.m.updated_at)
          pool = scored.map((s) => s.m)
          return {
            status: 200,
            body: {
              total_matches: pool.length,
              offset,
              returned: Math.min(limit, pool.length - offset),
              // mock 未配 embedding：显式 hybrid 按真实语义回退关键词并带标记
              mode: 'keyword',
              semantic_fallback: explicitHybrid && !configured ? true : undefined,
              results: pool.slice(offset, offset + limit).map((m) => ({
                id: formatId(m.id),
                tags: [...m.tags],
                summary: m.summary,
                score: scoreOf(m, query),
                snippet: buildSnippet(m.content, query),
                updated_at: m.updated_at,
              })),
            },
          }
        }
        const tag = normalizeTag(qs.get('tag') ?? '')
        const sort = qs.get('sort') ?? 'updated_at'
        const order = qs.get('order') === 'asc' ? 1 : -1
        const pool = state.memories
          .filter((m) => !tag || m.tags.includes(tag))
          .sort((a, b) => {
            if (sort === 'created_at') return (a.created_at - b.created_at) * order
            if (sort === 'id') return (a.id - b.id) * order
            return (a.updated_at - b.updated_at) * order
          })
        return {
          status: 200,
          body: { total: pool.length, offset, limit, memories: pool.slice(offset, offset + limit).map(summaryView) },
        }
      }
      if (method === 'POST') {
        const guard = requireCap(req, 'create')
        if (guard) return guard
        const body = await readBody(req)
        const summary = typeof body.summary === 'string' ? body.summary.trim() : ''
        const content = typeof body.content === 'string' ? body.content : ''
        if (!summary) return err(400, 'summary cannot be empty')
        if (!content.trim()) return err(400, 'content cannot be empty')
        const tags = [...new Set(Array.isArray(body.tags) ? body.tags.map(normalizeTag).filter(Boolean) : [])]
        const t = now()
        const memory: MockMemory = { id: state.nextId++, summary, content, tags, created_at: t, updated_at: t }
        state.memories.push(memory)
        for (const tag of tags) if (!state.tags.has(tag)) state.tags.set(tag, { description: '', created_at: t })
        return { status: 200, body: fullView(memory) }
      }
    }

    if (rest[0] === 'memories' && rest.length === 2) {
      const id = /^m(\d+)$/.exec(rest[1] ?? '')
      const memory = id ? state.memories.find((m) => m.id === Number(id[1])) : undefined
      if (!memory) return err(404, `memory '${rest[1]}' not found`)
      if (method === 'GET') {
        const guard = requireCap(req, 'read')
        if (guard) return guard
        return { status: 200, body: fullView(memory) }
      }
      if (method === 'PUT') {
        const guard = requireCap(req, 'update')
        if (guard) return guard
        const body = await readBody(req)
        if (typeof body.summary === 'string') {
          if (!body.summary.trim()) return err(400, 'summary cannot be empty')
          memory.summary = body.summary.trim()
        }
        if (typeof body.content === 'string') {
          if (!body.content.trim()) return err(400, 'content cannot be empty')
          memory.content = body.content
        }
        const add = Array.isArray(body.add_tags) ? body.add_tags.map(normalizeTag).filter(Boolean) : []
        const remove = Array.isArray(body.remove_tags) ? body.remove_tags.map(normalizeTag).filter(Boolean) : []
        memory.tags = [...new Set(memory.tags.filter((t) => !remove.includes(t)).concat(add))]
        const t = now()
        for (const tag of add) if (!state.tags.has(tag)) state.tags.set(tag, { description: '', created_at: t })
        memory.updated_at = t
        return { status: 200, body: fullView(memory) }
      }
      if (method === 'DELETE') {
        const guard = requireCap(req, 'delete')
        if (guard) return guard
        state.memories = state.memories.filter((m) => m !== memory)
        return { status: 200, body: null }
      }
    }

    // ---- 标签 ----
    if (rest[0] === 'tags' && rest.length === 1) {
      if (method === 'GET') {
        const guard = requireCap(req, 'read')
        if (guard) return guard
        const filter = qs.get('filter')
        let names = [...state.tags.keys()]
        if (filter) {
          let re: RegExp
          try {
            re = new RegExp(filter)
          } catch {
            return err(400, `invalid filter regex: ${filter}`)
          }
          names = names.filter((n) => re.test(n))
        }
        return {
          status: 200,
          body: {
            total_tags: names.length,
            total_memories: state.memories.length,
            tags: names.map((n) => tagView(n, state.tags.get(n)!, state.memories)),
          },
        }
      }
      if (method === 'POST') {
        const guard = requireCap(req, 'tag_manage')
        if (guard) return guard
        const body = await readBody(req)
        const name = normalizeTag(body.name)
        if (!name) return err(400, 'tag name cannot be empty')
        if (state.tags.has(name)) return err(400, `tag '${name}' already exists`)
        state.tags.set(name, {
          description: typeof body.description === 'string' ? body.description : '',
          created_at: now(),
        })
        return { status: 200, body: null }
      }
    }

    if (rest[0] === 'tags' && rest.length === 2) {
      const name = rest[1] ?? ''
      if (!state.tags.has(name)) return err(404, `tag '${name}' not found`)
      if (method === 'PUT') {
        const guard = requireCap(req, 'tag_manage')
        if (guard) return guard
        const body = await readBody(req)
        if (typeof body.description === 'string') state.tags.get(name)!.description = body.description
        const newName = normalizeTag(body.new_name)
        if (newName && newName !== name) {
          if (state.tags.has(newName)) return err(400, `tag '${newName}' already exists`)
          state.tags.set(newName, state.tags.get(name)!)
          state.tags.delete(name)
          for (const m of state.memories) m.tags = m.tags.map((t) => (t === name ? newName : t))
        }
        return { status: 200, body: null }
      }
      if (method === 'DELETE') {
        const guard = requireCap(req, 'tag_manage')
        if (guard) return guard
        const mode = qs.get('mode') ?? 'detach'
        if (mode !== 'detach' && mode !== 'purge') return err(400, `mode must be 'detach' or 'purge', got '${mode}'`)
        if (mode === 'detach') {
          for (const m of state.memories) m.tags = m.tags.filter((t) => t !== name)
        } else {
          state.memories = state.memories.filter((m) => !m.tags.includes(name))
        }
        state.tags.delete(name)
        return { status: 200, body: null }
      }
    }

    // ---- 运维 ----
    if (rest[0] === 'stats' && rest.length === 1 && method === 'GET') {
      const guard = requireCap(req, 'read')
      if (guard) return guard
      const newest = state.memories.reduce<MockMemory | null>(
        (acc, m) => (!acc || m.updated_at > acc.updated_at ? m : acc),
        null,
      )
      return {
        status: 200,
        body: {
          path: '(mock: in-memory)',
          memories: state.memories.length,
          tags: state.tags.size,
          next_id: formatId(state.nextId),
          file_size: 4096 + state.memories.reduce((n, m) => n + m.content.length + m.summary.length, 0),
          newest_update: newest ? { id: formatId(newest.id), updated_at: newest.updated_at } : null,
          schema_version: 1,
          embedding: state.settings.embedding_enabled
            ? {
                enabled: true,
                model: state.settings.embedding_model || 'mock-embedding',
                embedded: state.memories.length,
                pending: 0,
              }
            : { enabled: false },
        },
      }
    }

    if (rest[0] === 'doctor' && rest.length === 1 && method === 'GET') {
      const guard = requireCap(req, 'admin')
      if (guard) return guard
      return { status: 200, body: { ok: true, issues: [] } }
    }

    if (rest[0] === 'export' && rest.length === 1 && method === 'GET') {
      const guard = requireCap(req, 'admin')
      if (guard) return guard
      return {
        status: 200,
        attachment: 'agent-memory-export.json',
        body: {
          exported_at: now(),
          total_memories: state.memories.length,
          total_tags: state.tags.size,
          tags: [...state.tags.entries()].map(([n, t]) => tagView(n, t, state.memories)),
          memories: state.memories.map(fullView),
        },
      }
    }

    if (rest[0] === 'import' && rest.length === 1 && method === 'POST') {
      const guard = requireCap(req, 'admin')
      if (guard) return guard
      if (state.memories.length || state.tags.size) {
        return err(400, 'target database is not empty; import refuses to merge -- point --db at a fresh database')
      }
      const dump = await readBody(req)
      if (!Array.isArray(dump.tags) || !Array.isArray(dump.memories)) {
        return err(400, "invalid export: missing 'tags'/'memories' array")
      }
      for (const t of dump.tags) {
        const v = t as JsonBody
        const name = normalizeTag(v?.name)
        if (!name) return err(400, 'invalid export: tag without name')
        state.tags.set(name, {
          description: typeof v.description === 'string' ? v.description : '',
          created_at: now(),
        })
      }
      for (const m of dump.memories) {
        const v = m as JsonBody
        const summary = typeof v.summary === 'string' ? v.summary : ''
        const content = typeof v.content === 'string' ? v.content : ''
        if (!summary || !content.trim()) return err(400, 'invalid export: memory without summary/content')
        const tags = Array.isArray(v.tags) ? v.tags.map(normalizeTag).filter(Boolean) : []
        state.memories.push({
          id: state.nextId++,
          summary,
          content,
          tags,
          created_at: typeof v.created_at === 'number' ? v.created_at : now(),
          updated_at: typeof v.updated_at === 'number' ? v.updated_at : now(),
        })
      }
      return { status: 200, body: { imported_memories: dump.memories.length, imported_tags: dump.tags.length } }
    }

    if (rest[0] === 'embeddings' && rest[1] === 'backfill' && method === 'POST') {
      const guard = requireCap(req, 'admin')
      if (guard) return guard
      if (!state.settings.embedding_enabled) return { status: 200, body: { configured: false } }
      return { status: 200, body: { configured: true, processed: 0, remaining: 0 } }
    }

    if (rest[0] === 'embeddings' && rest[1] === 'test' && method === 'POST') {
      const guard = requireCap(req, 'admin')
      if (guard) return guard
      const s = state.settings
      if (!s.embedding_enabled || !s.embedding_base_url || !s.embedding_model || !s.embedding_api_key) {
        return { status: 200, body: { ok: false, error: 'embedding 配置不完整（服务地址 / 模型 / api key）' } }
      }
      return { status: 200, body: { ok: true, dim: 1024, elapsed_ms: 20 + Math.floor(Math.random() * 60) } }
    }

    // ---- 身份与设置（admin 专属，与真实服务端一样不走 MCP 工具面）----
    if (rest[0] === 'identities' && rest.length === 1) {
      if (method === 'GET') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        return {
          status: 200,
          body: {
            identities: state.identities.map((i) => ({
              name: i.name,
              token_hint: `${i.token.slice(0, 4)}...${i.token.slice(-4)}`,
              permissions: i.permissions,
              created_at: i.created_at,
            })),
          },
        }
      }
      if (method === 'POST') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        const body = await readBody(req)
        const name = normalizeTag(body.name)
        if (!name) return err(400, 'identity name cannot be empty')
        if (state.identities.some((i) => i.name === name)) return err(400, `identity '${name}' already exists`)
        const permissions = body.permissions
        if (!permissions || typeof permissions !== 'object') return err(400, 'permissions must be an object')
        const input = permissions as JsonBody
        for (const key of Object.keys(input)) {
          if (!CAPS.includes(key as Cap)) return err(400, `unknown capability: '${key}'`)
          if (typeof input[key] !== 'boolean') return err(400, `capability '${key}' must be a boolean`)
        }
        // 全键布尔（缺省视为 false），与服务端「permissions JSON 必须全键」对齐
        const perms = Object.fromEntries(CAPS.map((c) => [c, input[c] === true])) as Record<Cap, boolean>
        const token = `mock-${randomBytes(8).toString('hex')}`
        const identity: MockIdentity = { name, token, permissions: perms, created_at: now() }
        state.identities.push(identity)
        return {
          status: 200,
          body: {
            name,
            token_hint: `${token.slice(0, 4)}...${token.slice(-4)}`,
            permissions: identity.permissions,
            created_at: identity.created_at,
            token,
          },
        }
      }
    }

    if (rest[0] === 'identities' && rest.length >= 2) {
      const identity = state.identities.find((i) => i.name === rest[1])
      if (rest.length === 2 && method === 'PUT') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        if (!identity) return err(404, `identity '${rest[1]}' not found`)
        const body = await readBody(req)
        const permissions = body.permissions
        if (!permissions || typeof permissions !== 'object') return err(400, 'permissions must be an object')
        const input = permissions as JsonBody
        for (const key of Object.keys(input)) {
          if (!CAPS.includes(key as Cap)) return err(400, `unknown capability: '${key}'`)
          if (typeof input[key] !== 'boolean') return err(400, `capability '${key}' must be a boolean`)
        }
        identity.permissions = Object.fromEntries(CAPS.map((c) => [c, input[c] === true])) as Record<Cap, boolean>
        return { status: 200, body: null }
      }
      if (rest.length === 2 && method === 'DELETE') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        if (!identity) return err(404, `identity '${rest[1]}' not found`)
        state.identities = state.identities.filter((i) => i !== identity)
        return { status: 200, body: null }
      }
      if (rest.length === 3 && rest[2] === 'token-reset' && method === 'POST') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        if (!identity) return err(404, `identity '${rest[1]}' not found`)
        const token = `mock-${randomBytes(8).toString('hex')}`
        identity.token = token
        return { status: 200, body: { token } }
      }
    }

    if (rest[0] === 'settings') {
      if (method === 'GET') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        return { status: 200, body: settingsView(state.settings) }
      }
      if (method === 'PUT') {
        const guard = requireCap(req, 'admin')
        if (guard) return guard
        const body = await readBody(req)
        const s = state.settings
        if (typeof body.auth_required === 'boolean' && body.auth_required !== s.auth_required) {
          if (body.auth_required && !state.identities.some((i) => i.permissions.admin)) {
            return err(400, 'cannot enable auth: no admin-capable identity exists; create one first')
          }
          s.auth_required = body.auth_required
        }
        if (typeof body.instructions === 'string') s.instructions = body.instructions
        if (typeof body.conventions === 'string') s.conventions = body.conventions
        if (typeof body.embedding_enabled === 'boolean') s.embedding_enabled = body.embedding_enabled
        if (typeof body.embedding_base_url === 'string') s.embedding_base_url = body.embedding_base_url
        if (typeof body.embedding_model === 'string') s.embedding_model = body.embedding_model
        if (typeof body.embedding_api_key === 'string') s.embedding_api_key = body.embedding_api_key
        return { status: 200, body: settingsView(s) }
      }
    }

    return err(404, `no such route: ${method} ${url.pathname}`)
  }

  return {
    name: 'agent-memory-mock-api',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost')
        const mocked = url.pathname === '/health' || url.pathname === '/api' || url.pathname.startsWith('/api/')
        if (!mocked) {
          next()
          return
        }
        void (async () => {
          const method = (req.method ?? 'GET').toUpperCase()
          const latency = LATENCY_MS[0] + Math.floor(Math.random() * (LATENCY_MS[1] - LATENCY_MS[0]))
          await new Promise((r) => setTimeout(r, latency))
          try {
            const out = await handle(req, url, method)
            if (out.attachment) res.setHeader('Content-Disposition', `attachment; filename="${out.attachment}"`)
            sendJson(res, out.status, out.body)
          } catch (e) {
            sendJson(res, 500, { error: e instanceof Error ? e.message : String(e) })
          }
        })()
      })
      // ready 提示：Vite 自身横幅打印在 listening 之后，紧跟其后说明当前数据源
      server.httpServer?.once('listening', () => {
        server.config.logger.info(
          '\n  \x1b[32m➜\x1b[0m  \x1b[1mmock API\x1b[0m 已启用（内存假数据，重启复位）；' +
            '连真实后端请用 \x1b[36mnpm run dev:live\x1b[0m\n' +
            '     预置身份 token：leawind/admin = \x1b[33mmock-token-leawind-admin\x1b[0m，' +
            'guest/只读 = \x1b[33mmock-token-guest-viewer\x1b[0m\n',
        )
      })
    },
  }
}
