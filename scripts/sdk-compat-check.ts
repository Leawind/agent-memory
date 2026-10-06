// Dual-era MCP compatibility check: hand-written clients (modern 2026-07-28 and legacy
// 2025-06-18) against the agent-memory HTTP server.
//
// Both protocol surfaces are driven by hand over fetch (Node >= 24): the modern era for its
// per-request `_meta` + mirrored headers, the legacy era the way an official SDK 1.x client
// speaks it (initialize handshake, no per-request metadata).
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
const LEGACY_PROTOCOL_VERSION = "2025-06-18";

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

/** The raw text block of a tool result (the single data channel). */
function textOf(r: RpcOk): string | undefined {
  const blocks = r.result?.content as { type?: string, text?: string }[] | undefined
  return blocks?.find(b => b.type === 'text')?.text
}

/** The tool-result payload: the text content block parsed as JSON. List tools (tag_list,
 * memory_list, memory_search) render the compact line format instead — use textOf + lineHeader. */
function payloadOf(r: RpcOk): Record<string, unknown> | undefined {
  const text = textOf(r)
  return text === undefined ? undefined : (JSON.parse(text) as Record<string, unknown>)
}

/** Header line of the list-tool line format: `key: V | key: V | ...` → object. */
function lineHeader(text: string): Record<string, string> {
  const header = text.slice(0, text.indexOf("\n"))
  return Object.fromEntries(
    header.split(" | ").map((part) => {
      const sep = part.indexOf(": ")
      return [part.slice(0, sep), part.slice(sep + 2)]
    }),
  )
}

/** POST one JSON-RPC message the way a legacy (2025-06-18) client does: no `_meta`, no mirrored
 * headers — the initialize handshake pins the version, nothing else is carried per request. */
async function legacyRpc(
  id: string | number,
  method: string,
  params: Record<string, unknown> = {},
): Promise<RpcOk> {
  const res = await fetch(ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream", ...authHeaders() },
    body: JSON.stringify({ jsonrpc: "2.0", id, method, params }),
  })
  if (res.status === 202) return { status: 202 }
  const body = (await res.json()) as Record<string, never>
  return { status: res.status, ...body }
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
    "discover lists both supported versions",
    JSON.stringify(d.supportedVersions) ===
      JSON.stringify([PROTOCOL_VERSION, LEGACY_PROTOCOL_VERSION]),
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
  check("tools/list returns 12 tools", toolNames.length === 12, `got ${toolNames.length}`)
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
  const structured = payloadOf(created) as { memory?: { id?: string } } | undefined
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
  const searchedText = textOf(searched) ?? ""
  check("memory_search finds it", lineHeader(searchedText).total_matches === "1")
  check(
    "search returns summary rows only (no content column)",
    searchedText.split("\n").every((l) => !l.startsWith("content")),
  )

  const got = await rpc(5, "tools/call", { name: "memory_get", arguments: { ids: [createdId] } })
  check(
    "memory_get reveals content",
    (
      (payloadOf(got) as { memories?: { content?: string }[] } | undefined)?.memories?.[0]
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
      (payloadOf(updated) as { memory?: { tags?: string[] } } | undefined)?.memory?.tags as
        | string[]
        | undefined
    )?.includes("验证完成") === true,
  )

  // ---- memory_edit: span replacement inside existing content (no full restatement)
  const edited = await rpc(39, "tools/call", {
    name: "memory_edit",
    arguments: { id: createdId, old_string: "手写 2026-07-28 客户端连接成功", new_string: "手写 2026-07-28 客户端全流程跑通" },
  })
  check("memory_edit replaces the span", edited.result?.isError !== true, JSON.stringify(edited).slice(0, 120))
  const editStructured = payloadOf(edited) as { replaced?: number } | undefined
  check("memory_edit reports the replacement count", editStructured?.replaced === 1)
  const editedMiss = await rpc(40, "tools/call", {
    name: "memory_edit",
    arguments: { id: createdId, old_string: "not present anywhere", new_string: "x" },
  })
  check(
    "memory_edit missing old_string fails without side effects",
    editedMiss.result?.isError === true && JSON.stringify(editedMiss.result).includes("not found"),
  )

  // ---- Duplicate lifecycle: create a near-duplicate, merge it back, preview a purge
  const dup = await rpc(33, "tools/call", {
    name: "memory_create",
    arguments: { summary: "sdk 联调记忆：现代协议客户端可用", content: "重复内容，等待合并。" },
  })
  const dupId =
    (payloadOf(dup) as { memory?: { id?: string } } | undefined)?.memory?.id ?? null
  check("near-duplicate stored (duplicate_of is advisory)", dup.result?.isError !== true && typeof dupId === "string")
  if (dupId !== null) {
    const merged = await rpc(34, "tools/call", {
      name: "memory_merge",
      arguments: { target: createdId, source: dupId },
    })
    const mergedStructured = payloadOf(merged) as
      | { removed?: string, memory?: { id?: string, created?: string } }
      | undefined
    check(
      "memory_merge keeps the target and removes the source",
      mergedStructured?.removed === dupId && mergedStructured?.memory?.id === createdId,
    )
    const gone = await rpc(35, "tools/call", { name: "memory_get", arguments: { ids: [dupId] } })
    check(
      "merged source is gone",
      ((payloadOf(gone) as { missing?: string[] } | undefined)?.missing ?? []).includes(dupId),
    )
  }

  const tagList = await rpc(36, "tools/call", { name: "tag_list", arguments: {} })
  // Line format: `<count> <[*]name>[: <description>]` — the reserved flag is the `*` before the name
  const conventionsLine = (textOf(tagList) ?? "")
    .split("\n")
    .find((l) => /^\d+ \*conventions(:|$)/.test(l))
  check(
    "reserved tag listed with reserved flag",
    conventionsLine !== undefined,
    conventionsLine ?? "(conventions row missing)",
  )

  const purgePreview = await rpc(37, "tools/call", {
    name: "tag_delete",
    arguments: { name: "sdk", mode: "purge", dry_run: true },
  })
  const preview = payloadOf(purgePreview) as { dry_run?: boolean, memories_affected?: number } | undefined
  check(
    "tag_delete dry_run previews without deleting",
    preview?.dry_run === true && (preview?.memories_affected ?? 0) >= 1,
  )
  const stillThere = await rpc(38, "tools/call", { name: "memory_search", arguments: { query: "联调" } })
  check(
    "dry_run deleted nothing",
    lineHeader(textOf(stillThere) ?? "").total_matches !== "0",
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
  // Stale-check note: an omitted content used to be an argument error, but summary-only
  // memories are legal now — trigger the error channel with a missing required arg instead.
  const argError = await rpc(9, "tools/call", {
    name: "memory_create",
    arguments: {},
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
  const noMetaBody = (await noMeta.json()) as { result?: { resultType?: string } }
  check(
    "meta-less request is served as a legacy client",
    noMeta.status === 200 && noMetaBody.result?.resultType === "complete",
    JSON.stringify(noMetaBody).slice(0, 120),
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
  const legacyInit = await rpc(12, "initialize", { protocolVersion: LEGACY_PROTOCOL_VERSION })
  check(
    "modern envelope declaring the legacy initialize is answered with the handshake",
    legacyInit.status === 200 && legacyInit.result?.protocolVersion === LEGACY_PROTOCOL_VERSION,
    JSON.stringify(legacyInit.result ?? legacyInit.error).slice(0, 120),
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
  check("reconnect sees previous data (stateless server)", lineHeader(textOf(again) ?? "").total_matches === "1")

  const pong = await rpc(21, "ping")
  check("ping roundtrip", pong.result?.resultType === "complete")

  // ---- Legacy era (2025-06-18): the way an official SDK 1.x client connects. Read-only calls:
  // the data under test was created by the modern flow above, which also proves both eras share
  // one store.
  const init = await legacyRpc("legacy-1", "initialize", {
    protocolVersion: LEGACY_PROTOCOL_VERSION,
    capabilities: {},
    clientInfo: { name: "legacy-compat-check", version: "0.0.1" },
  })
  const legacyResult = init.result ?? {}
  check(
    "legacy initialize echoes the requested version",
    init.status === 200 && legacyResult.protocolVersion === LEGACY_PROTOCOL_VERSION,
    JSON.stringify(legacyResult).slice(0, 160),
  )
  const legacyCaps = legacyResult.capabilities as Record<string, Record<string, unknown>> | undefined
  check(
    "legacy capabilities promise no notification channel",
    legacyCaps?.resources?.subscribe === false && legacyCaps?.resources?.listChanged === false,
    JSON.stringify(legacyCaps),
  )
  check(
    "legacy initialize carries serverInfo at the top level",
    (legacyResult.serverInfo as Record<string, unknown> | undefined)?.name === "agent-memory",
  )
  check(
    "legacy initialize carries instructions",
    typeof legacyResult.instructions === "string" && (legacyResult.instructions as string).length > 0,
  )
  const legacyInitialized = await fetch(ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }),
  })
  check("legacy notifications/initialized accepted", legacyInitialized.status === 202)

  const legacyTools = await legacyRpc("legacy-2", "tools/list")
  check(
    "legacy tools/list works without any envelope",
    legacyTools.status === 200 &&
      ((legacyTools.result?.tools as { name: string }[] | undefined) ?? []).length === 12,
  )
  const legacySearch = await legacyRpc("legacy-3", "tools/call", {
    name: "memory_search",
    arguments: { query: "联调" },
  })
  check(
    "legacy tools/call reads the same store",
    lineHeader(textOf(legacySearch) ?? "").total_matches === "1",
  )
  const legacyUnknown = await legacyRpc("legacy-4", "bogus/method")
  check(
    "legacy unknown method surfaces as 404 + -32601",
    legacyUnknown.status === 404 && legacyUnknown.error?.code === -32601,
  )
  const legacyPong = await legacyRpc("legacy-5", "ping")
  check("legacy ping roundtrip", legacyPong.status === 200)

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
