// Official TypeScript SDK compatibility check: a real MCP client against the
// agent-memory HTTP server
//
// Usage (start the server first, e.g. agent-memory serve --port 8899):
//   pnpm install (repo root)
//   node sdk-compat-check.ts            # defaults to http://127.0.0.1:8899/mcp
//   MCP_URL=http://127.0.0.1:8899/mcp node sdk-compat-check.ts
// When the server has token auth enabled: MCP_TOKEN=xxx node sdk-compat-check.ts
// Exits 0 when all checks pass; suitable for a manual pre-release regression.
// Test data is cleaned up regardless of the outcome.
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";

const URL_BASE = process.env.MCP_URL || "http://127.0.0.1:8899/mcp";
const TOKEN = process.env.MCP_TOKEN;
const failures: string[] = [];
function check(name: string, cond: boolean, extra = ""): void {
  console.log(`${cond ? "PASS" : "FAIL"}  ${name}${extra ? "  " + extra : ""}`);
  if (!cond) failures.push(name);
}

// Authenticated servers: every transport carries the Bearer header
// (the MCP-standard token carrier)
const transportOptions = TOKEN
  ? { requestInit: { headers: { Authorization: `Bearer ${TOKEN}` } as Record<string, string> } }
  : undefined;

function makeTransport(): StreamableHTTPClientTransport {
  return new StreamableHTTPClientTransport(new URL(URL_BASE), transportOptions);
}

const client = new Client({ name: "sdk-compat-check", version: "0.0.1" });
const transport = makeTransport();
let createdId: string | null = null;
let done = false;

try {
  await client.connect(transport);
  check("connect (initialize handshake)", true);

  const serverInfo = client.getServerVersion();
  check(
    "initialize returns serverInfo",
    serverInfo !== undefined && serverInfo.name === "agent-memory",
    JSON.stringify(serverInfo)
  );

  const tools = await client.listTools();
  check("tools/list returns 10 tools", tools.tools.length === 10, `got ${tools.tools.length}`);

  const created = await client.callTool({
    name: "memory_create",
    arguments: {
      summary: "SDK 联调记忆：官方 TypeScript 客户端可用",
      content: "使用 @modelcontextprotocol/sdk 的 StreamableHTTPClientTransport 连接成功。",
      tags: ["sdk", "联调"],
    },
  });
  check("tools/call memory_create", created.isError !== true, JSON.stringify(created).slice(0, 120));
  const structured = created.structuredContent as
    | { memory?: { id?: string }; total_matches?: number; results?: { content?: unknown }[] }
    | undefined;
  createdId = structured?.memory?.id ?? null;
  check(
    "create returns structured id",
    typeof createdId === "string" && createdId.startsWith("m"),
    createdId ?? ""
  );
  if (createdId === null) throw new Error("create did not return an id");

  const searched = await client.callTool({ name: "memory_search", arguments: { query: "联调" } });
  check("memory_search finds it", structuredOf(searched)?.total_matches === 1);
  check(
    "search does not leak content",
    structuredOf(searched)?.results?.[0]?.content === undefined
  );

  const got = await client.callTool({ name: "memory_get", arguments: { ids: [createdId] } });
  check(
    "memory_get reveals content",
    (structuredOf(got)?.memories?.[0]?.content as string | undefined)?.includes(
      "StreamableHTTPClientTransport"
    ) === true
  );

  const updated = await client.callTool({
    name: "memory_update",
    arguments: { id: createdId, add_tags: ["验证完成"] },
  });
  check(
    "memory_update add_tags",
    structuredOf(updated)?.memory?.tags?.includes("验证完成") === true
  );

  let badIsError: unknown = false;
  try {
    const bad = await client.callTool({
      name: "memory_create",
      arguments: { summary: "only summary" },
    });
    badIsError = (bad as { isError?: unknown }).isError;
  } catch (e) {
    badIsError = " threw: " + (e as Error).message;
  }
  check("tool arg error surfaces as isError", badIsError === true);

  const pong = await client.ping();
  check("ping roundtrip", pong !== undefined);

  // Stateless server: data is still visible after reconnecting (new session)
  const client2 = new Client({ name: "sdk-compat-check-2", version: "0.0.1" });
  await client2.connect(makeTransport());
  const again = await client2.callTool({
    name: "memory_search",
    arguments: { query: "联调" },
  });
  check(
    "reconnect sees previous data (stateless server)",
    structuredOf(again)?.total_matches === 1
  );
  await client2.close();

  done = true;
} finally {
  // Clean up test data and close connections regardless of outcome
  try {
    if (createdId) {
      const cleanup = new Client({ name: "sdk-compat-cleanup", version: "0.0.1" });
      await cleanup.connect(makeTransport());
      await cleanup.callTool({ name: "memory_delete", arguments: { ids: [createdId] } });
      await cleanup.close();
    }
  } catch (e) {
    console.log("cleanup failed (ignored):", (e as Error).message);
  }
  try {
    await client.close();
  } catch {}
}

console.log(
  failures.length === 0 && done
    ? "\nALL CHECKS PASSED"
    : `\n${failures.length} FAILURES: ${failures.join(", ")}`
);
process.exit(failures.length === 0 && done ? 0 : 1);

function structuredOf(r: unknown): {
  total_matches?: number
  results?: { content?: unknown }[]
  memories?: { content?: string; tags?: string[] }[]
  memory?: { tags?: string[] }
} | undefined {
  const out = (r as { structuredContent?: unknown })?.structuredContent;
  return out as never;
}
