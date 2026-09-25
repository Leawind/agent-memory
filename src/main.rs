//! agent-memory：自托管 MCP 记忆服务器（HTTP），供多个 agent 长期共享记忆，
//! 并内嵌 Vue3 管理界面。
//!
//! 一个进程同时服务：agent 的 MCP Streamable HTTP 端点（/mcp）、
//! 管理界面（/）与管理后端（/api/*）。数据存 SQLite 单文件，
//! 多进程并发由 WAL + busy_timeout 保证。诊断信息只写 stderr。

#![forbid(unsafe_code)]

mod api;
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
    /// 数据库文件路径（默认 ~/.agent-memory/memory.db）
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
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(run(cli));
}

fn run(cli: Cli) -> i32 {
    let db_path = cli.db.unwrap_or_else(store::default_path);
    match cli.command.unwrap_or(Command::Serve {
        host: DEFAULT_HOST.to_string(),
        port: DEFAULT_PORT,
    }) {
        Command::Serve { host, port } => http::serve_http(&host, port, &db_path),
        Command::Stats => cmd_stats(&db_path),
        Command::Doctor => cmd_doctor(&db_path),
        Command::Export { path } => cmd_export(&db_path, &path),
        Command::Import { path } => cmd_import(&db_path, &path),
    }
}

/// 从 export JSON 恢复数据。要求目标库为空（导入是恢复/迁移而非合并）。
fn cmd_import(db_path: &std::path::Path, file: &std::path::Path) -> i32 {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("agent-memory: cannot read {}: {}", file.display(), e);
            return 1;
        }
    };
    let dump: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("agent-memory: {} is not valid JSON: {}", file.display(), e);
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
            eprintln!("agent-memory: import failed: {}", e);
            1
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
        eprintln!(
            "agent-memory: cannot read database at {} ({}).",
            db_path.display(),
            e
        );
        eprintln!("agent-memory: refusing to report on data that cannot be read cleanly.");
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
                    println!("- {}", issue);
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
    let mut body = match serde_json::to_string_pretty(&dump) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("agent-memory: failed to serialize export: {}", e);
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
            eprintln!(
                "agent-memory: {} already exists; choose another name",
                out_path.display()
            );
            return 1;
        }
        Err(e) => {
            eprintln!("agent-memory: cannot create {}: {}", out_path.display(), e);
            return 1;
        }
    };
    if let Err(e) = write_dump(file, &body) {
        eprintln!(
            "agent-memory: failed to write {}: {}",
            out_path.display(),
            e
        );
        return 1;
    }
    println!(
        "exported {} memories, {} tags to {}",
        dump["total_memories"],
        dump["total_tags"],
        out_path.display()
    );
    0
}

fn write_dump(mut file: std::fs::File, body: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    file.write_all(body.as_bytes())
}
