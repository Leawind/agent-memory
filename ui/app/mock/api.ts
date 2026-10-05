// dev 专用 mock API（Vite 中间件，`npm run dev` 默认启用）：用内存假数据喂饱四个面板，
// 让前端改完代码刷新就能看到界面，不依赖 Rust 后端；`--mode live` 不加载本插件。
// 只求界面能渲染、增删改看得见变化，不模拟鉴权/搜索评分等服务端语义——那类行为
// 用 `npm run dev:live` 连真实后端验证。数据只存内存，dev server 重启即复位。
import type { ServerResponse } from 'node:http'
import type { Connect, Plugin } from 'vite'

const now = (): number => Math.floor(Date.now() / 1000)
const DAY = 86_400

interface MockMemory {
  id: number
  summary: string
  content: string
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
    tags: ['rust', '架构'],
    created_at: t - 30 * DAY,
    updated_at: t - 2 * DAY,
  },
  {
    id: 2,
    summary: 'MCP protocol versions',
    content:
      '## Protocol compatibility\n\nWe support `2024-11-05`, `2025-03-26` and `2025-06-18`.\n\n- id boundary format is strictly `"m{n}"`\n- notifications never get a response',
    tags: ['mcp', 'rust'],
    created_at: t - 25 * DAY,
    updated_at: t - 3 * DAY,
  },
  {
    id: 3,
    summary: '组件库面板设计',
    content:
      '# 工作台组件\n\n1. `TagsSidebar`：左侧标签侧栏，点击弹 `TagDialog`\n2. `MemoriesPanel`：卡片列表，整卡点击弹记忆弹窗\n3. `AdminPanel`：身份、鉴权开关、提示词、备份（挂在管理弹窗里）\n\n概况是弹窗 `OpsDialog`：点击顶栏标题打开，每次打开重拉数据。样式全部引用 `--el-*` 变量，跟随宿主主题。',
    tags: ['前端', '设计'],
    created_at: t - 20 * DAY,
    updated_at: t - 1 * DAY,
  },
  {
    id: 4,
    summary: 'SQLite WAL 事务纪律',
    content: '读写分离：读请求 `BEGIN DEFERRED`（一致性快照、不抢写锁），写请求 `BEGIN IMMEDIATE`（一开始就取写锁）。',
    tags: ['rust', '架构'],
    created_at: t - 18 * DAY,
    updated_at: t - 4 * DAY,
  },
  {
    id: 5,
    summary: '发布前检查清单',
    content: '1. `cargo test` 全绿\n2. `cargo clippy --all-targets` 零告警\n3. `npm run typecheck` 通过',
    tags: ['流程', '测试'],
    created_at: t - 8 * DAY,
    updated_at: t - 10 * 3600,
  },
]
const tags = new Map(
  [
    ['rust', 'Rust 后端与工具链'],
    ['架构', '模块划分与不变量'],
    ['mcp', 'Model Context Protocol'],
    ['前端', 'Vue3 组件库与站点'],
    ['设计', '交互与视觉决定'],
    ['测试', '单元与端到端'],
  ].map(([name, description]) => [name, { description, created_at: t - 30 * DAY }]),
)
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
const fullView = (m: MockMemory) => ({
  id: formatId(m.id),
  tags: m.tags,
  summary: m.summary,
  created: compactTime(m.created_at),
  updated: compactTime(m.updated_at),
  content: m.content,
})
const summaryView = (m: MockMemory) => ({
  id: formatId(m.id),
  tags: m.tags,
  summary: m.summary,
  updated: compactTime(m.updated_at),
})
// 稀疏形状与真实服务端一致（见 ui/lib/src/types.ts）：空描述整个省略、reserved 仅保留标签出现
const tagViews = () =>
  [...tags.entries()]
    .map(([name, tag]) => {
      const view: Record<string, unknown> = {
        name,
        count: memories.filter((m) => m.tags.includes(name)).length,
      }
      if (tag.description) view.description = tag.description
      return view
    })
    .sort((a, b) => (b.count as number) - (a.count as number) || String(a.name).localeCompare(String(b.name)))

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
  const segs = url.pathname.split('/').filter(Boolean)
  const offset = Number(url.searchParams.get('offset') ?? 0) || 0
  const limit = Number(url.searchParams.get('limit') ?? 20) || 20

  // 免登录：固定 admin 身份，界面四个面板 + 顶栏身份区永远完整显示
  if (url.pathname === '/api/whoami') {
    sendJson(res, 200, {
      name: 'dev',
      mode: 'token',
      permissions: { read: true, create: true, update: true, delete: true, tag_manage: true, admin: true },
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
          updated_at: m.updated_at,
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
      tags: Array.isArray(body.tags) ? body.tags.map(String) : [],
      created_at: ts,
      updated_at: ts,
    }
    memories.push(memory)
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
    const add = Array.isArray(body.add_tags) ? body.add_tags.map(String) : []
    const remove = Array.isArray(body.remove_tags) ? body.remove_tags.map(String) : []
    if (typeof body.summary === 'string') memory.summary = body.summary
    if (typeof body.content === 'string') memory.content = body.content
    memory.tags.push(...add.filter((x) => !memory.tags.includes(x)))
    memory.tags = memory.tags.filter((x) => !remove.includes(x))
    memory.updated_at = now()
    sendJson(res, 200, fullView(memory))
    return
  }
  if (segs[1] === 'memories' && segs.length === 3 && memory && method === 'DELETE') {
    memories.splice(memories.indexOf(memory), 1)
    sendJson(res, 200, null)
    return
  }

  // 标签
  if (segs[1] === 'tags' && segs.length === 2 && method === 'GET') {
    sendJson(res, 200, { tags: tagViews() })
    return
  }
  if (segs[1] === 'tags' && segs.length === 2 && method === 'POST') {
    const body = await readBody(req)
    tags.set(String(body.name ?? ''), { description: String(body.description ?? ''), created_at: now() })
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
      tags.delete(segs[2] as string)
      tags.set(newName, tag)
      for (const m of memories) m.tags = m.tags.map((x) => (x === segs[2] ? newName : x))
    }
    sendJson(res, 200, null)
    return
  }
  if (segs[1] === 'tags' && segs.length === 3 && tag && method === 'DELETE') {
    const name = segs[2] as string
    tags.delete(name)
    if (url.searchParams.get('mode') === 'purge') {
      for (const m of [...memories]) if (m.tags.includes(name)) memories.splice(memories.indexOf(m), 1)
    } else {
      for (const m of memories) m.tags = m.tags.filter((x) => x !== name)
    }
    sendJson(res, 200, null)
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
    sendJson(res, 200, { imported_memories: 0, imported_tags: 0 })
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
          permissions: { read: true, create: true, update: true, delete: true, tag_manage: true, admin: true },
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
