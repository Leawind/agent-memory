//! agent-memory：自托管 MCP 记忆服务器（HTTP），供多个 agent 长期共享记忆，
//! 并内嵌 Vue3 管理界面。
//!
//! 一个进程同时服务：agent 的 MCP Streamable HTTP 端点（/mcp）、
//! 管理界面（/）与管理后端（/api/*）。数据存 SQLite 单文件，
//! 多进程并发由 WAL + busy_timeout 保证。诊断信息只写 stderr。

#![forbid(unsafe_code)]

mod api;
mod auth;
mod embed;
mod http;
mod model;
mod protocol;
mod search;
mod sql;
mod store;
mod tools;
mod util;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8899;

#[derive(Parser)]
#[command(
    name = "agent-memory",
    version,
    about = "自托管 MCP 记忆服务器：多个 agent 共享一个记忆库，内嵌 Web 管理界面"
)]
struct Cli {
    /// 数据库文件路径（默认当前工作目录下 memory.db）
    #[arg(long, global = true, value_name = "PATH")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// 启动 HTTP 服务器（默认子命令）
    Serve {
        /// 监听地址
        #[arg(long, value_name = "ADDR", default_value = DEFAULT_HOST)]
        host: String,
        /// 监听端口
        #[arg(long, value_name = "PORT", default_value_t = DEFAULT_PORT)]
        port: u16,
        /// 详细日志：记录全部请求（含静态资源/健康检查）并附带耗时、
        /// 请求者身份与 MCP 调用摘要。默认只记错误请求与启动/异常事件。
        #[arg(long)]
        verbose: bool,
    },
    /// 打印数据概况
    Stats,
    /// 数据体检：孤儿引用 / 大小写冲突标签组 / 空字段（发现问题时退出码 1）
    Doctor,
    /// 导出可读的 JSON 备份（文件已存在则拒绝覆盖）
    Export {
        /// 输出文件路径
        path: PathBuf,
    },
    /// 从 export 导出的 JSON 恢复数据（要求目标库为空）
    Import {
        /// 备份文件路径
        path: PathBuf,
    },
    /// 为缺向量的记忆批量补跑 embedding（语义搜索的派生数据重建；
    /// 导出/导入不携带向量，恢复库后跑一次即可）
    EmbedBackfill {
        /// 每批条数
        #[arg(long, default_value_t = embed::MAX_BATCH)]
        batch: usize,
    },
    /// 管理 token（无账号体系：token 即身份，日常增删走管理界面）
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
}

#[derive(Subcommand)]
enum TokenCommand {
    /// 重置身份的 token（旧 token 立即失效），新 token 打印到 stdout。
    /// 省略 NAME 时重置最早创建的管理员身份——用于丢失 token 的兜底恢复。
    Reset {
        /// 身份名（省略 = 最早创建的管理员）
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(run(cli));
}

fn run(cli: Cli) -> i32 {
    let db_path = cli.db.unwrap_or_else(store::default_path);
    // 所有子命令共享默认库路径；cwd 相对名下"跑错目录"会静默新建空库，
    // 一律先在 stderr 报出解析后的实际位置（stdout 保留给命令结果）。
    eprintln!("database: {}", store::normalize_path(&db_path).display());
    match cli.command.unwrap_or(Command::Serve {
        host: DEFAULT_HOST.to_string(),
        port: DEFAULT_PORT,
        verbose: false,
    }) {
        Command::Serve {
            host,
            port,
            verbose,
        } => http::serve_http(&host, port, &db_path, verbose),
        Command::Stats => cmd_stats(&db_path),
        Command::Doctor => cmd_doctor(&db_path),
        Command::Export { path } => cmd_export(&db_path, &path),
        Command::Import { path } => cmd_import(&db_path, &path),
        Command::EmbedBackfill { batch } => cmd_embed_backfill(&db_path, batch),
        Command::Token { command } => match command {
            TokenCommand::Reset { name } => cmd_token_reset(&db_path, name.as_deref()),
        },
    }
}

/// `token reset`：重新生成指定身份的 token（省略名字时选最早创建的管理员）。
/// 结果走 stdout（新 token），诊断走 stderr。
fn cmd_token_reset(db_path: &std::path::Path, name: Option<&str>) -> i32 {
    let result: Result<(String, String), String> =
        store::with_db_in(db_path, store::TxMode::Write, |st| {
            let target = match name {
                Some(n) => n.to_string(),
                None => st
                    .identity_list()?
                    .into_iter()
                    .find(|v| v["permissions"]["admin"] == true)
                    .and_then(|v| v["name"].as_str().map(str::to_string))
                    .ok_or("no admin identity found; create one in the web UI first")?,
            };
            let token = st
                .identity_reset_token(&target)?
                .ok_or_else(|| format!("identity '{target}' not found"))?;
            Ok((target, token))
        });
    match result {
        Ok((target, token)) => {
            println!("new token for '{target}':");
            println!("{token}");
            0
        }
        Err(e) => {
            eprintln!("token reset failed: {e}");
            1
        }
    }
}

/// 从 export JSON 恢复数据。要求目标库为空（导入是恢复/迁移而非合并）。
fn cmd_import(db_path: &std::path::Path, file: &std::path::Path) -> i32 {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {}: {}", file.display(), e);
            return 1;
        }
    };
    let dump: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{} is not valid JSON: {}", file.display(), e);
            return 1;
        }
    };
    match store::with_db_in(db_path, store::TxMode::Write, |st| st.import_dump(&dump)) {
        Ok((memories, tags)) => {
            println!(
                "imported {} memories, {} tags into {}",
                memories,
                tags,
                db_path.display()
            );
            0
        }
        Err(e) => {
            eprintln!("import failed: {e}");
            1
        }
    }
}

/// `embed-backfill`：循环补跑直到清零或失败。结果走 stdout，进度/诊断走 stderr。
fn cmd_embed_backfill(db_path: &std::path::Path, batch: usize) -> i32 {
    let mut total = 0usize;
    loop {
        match embed::process_pending(db_path, batch) {
            embed::EmbedOutcome::NotConfigured => {
                eprintln!("semantic search is not enabled/configured (embedding settings)");
                return 1;
            }
            embed::EmbedOutcome::Failed(e) => {
                eprintln!("embed-backfill failed after {total} memories: {e}");
                return 1;
            }
            embed::EmbedOutcome::Processed {
                processed,
                remaining,
            } => {
                if processed == 0 {
                    println!("done: {total} memories embedded, {remaining} remaining");
                    return 0;
                }
                total += processed;
                eprintln!("embedded {total}, remaining {remaining}");
            }
        }
    }
}

/// 只读子命令统一走只读快照事务：多条查询之间不会被并发写打断，
/// 结果保证同一时刻的一致视图（与 /api 只读端点行为一致）。
fn read_db<T>(
    db_path: &std::path::Path,
    f: impl FnOnce(&store::Store) -> Result<T, String>,
) -> Result<T, i32> {
    store::with_db_in(db_path, store::TxMode::ReadOnly, f).map_err(|e| {
        eprintln!("cannot read database at {} ({}).", db_path.display(), e);
        eprintln!("refusing to report on data that cannot be read cleanly.");
        1
    })
}

fn cmd_stats(db_path: &std::path::Path) -> i32 {
    let s = match read_db(db_path, |st| st.stats()) {
        Ok(s) => s,
        Err(code) => return code,
    };
    println!("path: {}", s["path"].as_str().unwrap_or("?"));
    println!("memories: {}", s["memories"]);
    println!("tags: {}", s["tags"]);
    println!("file size: {} bytes", s["file_size"]);
    println!("next id: {}", s["next_id"].as_str().unwrap_or("?"));
    if let Some(newest) = s["newest_update"].as_object() {
        println!(
            "newest update: {} at {}",
            newest["id"].as_str().unwrap_or("?"),
            newest["updated_at"]
        );
    }
    if let Some(emb) = s["embedding"].as_object() {
        if emb["enabled"].as_bool().unwrap_or(false) {
            println!(
                "embeddings: {} embedded, {} pending (model {})",
                emb["embedded"], emb["pending"], emb["model"]
            );
        } else {
            println!("embeddings: disabled");
        }
    }
    0
}

fn cmd_doctor(db_path: &std::path::Path) -> i32 {
    let result = read_db(db_path, |st| {
        let issues = st.hygiene_issues()?;
        let stats = st.stats()?;
        Ok((issues, stats))
    });
    match result {
        Err(code) => code,
        Ok((issues, stats)) => {
            if issues.is_empty() {
                println!(
                    "no issues found in {} ({} memories, {} tags)",
                    db_path.display(),
                    stats["memories"],
                    stats["tags"],
                );
                0
            } else {
                for issue in &issues {
                    println!("- {issue}");
                }
                println!("{} issue(s) found in {}", issues.len(), db_path.display());
                1
            }
        }
    }
}

fn cmd_export(db_path: &std::path::Path, out_path: &std::path::Path) -> i32 {
    let dump = match read_db(db_path, |st| st.export_dump()) {
        Ok(d) => d,
        Err(code) => return code,
    };
    // 导出一律紧凑 JSON：备份面向程序（import），不面向人读。
    let mut body = match serde_json::to_string(&dump) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to serialize export: {e}");
            return 1;
        }
    };
    body.push('\n');
    // 拒绝覆盖已有文件：备份操作宁可让操作者换个名字，也不默默毁掉旧备份。
    let file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out_path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            eprintln!("{} already exists; choose another name", out_path.display());
            return 1;
        }
        Err(e) => {
            eprintln!("cannot create {}: {}", out_path.display(), e);
            return 1;
        }
    };
    if let Err(e) = write_dump(file, &body) {
        eprintln!("failed to write {}: {}", out_path.display(), e);
        return 1;
    }
    println!(
        "exported {} memories, {} tags to {}",
        dump["memories"].as_object().map(|m| m.len()).unwrap_or(0),
        dump["tags"].as_object().map(|t| t.len()).unwrap_or(0),
        out_path.display()
    );
    0
}

fn write_dump(mut file: std::fs::File, body: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    file.write_all(body.as_bytes())
}
