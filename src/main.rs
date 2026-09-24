//! agent-memory：自托管、低占用的 MCP 记忆服务器（stdio 传输）。
//!
//! 架构：单线程事件循环读取 stdin 上的换行分隔 JSON-RPC 消息，
//! 处理后将响应写入 stdout。诊断信息只写 stderr（stdout 是协议通道）。
//! 数据持久化到单个 JSON 文件，每次请求在文件锁内"重新加载 → 修改 → 原子写回"，
//! 因此多个客户端进程共享同一数据文件也是安全的。
//!
//! 子命令：默认（无子命令）为 serve，供 MCP 客户端拉起；
//! `stats` / `export <file>` 是只读运维命令，供人工检查与备份。

#![forbid(unsafe_code)]

mod model;
mod protocol;
mod search;
mod store;
mod tools;

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

fn main() {
    std::process::exit(run());
}

enum Command {
    /// 默认：作为 MCP stdio 服务器运行
    Serve,
    /// 打印数据文件概况
    Stats,
    /// 导出可读的 JSON 备份到指定路径（已存在则拒绝覆盖）
    Export(PathBuf),
    /// 数据体检：孤儿标签引用、大小写冲突组、空字段等（只读）
    Doctor,
}

fn run() -> i32 {
    let mut data_path = store::default_path();
    let mut command = Command::Serve;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("agent-memory {}", env!("CARGO_PKG_VERSION"));
                return 0;
            }
            "--help" | "-h" => {
                print_help();
                return 0;
            }
            "--data" => match args.next() {
                Some(p) => data_path = PathBuf::from(p),
                None => {
                    eprintln!("agent-memory: --data requires a path argument");
                    return 2;
                }
            },
            "serve" => command = Command::Serve,
            "stats" => command = Command::Stats,
            "export" => match args.next() {
                Some(p) if !p.starts_with('-') => command = Command::Export(PathBuf::from(p)),
                _ => {
                    eprintln!("agent-memory: export requires an output file path");
                    return 2;
                }
            },
            "doctor" => command = Command::Doctor,
            other => {
                eprintln!("agent-memory: unknown argument '{}'", other);
                print_help();
                return 2;
            }
        }
    }

    match command {
        Command::Serve => serve(&data_path),
        Command::Stats => cmd_stats(&data_path),
        Command::Export(out) => cmd_export(&data_path, &out),
        Command::Doctor => cmd_doctor(&data_path),
    }
}

/// MCP stdio 服务器主循环。返回进程退出码。
fn serve(data_path: &Path) -> i32 {
    // 启动前先验证数据文件可读；读到损坏数据时拒绝启动，避免在坏数据上继续写入。
    if let Err(e) = store::Store::load(data_path.to_path_buf()) {
        eprintln!(
            "agent-memory: cannot load store at {} ({}).",
            data_path.display(),
            e
        );
        eprintln!(
            "agent-memory: refusing to start to protect your data; a backup may exist at {}.",
            store::backup_path(data_path).display()
        );
        return 1;
    }

    eprintln!(
        "agent-memory v{}: MCP stdio server ready; store at {}",
        env!("CARGO_PKG_VERSION"),
        data_path.display()
    );

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("agent-memory: stdin read error: {}", e);
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let msg: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let resp = protocol::error_value(
                    &serde_json::Value::Null,
                    -32700,
                    &format!("parse error: {}", e),
                );
                if write_response(&mut out, &resp).is_err() {
                    break;
                }
                continue;
            }
        };
        // 写失败通常意味着客户端已关闭管道，退出循环结束进程。
        if protocol::handle_message(data_path, &msg, &mut out).is_err() {
            break;
        }
    }
    0
}

fn cmd_stats(data_path: &Path) -> i32 {
    match store::Store::load(data_path.to_path_buf()) {
        Err(e) => {
            eprintln!(
                "agent-memory: cannot load store at {} ({}).",
                data_path.display(),
                e
            );
            1
        }
        Ok(st) => {
            let file_size = std::fs::metadata(data_path).map(|m| m.len()).unwrap_or(0);
            print!("{}", stats_report(&st, file_size));
            0
        }
    }
}

/// 数据体检：0 = 无问题，1 = 发现问题（便于脚本判断）。
fn cmd_doctor(data_path: &Path) -> i32 {
    match store::Store::load(data_path.to_path_buf()) {
        Err(e) => {
            eprintln!(
                "agent-memory: cannot load store at {} ({}).",
                data_path.display(),
                e
            );
            1
        }
        Ok(st) => {
            let issues = st.hygiene_issues();
            if issues.is_empty() {
                println!(
                    "no issues found in {} ({} memories, {} tags)",
                    data_path.display(),
                    st.memories.len(),
                    st.tags.len()
                );
                0
            } else {
                for issue in &issues {
                    println!("- {}", issue);
                }
                println!("{} issue(s) found in {}", issues.len(), data_path.display());
                1
            }
        }
    }
}

fn stats_report(st: &store::Store, file_size: u64) -> String {
    let mut s = String::new();
    s.push_str(&format!("path: {}\n", st.path.display()));
    s.push_str(&format!("memories: {}\n", st.memories.len()));
    s.push_str(&format!("tags: {}\n", st.tags.len()));
    s.push_str(&format!("file size: {} bytes\n", file_size));
    s.push_str(&format!("next id: m{}\n", st.next_id));
    if let Some(newest) = st.memories.iter().max_by_key(|m| m.updated_at) {
        s.push_str(&format!(
            "newest update at: {} (memory {})\n",
            newest.updated_at, newest.id
        ));
    }
    s
}

fn cmd_export(data_path: &Path, out_path: &Path) -> i32 {
    let st = match store::Store::load(data_path.to_path_buf()) {
        Err(e) => {
            eprintln!(
                "agent-memory: cannot load store at {} ({}).",
                data_path.display(),
                e
            );
            return 1;
        }
        Ok(st) => st,
    };
    let dump = export_dump(&st);
    let body = match serde_json::to_string_pretty(&dump) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("agent-memory: failed to serialize export: {}", e);
            return 1;
        }
    };
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
        st.memories.len(),
        st.tags.len(),
        out_path.display()
    );
    0
}

fn write_dump(mut file: std::fs::File, body: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    file.write_all(body.as_bytes())
}

/// 导出内容：完整记忆 + 标签表，独立于存储内部格式（FORMAT_VERSION 变化不影响导出）。
fn export_dump(st: &store::Store) -> serde_json::Value {
    serde_json::json!({
        "exported_at": model::now(),
        "total_memories": st.memories.len(),
        "total_tags": st.tags.len(),
        "tags": st.tags.values().map(|t| serde_json::json!({
            "name": t.name,
            "description": t.description,
            "created_at": t.created_at,
        })).collect::<Vec<_>>(),
        "memories": st.memories.iter().map(|m| m.full_view()).collect::<Vec<_>>(),
    })
}

fn write_response(out: &mut impl Write, v: &serde_json::Value) -> std::io::Result<()> {
    let mut s = serde_json::to_string(v)?;
    s.push('\n');
    out.write_all(s.as_bytes())?;
    out.flush()
}

fn print_help() {
    println!("agent-memory — self-hosted MCP memory server (stdio)");
    println!();
    println!("Usage:");
    println!("  agent-memory [serve] [--data <path>]      run as MCP stdio server (default)");
    println!("  agent-memory stats [--data <path>]        print store summary");
    println!("  agent-memory doctor [--data <path>]       check store hygiene (exit 1 on issues)");
    println!("  agent-memory export <file> [--data <path>]  export readable JSON backup");
    println!("  agent-memory --version");
    println!();
    println!("Options:");
    println!("  --data <path>  data file location (overrides AGENT_MEMORY_PATH)");
    println!();
    println!("Environment:");
    println!(
        "  AGENT_MEMORY_PATH          data file location (default: ~/.agent-memory/memory.json)"
    );
    println!("  AGENT_MEMORY_LOCK_WAIT_MS  store lock wait limit in ms (default 5000, max 60000)");
    println!();
    println!("This binary speaks MCP (JSON-RPC 2.0) over stdio; launch it from an MCP");
    println!("client such as ZCode, Claude Desktop or Cursor.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Memory;

    #[test]
    fn export_dump_contains_full_memories_and_tags() {
        let mut st = store::Store::empty(PathBuf::from("unused.json"));
        let id = st.new_id();
        st.memories.push(Memory {
            id,
            summary: "s".into(),
            content: "body".into(),
            tags: vec!["t".into()],
            created_at: 1,
            updated_at: 2,
        });
        st.tags.insert("t".to_string(), model::Tag::new("t".into()));

        let dump = export_dump(&st);
        assert_eq!(dump["total_memories"], 1);
        assert_eq!(dump["total_tags"], 1);
        assert_eq!(dump["memories"][0]["content"], "body");
        assert_eq!(dump["tags"][0]["name"], "t");
        assert!(dump["exported_at"].is_u64());
    }

    #[test]
    fn stats_report_lists_counts() {
        let mut st = store::Store::empty(PathBuf::from("unused.json"));
        let id = st.new_id();
        st.memories.push(Memory {
            id,
            summary: "s".into(),
            content: "c".into(),
            tags: vec![],
            created_at: 1,
            updated_at: 7,
        });
        let report = stats_report(&st, 123);
        assert!(report.contains("memories: 1"));
        assert!(report.contains("tags: 0"));
        assert!(report.contains("file size: 123 bytes"));
        assert!(report.contains("newest update at: 7"));
    }
}
