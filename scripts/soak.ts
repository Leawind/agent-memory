// 混合负载浸泡脚本：写者 + 读者 + 标签操作并发打两个服务器进程（共享同一库），
// 验证读写事务分离下长时间竞争的稳定性（零丢失、零服务端错误）。
//
// 用法（先起两个共享同一 --db 的服务器；Node ≥24 可直接运行 .ts）：
//   node soak.ts <port1> <port2>      # 默认时长 60 秒
// 失败分类：ECONNREFUSED 风暴 = 客户端并发超出 accept 队列（测试工具过载，
// 非服务缺陷）；HTTP 5xx / SQLITE_BUSY 字样 = 服务端真实问题。
import http from "node:http";

const PORTS = [Number(process.argv[2]), Number(process.argv[3])];
const DURATION_MS = 60_000;
const MAX_SOCKETS = 4;
const ops = { create: 0, search: 0, list: 0, tag: 0 };
const fails = { refused: 0, reset: 0, status4: 0, status5: 0, other: 0 };
const samples: string[] = [];

const agent = new http.Agent({ keepAlive: true, maxSockets: MAX_SOCKETS });

interface HttpResult {
  status?: number;
  body?: string;
  error?: Error;
}

function req(method: string, port: number, path: string, body?: unknown): Promise<HttpResult> {
  return new Promise((resolve) => {
    const data = body !== undefined ? JSON.stringify(body) : null;
    const r = http.request(
      {
        host: "127.0.0.1",
        port,
        method,
        path,
        agent,
        headers: data
          ? { "Content-Type": "application/json", "Content-Length": Buffer.byteLength(data) }
          : {},
      },
      (res) => {
        const chunks: Buffer[] = [];
        res.on("data", (x: Buffer) => chunks.push(x));
        res.on("end", () => resolve({ status: res.statusCode, body: Buffer.concat(chunks).toString() }));
      }
    );
    r.on("error", (e: Error) => resolve({ error: e }));
    if (data) r.write(data);
    r.end();
  });
}

function recordFail(label: string, r: HttpResult): void {
  if (r.error) {
    const code = (r.error as NodeJS.ErrnoException).code ?? "unknown";
    if (code === "ECONNREFUSED") fails.refused++;
    else if (code === "ECONNRESET" || code === "EPIPE") fails.reset++;
    else fails.other++;
    if (samples.length < 5) samples.push(`${label}: ${code}`);
  } else if ((r.status ?? 0) >= 500) {
    fails.status5++;
    if (samples.length < 5) samples.push(`${label}: HTTP ${r.status} ${r.body?.slice(0, 80)}`);
  } else {
    fails.status4++;
    if (samples.length < 5) samples.push(`${label}: HTTP ${r.status} ${r.body?.slice(0, 80)}`);
  }
}

async function main(): Promise<void> {
  const deadline = Date.now() + DURATION_MS;
  const work: Promise<void>[] = [];

  for (const w of [0, 1]) {
    work.push((async () => {
      while (Date.now() < deadline) {
        const r = await req("POST", PORTS[w % 2], "/api/memories", {
          summary: `soak ${w} ${Date.now()}`,
          content: "soak body",
          tags: ["soak"],
        });
        if (r.status === 200) ops.create++;
        else recordFail("create", r);
      }
    })());
  }
  for (const w of [0, 1]) {
    work.push((async () => {
      while (Date.now() < deadline) {
        const r = await req("GET", PORTS[w % 2], "/api/memories?query=soak&limit=5");
        if (r.status === 200) ops.search++;
        else recordFail("search", r);
      }
    })());
  }
  work.push((async () => {
    let i = 0;
    while (Date.now() < deadline) {
      const name = `soaktag${i % 5}`;
      await req("POST", PORTS[i % 2], "/api/tags", { name, description: "soak" });
      await req("PUT", PORTS[i % 2], `/api/tags/${name}`, { description: `v${i}` });
      await req("DELETE", PORTS[i % 2], `/api/tags/${name}?mode=detach`);
      ops.tag++;
      i++;
    }
  })());
  work.push((async () => {
    while (Date.now() < deadline) {
      const r = await req("GET", PORTS[0], "/api/memories?limit=20");
      if (r.status === 200) ops.list++;
      else recordFail("list", r);
    }
  })());

  await Promise.all(work);
  console.log("ops:", JSON.stringify(ops));
  console.log("fails:", JSON.stringify(fails));
  console.log("error samples:");
  for (const s of samples) console.log("  -", s);
}

void main();
