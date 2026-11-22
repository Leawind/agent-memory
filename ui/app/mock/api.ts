// dev 专用 mock API（Vite 中间件，`npm run dev` 默认启用）：用内存假数据喂饱四个面板，
// 让前端改完代码刷新就能看到界面，不依赖 Rust 后端；`--mode live` 不加载本插件。
// 只求界面能渲染、增删改看得见变化，不模拟鉴权/搜索评分等服务端语义——那类行为
// 用 `npm run dev:live` 连真实后端验证。数据只存内存，dev server 重启即复位。
import type { ServerResponse } from 'node:http'
import type { Connect, Plugin } from 'vite'

const now = (): number => Math.floor(Date.now() / 1000)
const DAY = 86_400

/** `scope::value` 的作用域半段；裸名返回 null */
const scopeOf = (name: string): string | null => {
  const i = name.indexOf('::')
  return i > 0 ? name.slice(0, i) : null
}

interface MockMemory {
  id: number
  summary: string
  content: string
  /** 全名列表：裸名（rust）或带作用域（system::pc1sw0），一条记忆每个作用域至多一个 */
  tags: string[]
  created_at: number
  updated_at: number
}

// ---- 种子数据：中英混合 Markdown，够列表/搜索/详情/管理各面板渲染即可 ----
const t = now()
const memories: MockMemory[] = [
  {
    id: 1,
    summary: '项目结构约定',
    content:
      '# 模块划分\n\n- `src/tools/defs.rs` 是对 agent 的**契约**，唯一权威来源\n- SQL 一律外置到 `sql/*.sql`，Rust 代码不出现 SQL 文本\n- 迁移脚本在 `migrations/`，build.rs 编译期登记\n\n> 改契约前先读 AGENTS.md 的架构不变量。',
    tags: ['rust', '架构', 'project::agent-memory'],
    created_at: t - 30 * DAY,
    updated_at: t - 2 * DAY,
  },
  {
    id: 2,
    summary: 'MCP protocol versions',
    content:
      '## Protocol compatibility\n\nWe support `2024-11-05`, `2025-03-26` and `2025-06-18`.\n\n- id boundary format is strictly `"m{n}"`\n- notifications never get a response',
    tags: ['mcp', 'rust', 'project::agent-memory', 'score::3'],
    created_at: t - 25 * DAY,
    updated_at: t - 3 * DAY,
  },
  {
    id: 3,
    summary: '组件库面板设计',
    content:
      '# 工作台组件\n\n1. `TagsSidebar`：左侧标签侧栏，按作用域分组，点击弹 `TagDialog` / `ScopeDialog`\n2. `MemoriesPanel`：卡片列表，整卡点击弹记忆弹窗\n3. `AdminPanel`：身份、鉴权开关、提示词、备份（挂在管理弹窗里）\n\n概况是弹窗 `OpsDialog`：点击顶栏标题打开，每次打开重拉数据。样式全部引用 `--el-*` 变量，跟随宿主主题。',
    tags: ['前端', '设计', 'project::ui', 'phase::design', 'phase::review'],
    created_at: t - 20 * DAY,
    updated_at: t - 1 * DAY,
  },
  {
    id: 4,
    summary: 'SQLite WAL 事务纪律',
    content: '读写分离：读请求 `BEGIN DEFERRED`（一致性快照、不抢写锁），写请求 `BEGIN IMMEDIATE`（一开始就取写锁）。',
    tags: ['rust', '架构', 'project::agent-memory'],
    created_at: t - 18 * DAY,
    updated_at: t - 4 * DAY,
  },
  {
    id: 5,
    summary: '发布前检查清单',
    content: '1. `cargo test` 全绿\n2. `cargo clippy --all-targets` 零告警\n3. `npm run typecheck` 通过',
    tags: ['流程', '测试', 'score::5'],
    created_at: t - 8 * DAY,
    updated_at: t - 10 * 3600,
  },
]
interface MockTag {
  description: string
  created_at: number
}
interface MockScope {
  description: string
  created_at: number
  /** 必填：每次写入记忆都必须声明（真实服务端存生效时间，这里只留布尔） */
  required?: boolean
}
// 词汇表按真实服务端形状存全名：作用域是独立实体，`scope::value` 是一个词条
const tags = new Map<string, MockTag>(
  [
    ['rust', 'Rust 后端与工具链'],
    ['架构', '模块划分与不变量'],
    ['mcp', 'Model Context Protocol'],
    ['前端', 'Vue3 组件库与站点'],
    ['设计', '交互与视觉决定'],
    ['测试', '单元与端到端'],
    ['流程', '发布与协作约定'],
    ['project::agent-memory', '本仓库'],
    ['project::ui', '管理界面'],
    ['phase::design', '设计阶段'],
    ['phase::review', '评审阶段'],
    ['score::3', '常规优先级'],
    ['score::5', '紧急优先级'],
  ].map(([name, description]) => [name, { description, created_at: t - 30 * DAY }]),
)
const scopes = new Map<string, MockScope>(
  [
    ['project', '项目归属'],
    ['phase', '工作阶段'],
    // 必填作用域示例：编辑器的“新建记忆”会要求声明它（取值或“不取值”）
    ['score', '优先级分值'],
  ].map(([name, description]) => [name, { description, created_at: t - 30 * DAY }]),
)
scopes.get('score')!.required = true
const predicates: Array<{ name: string; predicate: string; description: string }> = []
const settings = {
  auth_required: false,
  // 匿名身份能力集：真实服务端为全键布尔对象或 null（未设置 = 匿名被拒绝）
  anonymous_permissions: null as Record<string, boolean> | null,
  instructions: '',
  embedding_enabled: false,
  embedding_base_url: '',
  embedding_model: '',
  embedding_api_key: '',
}
let nextId = memories.length + 1

// ---- 视图组装（字段名与真实服务端一致，见 ui/lib/src/types.ts）----
const formatId = (id: number): string => `m${id}`
const parseId = (raw: string | undefined): MockMemory | undefined => {
  const m = /^m(\d+)$/.exec(raw ?? '')
  return m ? memories.find((x) => x.id === Number(m[1])) : undefined
}
// 紧凑本地墙钟，与真实服务端的 created/updated 渲染一致（YYYY-MM-DD HH:MM）
const compactTime = (ts: number): string => {
  const d = new Date(ts * 1000)
  const p = (n: number): string => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}
// 稀疏作用域取值视图（与真实服务端的 attributes_view 一致）：键是作用域，值来自
// `scope::value` 的取值半段；能解析为规范数的值渲染成 JSON 数字。无取值则整个省略
// （mock 不持久化“声明不取值”，读回即无声明——真实服务端按必填生效时间区分存量）
const attributesOf = (m: MockMemory): Record<string, unknown> => {
  const out: Record<string, unknown> = {}
  for (const tag of m.tags) {
    const scope = scopeOf(tag)
    if (scope === null) continue
    const raw = tag.slice(scope.length + 2)
    out[scope] = /^-?(0|[1-9]\d*)(\.\d+)?([eE]-?\d+)?$/.test(raw) ? Number(raw) : raw
  }
  return out
}
const withAttributes = (view: Record<string, unknown>, m: MockMemory): Record<string, unknown> => {
  const attributes = attributesOf(m)
  if (Object.keys(attributes).length > 0) view.attributes = attributes
  return view
}
const fullView = (m: MockMemory) =>
  withAttributes(
    {
      id: formatId(m.id),
      tags: m.tags,
      summary: m.summary,
      created: compactTime(m.created_at),
      updated: compactTime(m.updated_at),
      content: m.content,
    },
    m,
  )
const summaryView = (m: MockMemory) =>
  withAttributes(
    {
      id: formatId(m.id),
      tags: m.tags,
      summary: m.summary,
      updated: compactTime(m.updated_at),
    },
    m,
  )
// 词汇表按真实服务端形状存全名；作用域词条不因打标签隐式创建（scope_set 显式建）
function ensureTag(name: string): void {
  if (!tags.has(name)) tags.set(name, { description: '', created_at: now() })
}
// 写入的两个容器（与真实服务端同形）：tags 列出裸标签，attributes 给出作用域取值，
// attributes 里的 null 表示该记忆在此作用域下不取任何标签（mock 只摘掉现有取值，
// 不记住声明）。一个作用域在一条记忆里至多一个取值：写入前先摘掉旧取值。
function entityOf(value: unknown): string | null {
  if (value === null || value === undefined) return null
  return String(value)
}
// 整组替换（新建时使用）
function setTagsFromWrite(m: MockMemory, bare: unknown, attributes: unknown): void {
  m.tags = []
  applyTagsWrite(m, bare, attributes)
}
// 逐项写入（新建与更新共用）：裸名先摘后写，作用域取值整段替换
function applyTagsWrite(m: MockMemory, bare: unknown, attributes: unknown): void {
  for (const raw of Array.isArray(bare) ? bare : []) {
    const name = String(raw).trim()
    if (!name || scopeOf(name) !== null) continue
    m.tags = m.tags.filter((x) => x !== name)
    ensureTag(name)
    m.tags.push(name)
  }
  const attrs = attributes as Record<string, unknown> | null | undefined
  if (!attrs || typeof attrs !== 'object') return
  for (const [rawScope, value] of Object.entries(attrs)) {
    const scope = rawScope.trim()
    if (!scope) continue
    const valueText = entityOf(value)
    m.tags = m.tags.filter((x) => scopeOf(x) !== scope)
    if (valueText === null) continue
    const name = `${scope}::${valueText}`
    ensureTag(name)
    m.tags.push(name)
  }
}
// 每个词条的记忆数；作用域另计携带数与词条数——与真实服务端 tag_view_all/scope_view_all 对应
const tagStats = () => {
  const counts = new Map<string, Set<number>>()
  for (const m of memories) {
    for (const tag of m.tags) {
      const set = counts.get(tag) ?? new Set<number>()
      set.add(m.id)
      counts.set(tag, set)
    }
  }
  return counts
}
// 稀疏形状与真实服务端一致（见 ui/lib/src/types.ts）：空描述整个省略、required 仅在
// 必填作用域上出现；filter 按名字做大小写不敏感的子串过滤（真实服务端是正则，UI 不传）
const tagViews = (filter?: string | null) => {
  const counts = tagStats()
  for (const name of counts.keys()) ensureTag(name)
  const q = (filter ?? '').toLowerCase()
  return [...counts.entries()]
    .filter(([name]) => !q || name.toLowerCase().includes(q))
    .map(([name, mems]) => {
      const view: Record<string, unknown> = { name, count: mems.size }
      const description = tags.get(name)?.description
      if (description) view.description = description
      const scope = scopeOf(name)
      if (scope !== null) view.scope = scope
      if (name === 'convention') view.reserved = true
      return view
    })
    .sort((a, b) => (b.count as number) - (a.count as number) || String(a.name).localeCompare(String(b.name)))
}
const scopeViews = (filter?: string | null) => {
  const counts = tagStats()
  const q = (filter ?? '').toLowerCase()
  return [...scopes.entries()]
    .filter(([name]) => !q || name.toLowerCase().includes(q))
    .map(([name, scope]) => {
      const values = [...counts.keys()].filter((tag) => scopeOf(tag) === name)
      const mems = new Set<number>()
      for (const value of values) for (const id of counts.get(value) ?? []) mems.add(id)
      const view: Record<string, unknown> = { name, count: mems.size, tag_count: values.length }
      if (scope.description) view.description = scope.description
      if (scope.required) view.required = true
      return view
    })
    .sort((a, b) => (b.count as number) - (a.count as number) || String(a.name).localeCompare(String(b.name)))
}

// 片段：命中处前后取一段、转义后用 <mark> 包住（UI 侧会再过 DOMPurify）
function snippet(content: string, query: string): string {
  const esc = (s: string): string => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  const i = content.toLowerCase().indexOf(query.toLowerCase())
  if (i < 0) return esc(content.slice(0, 140))
  return (
    esc(content.slice(Math.max(0, i - 50), i)) +
    '<mark>' +
    esc(content.slice(i, i + query.length)) +
    '</mark>' +
    esc(content.slice(i + query.length, i + query.length + 90))
  )
}

// ---- HTTP 小工具 ----
function sendJson(res: ServerResponse, status: number, body: unknown): void {
  res.statusCode = status
  res.setHeader('Content-Type', 'application/json; charset=utf-8')
  res.end(body === undefined ? '' : JSON.stringify(body))
}
const notFound = (res: ServerResponse, method: string, path: string): void =>
  sendJson(res, 404, { error: `no such route: ${method} ${path}` })

function readBody(req: Connect.IncomingMessage): Promise<Record<string, unknown>> {
  return new Promise((resolve) => {
    const chunks: Buffer[] = []
    req.on('data', (c: Buffer) => chunks.push(c))
    req.on('end', () => {
      try {
        resolve(JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}'))
      } catch {
        resolve({})
      }
    })
  })
}

async function handle(req: Connect.IncomingMessage, res: ServerResponse, url: URL, method: string): Promise<void> {
  const segs = url.pathname.split('/').filter(Boolean).map(decodeURIComponent)
  const offset = Number(url.searchParams.get('offset') ?? 0) || 0
  const limit = Number(url.searchParams.get('limit') ?? 20) || 20

  // 免登录：固定 admin 身份，界面四个面板 + 顶栏身份区永远完整显示
  if (url.pathname === '/api/whoami') {
    sendJson(res, 200, {
      name: 'dev',
      mode: 'token',
      permissions: {
        read: true,
        create: true,
        update: true,
        delete: true,
        tag_manage: true,
        predicate_manage: true,
        admin: true,
      },
    })
    return
  }
  if (url.pathname === '/health') {
    sendJson(res, 200, { status: 'ok', version: '0.0.0-mock' })
    return
  }

  if (segs[0] !== 'api') {
    notFound(res, method, url.pathname)
    return
  }

  if (segs[1] === 'predicates' && method === 'GET') {
    sendJson(res, 200, { predicates })
    return
  }
  if (segs[1] === 'predicates' && (method === 'PUT' || method === 'DELETE')) {
    const body = await readBody(req)
    const index = predicates.findIndex((p) => p.name === body.name)
    if (index >= 0) predicates.splice(index, 1)
    if (method === 'PUT')
      predicates.push({
        name: String(body.name),
        predicate: String(body.predicate),
        description: String(body.description ?? ''),
      })
    sendJson(res, 200, method === 'PUT' ? { saved: true } : { deleted: true })
    return
  }

  // 记忆
  if (segs[1] === 'memories' && segs.length === 2 && method === 'GET') {
    const query = (url.searchParams.get('query') ?? '').trim()
    if (query) {
      const hits = memories.filter((m) => (m.summary + m.content).toLowerCase().includes(query.toLowerCase()))
      sendJson(res, 200, {
        total_matches: hits.length,
        offset,
        returned: hits.length,
        mode: 'keyword',
        results: hits.map((m) => ({
          id: formatId(m.id),
          tags: m.tags,
          summary: m.summary,
          score: 1,
          snippet: snippet(m.content, query),
          updated: compactTime(m.updated_at),
        })),
      })
    } else {
      const sorted = [...memories].sort((a, b) => b.updated_at - a.updated_at)
      sendJson(res, 200, {
        total: sorted.length,
        offset,
        limit,
        memories: sorted.slice(offset, offset + limit).map(summaryView),
      })
    }
    return
  }
  if (segs[1] === 'memories' && segs.length === 2 && method === 'POST') {
    const body = await readBody(req)
    const ts = now()
    const memory: MockMemory = {
      id: nextId++,
      summary: String(body.summary ?? ''),
      content: String(body.content ?? ''),
      tags: [],
      created_at: ts,
      updated_at: ts,
    }
    memories.push(memory)
    setTagsFromWrite(memory, body.tags, body.attributes)
    sendJson(res, 200, fullView(memory))
    return
  }
  const memory = segs[1] === 'memories' ? parseId(segs[2]) : undefined
  if (segs[1] === 'memories' && segs.length === 3 && !memory) {
    sendJson(res, 404, { error: 'memory not found' })
    return
  }
  if (segs[1] === 'memories' && segs.length === 3 && memory && method === 'GET') {
    sendJson(res, 200, fullView(memory))
    return
  }
  if (segs[1] === 'memories' && segs.length === 3 && memory && method === 'PUT') {
    const body = await readBody(req)
    if (typeof body.summary === 'string') memory.summary = body.summary
    if (typeof body.content === 'string') memory.content = body.content
    // 增量语义与真实服务端一致：先写 tags/attributes 的声明，再按全名删除（同名双写时删除赢）
    applyTagsWrite(memory, body.tags, body.attributes)
    const removes = Array.isArray(body.remove_tags) ? body.remove_tags.map(String) : []
    if (removes.length > 0) {
      memory.tags = memory.tags.filter((x) => !removes.includes(x))
    }
    memory.updated_at = now()
    sendJson(res, 200, fullView(memory))
    return
  }
  if (segs[1] === 'memories' && segs.length === 3 && memory && method === 'DELETE') {
    memories.splice(memories.indexOf(memory), 1)
    sendJson(res, 200, null)
    return
  }

  // 标签与作用域：两个命名空间并排返回，filter 同时收窄两者（与真实服务端 tag_list 一致）
  if (segs[1] === 'tags' && segs.length === 2 && method === 'GET') {
    sendJson(res, 200, {
      tags: tagViews(url.searchParams.get('filter')),
      scopes: scopeViews(url.searchParams.get('filter')),
    })
    return
  }
  if (segs[1] === 'tags' && segs.length === 2 && method === 'POST') {
    const body = await readBody(req)
    const name = String(body.name ?? '')
    // 带作用域的名字要求作用域已存在（tagging 从不隐式发明作用域）
    const scope = scopeOf(name)
    if (scope !== null && !scopes.has(scope)) {
      sendJson(res, 400, { error: `scope '${scope}' does not exist` })
      return
    }
    tags.set(name, { description: String(body.description ?? ''), created_at: now() })
    sendJson(res, 200, null)
    return
  }
  const tag = segs[1] === 'tags' ? tags.get(segs[2] ?? '') : undefined
  if (segs[1] === 'tags' && segs.length === 3 && !tag) {
    sendJson(res, 404, { error: 'tag not found' })
    return
  }
  if (segs[1] === 'tags' && segs.length === 3 && tag && method === 'PUT') {
    const body = await readBody(req)
    if (typeof body.description === 'string') tag.description = body.description
    const newName = typeof body.new_name === 'string' ? body.new_name : ''
    if (newName && newName !== segs[2]) {
      const oldName = segs[2] as string
      tags.delete(oldName)
      tags.set(newName, tag)
      // 改名跟随词条本身：每条记忆里的旧全名换成新全名
      for (const m of memories) {
        m.tags = m.tags.map((x) => (x === oldName ? newName : x))
      }
    }
    sendJson(res, 200, null)
    return
  }
  if (segs[1] === 'tags' && segs.length === 3 && tag && method === 'DELETE') {
    const name = segs[2] as string
    tags.delete(name)
    // detach：从所有记忆摘除该词条；purge：携带该词条的记忆整个删除
    if (url.searchParams.get('mode') === 'purge') {
      for (const m of [...memories]) if (m.tags.includes(name)) memories.splice(memories.indexOf(m), 1)
    } else {
      for (const m of memories) {
        m.tags = m.tags.filter((x) => x !== name)
      }
    }
    sendJson(res, 200, null)
    return
  }

  // 作用域：GET 与 /api/tags 同形（两个命名空间并排）；POST/PUT 走 scope_set 语义
  // （不存在即创建，存在即更新描述与必填标记）
  if (segs[1] === 'scopes' && segs.length === 2 && method === 'GET') {
    sendJson(res, 200, {
      tags: tagViews(url.searchParams.get('filter')),
      scopes: scopeViews(url.searchParams.get('filter')),
    })
    return
  }
  if (segs[1] === 'scopes' && segs.length === 2 && method === 'POST') {
    const body = await readBody(req)
    const name = String(body.name ?? '')
    if (!name || scopeOf(name) !== null) {
      sendJson(res, 400, { error: 'scope name must be one identifier word' })
      return
    }
    if (!scopes.has(name)) {
      scopes.set(name, {
        description: String(body.description ?? ''),
        created_at: now(),
        required: body.required === true ? true : undefined,
      })
      sendJson(res, 200, { created: true })
      return
    }
    const scope = scopes.get(name)!
    let descriptionUpdated = false
    let requiredUpdated = false
    if (typeof body.description === 'string' && body.description !== scope.description) {
      scope.description = body.description
      descriptionUpdated = true
    }
    if (typeof body.required === 'boolean' && body.required !== (scope.required === true)) {
      scope.required = body.required ? true : undefined
      requiredUpdated = true
    }
    sendJson(res, 200, { description_updated: descriptionUpdated, required_updated: requiredUpdated })
    return
  }
  const scope = segs[1] === 'scopes' ? scopes.get(segs[2] ?? '') : undefined
  if (segs[1] === 'scopes' && segs.length === 3 && !scope) {
    sendJson(res, 404, { error: 'scope not found' })
    return
  }
  if (segs[1] === 'scopes' && segs.length === 3 && scope && method === 'PUT') {
    const body = await readBody(req)
    let descriptionUpdated = false
    let requiredUpdated = false
    if (typeof body.description === 'string' && body.description !== scope.description) {
      scope.description = body.description
      descriptionUpdated = true
    }
    if (typeof body.required === 'boolean' && body.required !== (scope.required === true)) {
      scope.required = body.required ? true : undefined
      requiredUpdated = true
    }
    sendJson(res, 200, { description_updated: descriptionUpdated, required_updated: requiredUpdated })
    return
  }

  // 管理界面的运行时面板：真实服务端按进程状态返回，mock 给出可观测的样例值
  if (segs[1] === 'search' && segs[2] === 'runtime' && method === 'GET') {
    const limits = (settings as unknown as Record<string, unknown>).search_limits as
      | {
          semantic_candidates?: number
          adaptive?: { enabled?: boolean; min_candidates?: number }
        }
      | undefined
    const adaptive = limits?.adaptive?.enabled === true
    const selected = adaptive ? (limits?.adaptive?.min_candidates ?? 20) : (limits?.semantic_candidates ?? 100)
    sendJson(res, 200, {
      process_local: true,
      adaptive_enabled: adaptive,
      models: [
        {
          id: 'local-model',
          selected_candidates: selected,
          last_candidates: Math.min(1000, selected + 4),
          last_latency_ms: adaptive ? 640 : 912,
          samples: 24,
          failures: 0,
        },
      ],
    })
    return
  }
  if (segs[1] === 'access' && segs[2] === 'stats' && method === 'GET') {
    sendJson(res, 200, {
      raw_events: 128,
      hourly_buckets: 42,
      tracked_memories: Math.max(1, memories.length - 1),
      dropped_read_events: 0,
    })
    return
  }
  if (segs[1] === 'access' && (segs[2] === 'compact' || segs[2] === 'rebuild') && method === 'POST') {
    sendJson(res, 200, {})
    return
  }
  // 没有任何向量缓存（未配置 embedding 时的真实形态就是一个空列表）
  if (segs[1] === 'embeddings' && segs[2] === 'caches' && method === 'GET') {
    sendJson(res, 200, { caches: [] })
    return
  }

  // 运维：统计从内存推导，其余固定应答
  if (segs[1] === 'stats' && method === 'GET') {
    const newest = memories.reduce<MockMemory | null>((a, m) => (!a || m.updated_at > a.updated_at ? m : a), null)
    sendJson(res, 200, {
      path: '(mock: in-memory)',
      memories: memories.length,
      tags: tags.size,
      file_size: 4096,
      schema_version: 1,
      newest_update: newest ? { id: formatId(newest.id), updated_at: newest.updated_at } : null,
      embedding: { enabled: false },
    })
    return
  }
  if (segs[1] === 'doctor' && method === 'GET') {
    sendJson(res, 200, { ok: true, issues: [] })
    return
  }
  if (segs[1] === 'export' && method === 'GET') {
    sendJson(res, 200, {
      exported_at: now(),
      // 导出保持主数据形状（epoch 秒），对齐真实服务的备份格式（memory_view 才做本地渲染）
      // 必填约定随作用域携带生效时间；mock 只存布尔，用创建时刻顶替
      scopes: [...scopes.entries()].map(([name, scope]) => ({
        name,
        description: scope.description,
        ...(scope.required ? { required_since: scope.created_at } : {}),
      })),
      tags: [...tags.entries()].map(([name, tag]) => ({ name, description: tag.description })),
      memories: memories.map((m) => ({
        summary: m.summary,
        content: m.content,
        tags: m.tags,
        created_at: m.created_at,
        updated_at: m.updated_at,
      })),
    })
    return
  }
  if (segs[1] === 'import' && method === 'POST') {
    sendJson(res, 200, {
      imported_memories: 0,
      imported_tags: 0,
      skipped_memories: 0,
      unclassified_memories: 0,
    })
    return
  }
  if (segs[1] === 'embeddings' && method === 'POST') {
    sendJson(res, 200, segs[2] === 'test' ? { ok: false, error: 'mock 环境未配置 embedding' } : { configured: false })
    return
  }

  // 身份与设置：一条静态 admin 记录 + settings 对象的朴素读写
  if (segs[1] === 'identities' && segs.length === 2 && method === 'GET') {
    sendJson(res, 200, {
      identities: [
        {
          name: 'dev',
          token_hint: 'mock...token',
          permissions: {
            read: true,
            create: true,
            update: true,
            delete: true,
            tag_manage: true,
            predicate_manage: true,
            admin: true,
          },
          created_at: t - 30 * DAY,
        },
      ],
    })
    return
  }
  if (segs[1] === 'identities' && method === 'POST') {
    sendJson(res, 200, { token: `mock-${Math.random().toString(36).slice(2)}` })
    return
  }
  if (segs[1] === 'identities' && (method === 'PUT' || method === 'DELETE')) {
    sendJson(res, 200, null)
    return
  }
  if (segs[1] === 'identities' && segs[2] === 'token-reset' && method === 'POST') {
    sendJson(res, 200, { token: `mock-${Math.random().toString(36).slice(2)}` })
    return
  }
  if (segs[1] === 'settings' && method === 'GET') {
    sendJson(res, 200, { ...settings, default_instructions: '（mock）内置默认提示词占位。' })
    return
  }
  if (segs[1] === 'settings' && method === 'PUT') {
    Object.assign(settings, await readBody(req))
    sendJson(res, 200, { ...settings, default_instructions: '（mock）内置默认提示词占位。' })
    return
  }

  notFound(res, method, url.pathname)
}

export function mockApi(): Plugin {
  return {
    name: 'agent-memory-mock-api',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = new URL(req.url ?? '/', 'http://localhost')
        if (url.pathname !== '/health' && !url.pathname.startsWith('/api/')) {
          next()
          return
        }
        void handle(req, res, url, (req.method ?? 'GET').toUpperCase()).catch((e: unknown) =>
          sendJson(res, 500, { error: e instanceof Error ? e.message : String(e) }),
        )
      })
      server.httpServer?.once('listening', () => {
        server.config.logger.info(
          '\n  \x1b[32m➜\x1b[0m  mock API 已启用（内存假数据，重启复位）；连真实后端用 \x1b[36mnpm run dev:live\x1b[0m\n',
        )
      })
    },
  }
}
