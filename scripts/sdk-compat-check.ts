// Modern-protocol compatibility check: a hand-written MCP client (2026-07-28) against the
// agent-memory HTTP server.
//
// The official TypeScript SDK has not shipped a modern-protocol client yet (1.x speaks the
// legacy initialize handshake, which this server deliberately rejects), so this script drives
// the protocol by hand over fetch (Node >= 24). When an SDK v2 lands, swap the transport back.
//
// Usage (start the server first, e.g. agent-memory serve --port 8899):
//   node scripts/sdk-compat-check.ts            # defaults to http://127.0.0.1:8899/mcp
//   MCP_URL=http://127.0.0.1:8899/mcp node scripts/sdk-compat-check.ts
// When the server has token auth enabled: MCP_TOKEN=xxx node scripts/sdk-compat-check.ts
// Exits 0 when all checks pass; suitable for a manual pre-release regression.
// Test data is cleaned up regardless of the outcome.

const ENDPOINT = process.env.MCP_URL || "http://127.0.0.1:8899/mcp";
const TOKEN = process.env.MCP_TOKEN;
const PROTOCOL_VERSION = "2026-07-28";

const failures: string[] = [];
function check(name: string, cond: boolean, extra = ""): void {
  console.log(`${cond ? "PASS" : "FAIL"}  ${name}${extra ? "  " + extra : ""}`);
  if (!cond) failures.push(name);
}

function authHeaders(): Record<string, string> {
  return TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {};
}

/** The per-request `_meta` every modern request must carry. */
function meta(): Record<string, unknown> {
  return {
    "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
    "io.modelcontextprotocol/clientInfo": { name: "sdk-compat-check", version: "0.0.1" },
    "io.modelcontextprotocol/clientCapabilities": {},
  };
}

interface RpcOk {
  status: number
  result?: Record<string, unknown>
  error?: { code: number, message?: string, data?: unknown }
}

/** POST one JSON-RPC message with conforming mirrored headers; parse the JSON-RPC reply. */
async function rpc(
  id: string | number,
  method: string,
  params: Record<string, unknown> = {},
): Promise<RpcOk> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
    "MCP-Protocol-Version": PROTOCOL_VERSION,
    "Mcp-Method": method,
    ...authHeaders(),
  };
  if (method === "tools/call" && typeof params.name === "string") headers["Mcp-Name"] = params.name
  if (method === "resources/read" && typeof params.uri === "string") headers["Mcp-Name"] = params.uri

  const res = await fetch(ENDPOINT, {
    method: "POST",
    headers,
    body: JSON.stringify({ jsonrpc: "2.0", id, method, params: { ...params, _meta: meta() } }),
  });
  if (res.status === 202) return { status: 202 }
  const body = (await res.json()) as Record<string, never>
  return { status: res.status, ...body }
}

function structuredOf(r: RpcOk): Record<string, unknown> | undefined {
  return r.result?.structuredContent as Record<string, unknown> | undefined
}

/**
 * subscriptions/listen as a real client: POST, then read the SSE stream incrementally until a
 * predicate matches or the deadline passes. The stream is aborted on return.
 */
async function listenFor(
  id: string | number,
  filter: Record<string, unknown>,
  want: (message: Record<string, unknown>) => boolean,
  timeoutMs: number,
  /** Called once the acknowledgment arrived: the place to trigger the watched write. */
  trigger?: () => Promise<void>,
): Promise<{ matched: Record<string, unknown> | null, ack: Record<string, unknown> | null }> {
  const controller = new AbortController()
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
    "MCP-Protocol-Version": PROTOCOL_VERSION,
    "Mcp-Method": "subscriptions/listen",
    ...authHeaders(),
  }
  const res = await fetch(ENDPOINT, {
    method: "POST",
    headers,
    body: JSON.stringify({
      jsonrpc: "2.0",
      id,
      method: "subscriptions/listen",
      params: { ...filter, _meta: meta() },
    }),
    signal: controller.signal,
  })
  if (!res.ok || !res.body) {
    controller.abort()
    throw new Error(`listen failed: HTTP ${res.status}`)
  }
  const reader = (res.body as ReadableStream<Uint8Array>).getReader()
  const decoder = new TextDecoder()
  let buffer = ""
  let ack: Record<string, unknown> | null = null
  let matched: Record<string, unknown> | null = null
  const deadline = Date.now() + timeoutMs
  try {
    while (Date.now() < deadline && matched === null) {
      const chunk = await Promise.race([
        reader.read(),
        new Promise<null>((resolve) => setTimeout(() => resolve(null), Math.max(deadline - Date.now(), 1))),
      ])
      if (!chunk) break // timed out
      if (chunk.done) break
      buffer += decoder.decode(chunk.value, { stream: true })
      // SSE framing: events separated by a blank line; ignore comment (keep-alive) lines
      let sep: number
      while ((sep = buffer.indexOf("\n\n")) !== -1 && matched === null) {
        const block = buffer.slice(0, sep)
        buffer = buffer.slice(sep + 2)
        const dataLine = block.split("\n").find((l) => l.startsWith("data:"))
        if (!dataLine) continue // keep-alive comment or malformed block
        const message = JSON.parse(dataLine.slice(5).trim()) as Record<string, unknown>
        const method = message.method as string | undefined
        if (method === "notifications/subscriptions/acknowledged") {
          ack = message.params as Record<string, unknown>
          if (trigger) await trigger()
          continue
        }
        if (method && want(message)) matched = message
      }
    }
  } finally {
    controller.abort()
  }
  return { matched, ack }
}

let createdId: string | null = null
let done = false

try {
  // ---- Discovery replaces the legacy initialize handshake
  const discover = await rpc("discover-1", "server/discover")
  const d = discover.result ?? {}
  check("server/discover succeeds", discover.status === 200 && d.resultType === "complete")
  check(
    "discover lists the supported version",
    JSON.stringify(d.supportedVersions) === JSON.stringify([PROTOCOL_VERSION]),
    JSON.stringify(d.supportedVersions),
  )
  const capabilities = d.capabilities as Record<string, Record<string, unknown>> | undefined
  check(
    "discover advertises resources with subscribe",
    capabilities?.resources?.listChanged === true && capabilities?.resources?.subscribe === true,
  )
  const serverInfo = (d._meta as Record<string, Record<string, unknown>> | undefined)?.[
    "io.modelcontextprotocol/serverInfo"
  ]
  check(
    "discover carries serverInfo",
    serverInfo?.name === "agent-memory",
    JSON.stringify(serverInfo),
  )
  const instructions = (d.instructions as string | undefined) ?? ""
  check(
    "discover carries an identity line (open mode)",
    instructions.includes("Access mode: open"),
  )
  check(
    "discover is private and immediately stale (identity-dependent)",
    d.ttlMs === 0 && d.cacheScope === "private",
  )

  // ---- tools/list: the full inventory plus static caching hints
  const tools = await rpc(2, "tools/list")
  const toolNames = ((tools.result?.tools as { name: string }[] | undefined) ?? []).map((t) => t.name)
  check("tools/list returns 11 tools", toolNames.length === 11, `got ${toolNames.length}`)
  check(
    "tools/list is public and cacheable",
    tools.result?.cacheScope === "public" && tools.result?.ttlMs === 3_600_000,
  )

  // ---- Tool flow: create -> search (progressive disclosure) -> fetch full text
  const created = await rpc(3, "tools/call", {
    name: "memory_create",
    arguments: {
      summary: "SDK 联调记忆：现代协议客户端可用",
      content: "手写 2026-07-28 客户端连接成功，含 resources 与订阅流。",
      tags: ["sdk", "联调"],
      // Fixture tags: auto-create deliberately, the default now rejects unknown names
      create_missing_tags: true,
    },
  })
  check("tools/call memory_create", created.result?.isError !== true, JSON.stringify(created).slice(0, 120))
  const structured = structuredOf(created) as
    | { memory?: { id?: string }, total_matches?: number, results?: { content?: unknown }[] }
    | undefined
  createdId = structured?.memory?.id ?? null
  check(
    "create returns structured id",
    typeof createdId === "string" && createdId.startsWith("m"),
    createdId ?? "",
  )
  if (createdId === null) throw new Error("create did not return an id")

  // Unknown tags are rejected by default (with the closest existing names listed); the flag opts in
  const unknownTag = await rpc(31, "tools/call", {
    name: "memory_create",
    arguments: { summary: "tag gate probe", content: "body", tags: ["sdk"] },
  })
  check("existing tags need no create_missing_tags", unknownTag.result?.isError !== true)
  const rejectedTag = await rpc(32, "tools/call", {
    name: "memory_create",
    arguments: { summary: "tag gate probe 2", content: "body", tags: ["no-such-tag-xyz"] },
  })
  check(
    "unknown tags rejected unless create_missing_tags",
    rejectedTag.result?.isError === true &&
      JSON.stringify(rejectedTag.result).includes("unknown tags"),
    JSON.stringify(rejectedTag.result).slice(0, 120),
  )

  const searched = await rpc(4, "tools/call", { name: "memory_search", arguments: { query: "联调" } })
  const searchedStructured = structuredOf(searched) as typeof structured
  check("memory_search finds it", searchedStructured?.total_matches === 1)
  check(
    "search does not leak content",
    searchedStructured?.results?.[0]?.content === undefined,
  )

  const got = await rpc(5, "tools/call", { name: "memory_get", arguments: { ids: [createdId] } })
  check(
    "memory_get reveals content",
    (
      (structuredOf(got) as { memories?: { content?: string }[] } | undefined)?.memories?.[0]
        ?.content as string | undefined
    )?.includes("2026-07-28") === true,
  )

  const updated = await rpc(6, "tools/call", {
    name: "memory_update",
    arguments: { id: createdId, add_tags: ["验证完成"], create_missing_tags: true },
  })
  check(
    "memory_update add_tags",
    (
      (structuredOf(updated) as { memory?: { tags?: string[] } } | undefined)?.memory?.tags as
        | string[]
        | undefined
    )?.includes("验证完成") === true,
  )

  // ---- Duplicate lifecycle: create a near-duplicate, merge it back, preview a purge
  const dup = await rpc(33, "tools/call", {
    name: "memory_create",
    arguments: { summary: "sdk 联调记忆：现代协议客户端可用", content: "重复内容，等待合并。" },
  })
  const dupId =
    (structuredOf(dup) as { memory?: { id?: string } } | undefined)?.memory?.id ?? null
  check("near-duplicate stored (duplicate_of is advisory)", dup.result?.isError !== true && typeof dupId === "string")
  if (dupId !== null) {
    const merged = await rpc(34, "tools/call", {
      name: "memory_merge",
      arguments: { target: createdId, source: dupId },
    })
    const mergedStructured = structuredOf(merged) as
      | { removed?: string, memory?: { id?: string, created?: string } }
      | undefined
    check(
      "memory_merge keeps the target and removes the source",
      mergedStructured?.removed === dupId && mergedStructured?.memory?.id === createdId,
    )
    const gone = await rpc(35, "tools/call", { name: "memory_get", arguments: { ids: [dupId] } })
    check(
      "merged source is gone",
      ((structuredOf(gone) as { missing?: string[] } | undefined)?.missing ?? []).includes(dupId),
    )
  }

  const tagList = await rpc(36, "tools/call", { name: "tag_list", arguments: {} })
  const conventions = (structuredOf(tagList) as { tags?: { name?: string, reserved?: boolean }[] })
    ?.tags?.find((t) => t.name === "conventions")
  check("reserved tag listed with reserved flag", conventions?.reserved === true)

  const purgePreview = await rpc(37, "tools/call", {
    name: "tag_delete",
    arguments: { name: "sdk", mode: "purge", dry_run: true },
  })
  const preview = structuredOf(purgePreview) as { dry_run?: boolean, memories_affected?: number } | undefined
  check(
    "tag_delete dry_run previews without deleting",
    preview?.dry_run === true && (preview?.memories_affected ?? 0) >= 1,
  )
  const stillThere = await rpc(38, "tools/call", { name: "memory_search", arguments: { query: "联调" } })
  check(
    "dry_run deleted nothing",
    (structuredOf(stillThere) as { total_matches?: number } | undefined)?.total_matches !== 0,
  )

  // ---- Error channels
  const unknownTool = await rpc(7, "tools/call", { name: "no_such_tool" })
  check(
    "unknown tool surfaces as -32602 on HTTP 200",
    unknownTool.status === 200 && unknownTool.error?.code === -32602,
  )
  const unknownMethod = await rpc(8, "bogus/method")
  check(
    "unknown method surfaces as 404 + -32601",
    unknownMethod.status === 404 && unknownMethod.error?.code === -32601,
  )
  const argError = await rpc(9, "tools/call", {
    name: "memory_create",
    arguments: { summary: "only summary" },
  })
  check("tool argument error surfaces as isError", argError.result?.isError === true)
  const noMeta = await fetch(ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "MCP-Protocol-Version": PROTOCOL_VERSION,
      "Mcp-Method": "ping",
      ...authHeaders(),
    },
    body: JSON.stringify({ jsonrpc: "2.0", id: 10, method: "ping" }),
  })
  const noMetaBody = (await noMeta.json()) as { error?: { code?: number } }
  check(
    "missing _meta rejected with 400 + -32602",
    noMeta.status === 400 && noMetaBody.error?.code === -32602,
  )
  const headerMismatch = await fetch(ENDPOINT, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "MCP-Protocol-Version": "2025-06-18",
      "Mcp-Method": "ping",
      ...authHeaders(),
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 11,
      method: "ping",
      params: { _meta: meta() },
    }),
  })
  const mismatchBody = (await headerMismatch.json()) as { error?: { code?: number } }
  check(
    "header/body version mismatch rejected with 400 + -32020",
    headerMismatch.status === 400 && mismatchBody.error?.code === -32020,
  )
  const legacyInit = await rpc(12, "initialize", { protocolVersion: "2025-06-18" })
  check(
    "legacy initialize answered with -32022 naming supported versions",
    legacyInit.status === 400 && legacyInit.error?.code === -32022,
  )
  const batch = await fetch(ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: JSON.stringify([
      { jsonrpc: "2.0", id: 13, method: "ping" },
      { jsonrpc: "2.0", method: "notifications/initialized" },
    ]),
  })
  const batchBody = (await batch.json()) as { error?: { code?: number } }
  check(
    "batch bodies rejected with 400 + -32600",
    batch.status === 400 && batchBody.error?.code === -32600,
  )

  // ---- Resources: templates, catalog (no content), tag directory, memory body
  const templates = await rpc(14, "resources/templates/list")
  const templateUris = (
    (templates.result?.resourceTemplates as { uriTemplate: string }[] | undefined) ?? []
  ).map((t) => t.uriTemplate)
  check(
    "templates list the two memory:// shapes",
    JSON.stringify(templateUris) ===
      JSON.stringify(["memory://tags/{tag}", "memory://memories/{id}"]),
    JSON.stringify(templateUris),
  )

  const list = await rpc(15, "resources/list")
  const resources = (list.result?.resources as { uri: string, text?: string }[] | undefined) ?? []
  check(
    "resources/list carries tag entries",
    resources.some((r) => r.uri === "memory://tags/sdk"),
    JSON.stringify(resources.map((r) => r.uri)),
  )
  check(
    "catalog entries carry no content",
    resources.every((r) => r.text === undefined),
  )

  const tagRead = await rpc(16, "resources/read", { uri: "memory://tags/sdk" })
  const tagDoc = JSON.parse(
    (tagRead.result?.contents as { text: string }[] | undefined)?.[0]?.text ?? "{}",
  ) as { name?: string, memories?: { id?: string }[], count?: number }
  check(
    "tag directory read returns the JSON catalog",
    tagDoc.name === "sdk" && (tagDoc.count ?? 0) >= 1,
  )
  check(
    "tag directory leaks no memory content",
    !("content" in ((tagDoc.memories?.[0] as Record<string, unknown> | undefined) ?? {})),
  )

  const memoryRead = await rpc(17, "resources/read", { uri: `memory://memories/${createdId}` })
  const memoryContent = (memoryRead.result?.contents as { mimeType?: string, text?: string }[] | undefined)?.[0]
  check(
    "memory read returns markdown content",
    memoryContent?.mimeType === "text/markdown" &&
      (memoryContent?.text as string | undefined)?.includes("2026-07-28") === true,
  )
  const missingRead = await rpc(18, "resources/read", { uri: "memory://memories/m999999" })
  check(
    "missing resource answers -32602 with data.uri",
    missingRead.status === 400 && missingRead.error?.code === -32602,
  )

  // ---- Subscriptions: open a stream, trigger a write from another request, receive the event
  // The stream opens first; the watched write fires once the acknowledgment arrived.
  // Note: the sdk tag already exists by now, so this write is the one catalog change.
  const subscription = await listenFor(
    "listen-1",
    { notifications: { resourcesListChanged: true, resourceSubscriptions: ["memory://tags/sdk"] } },
    (message) => message.method === "notifications/resources/list_changed",
    10_000,
    async () => {
      await rpc(19, "tools/call", { name: "tag_create", arguments: { name: "sdk-subscribe" } })
    },
  )
  const ackNotifications = (subscription.ack?.notifications as Record<string, unknown> | undefined) ?? {}
  check("subscription acknowledged with the honored subset", ackNotifications.resourcesListChanged === true)
  check(
    "acknowledgment carries the subscription id",
    (subscription.ack?._meta as Record<string, unknown> | undefined)?.[
      "io.modelcontextprotocol/subscriptionId"
    ] === "listen-1",
  )
  check("write during subscription delivers list_changed", subscription.matched !== null)

  // ---- Stateless server: a fresh request context sees the same data (no session state)
  const again = await rpc(20, "tools/call", { name: "memory_search", arguments: { query: "联调" } })
  check("reconnect sees previous data (stateless server)", structuredOf(again)?.total_matches === 1)

  const pong = await rpc(21, "ping")
  check("ping roundtrip", pong.result?.resultType === "complete")

  done = true
} catch (e) {
  failures.push(`unexpected error: ${(e as Error).message}`)
  console.log("ERROR", (e as Error).stack)
} finally {
  // Clean up test data and close connections regardless of outcome
  if (createdId) {
    try {
      await rpc("cleanup-1", "tools/call", {
        name: "memory_delete",
        arguments: { ids: [createdId] },
      })
      await rpc("cleanup-2", "tools/call", {
        name: "tag_delete",
        arguments: { name: "sdk-subscribe", mode: "detach" },
      })
    } catch (e) {
      console.log("cleanup failed (ignored):", (e as Error).message)
    }
  }
}

console.log(
  failures.length === 0 && done
    ? "\nALL CHECKS PASSED"
    : `\n${failures.length} FAILURES: ${failures.join(", ")}`,
)
process.exit(failures.length === 0 && done ? 0 : 1)
