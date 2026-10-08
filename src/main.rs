//! agent-memory: a self-hosted MCP memory server (HTTP) that lets multiple agents share a long-lived memory,
//! with an embedded Vue3 management UI.
//!
//! One process serves everything at once: the agents' MCP Streamable HTTP endpoint (/mcp),
//! the management UI (/) and the admin backend (/api/*). Data lives in a single SQLite file;
//! concurrent multi-process access is guaranteed by WAL + busy_timeout. Diagnostics go to stderr only.

#![forbid(unsafe_code)]

mod access;
mod adaptive;
mod api;
mod auth;
mod embed;
mod http;
mod lifecycle;
mod model;
mod notify;
mod protocol;
mod rerank;
mod resources;
mod search;
mod sql;
mod store;
mod tag_expr;
mod tag_rules;
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
    about = "Self-hosted MCP memory server: multiple agents sharing one memory store, with an embedded Web management UI"
)]
struct Cli {
    /// Database file path (defaults to memory.db in the current working directory)
    #[arg(long, global = true, value_name = "PATH")]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the HTTP server (default subcommand)
    Serve {
        /// Listen address
        #[arg(long, value_name = "ADDR", default_value = DEFAULT_HOST)]
        host: String,
        /// Listen port
        #[arg(long, value_name = "PORT", default_value_t = DEFAULT_PORT)]
        port: u16,
        /// Verbose logging: record every request (including static assets/health checks) with
        /// duration, caller identity and MCP call summaries. By default only failed requests and startup/error events are logged.
        #[arg(long)]
        verbose: bool,
    },
    /// Print data statistics
    Stats,
    /// Data health check: orphan references / case-conflicting tag groups / empty fields (exits with code 1 when issues are found)
    Doctor,
    /// Export a human-readable JSON backup (refuses to overwrite an existing file)
    Export {
        /// Output file path
        path: PathBuf,
    },
    /// Restore data from a JSON backup produced by export (the target database must be empty)
    Import {
        /// Backup file path
        path: PathBuf,
    },
    /// Batch backfill embeddings for memories missing vectors (rebuilds the derived data
    /// behind semantic search; export/import does not carry vectors, so run this once after restoring a database).
    /// --model targets one cache identity (see `agent-memory stats` or the admin UI); without it
    /// the highest-priority available candidate's cache is drained.
    EmbedBackfill {
        /// Number of items per batch
        #[arg(long, default_value_t = embed::MAX_BATCH)]
        batch: usize,
        /// Cache identity key to backfill (defaults to the active candidate's)
        #[arg(long)]
        model: Option<String>,
    },
    /// Manage tokens (no account system: the token is the identity; day-to-day management happens in the Web UI)
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
}

#[derive(Subcommand)]
enum TokenCommand {
    /// Reset an identity's token (the old one is invalidated immediately); the new token is printed to stdout.
    /// When NAME is omitted, the earliest-created admin identity is reset — a fallback for lost tokens.
    Reset {
        /// Identity name (omit = the earliest-created admin)
        name: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(run(cli));
}

fn run(cli: Cli) -> i32 {
    let db_path = cli.db.unwrap_or_else(store::default_path);
    // All subcommands share the default database path; with a cwd-relative name, running from the
    // wrong directory would silently create an empty database — always report the resolved location on stderr first (stdout is reserved for command results).
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
        Command::EmbedBackfill { batch, model } => {
            cmd_embed_backfill(&db_path, batch, model.as_deref())
        }
        Command::Token { command } => match command {
            TokenCommand::Reset { name } => cmd_token_reset(&db_path, name.as_deref()),
        },
    }
}

/// `token reset`: regenerate the token for the given identity (earliest-created admin when the name is omitted).
/// Results go to stdout (the new token), diagnostics to stderr.
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

/// Restore data from an export JSON. The target database must be empty (import is a restore/migration, not a merge).
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

/// `embed-backfill`: loop until the pending queue drains or a failure occurs. Results go to stdout, progress/diagnostics to stderr.
/// `model` selects a specific cache identity; without it the active candidate's cache is drained.
fn cmd_embed_backfill(db_path: &std::path::Path, batch: usize, model: Option<&str>) -> i32 {
    // Explicit identity: resolve it to a configured candidate first (a cache that matches no
    // enabled entry can only be deleted, not backfilled)
    let explicit = match model {
        Some(key) => match read_db(db_path, |st| {
            Ok(st
                .embedding_entries()?
                .into_iter()
                .find(|e| e.vector_key() == key)
                .and_then(|e| e.usable()))
        }) {
            Ok(Some(cfg)) => Some(cfg),
            Ok(None) => {
                eprintln!("no enabled embedding entry matches cache key '{key}'");
                return 1;
            }
            Err(code) => return code,
        },
        None => None,
    };
    let mut total = 0usize;
    loop {
        let outcome = match &explicit {
            Some(cfg) => embed::process_pending_for(db_path, batch, cfg)
                .unwrap_or_else(embed::EmbedOutcome::Failed),
            None => embed::process_pending(db_path, batch),
        };
        match outcome {
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

/// Read-only subcommands all use read-only snapshot transactions: concurrent writes cannot
/// interleave between queries, so results reflect one consistent moment (same behavior as the /api read-only endpoints).
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
            for m in emb["models"].as_array().unwrap_or(&Vec::new()) {
                println!(
                    "  - {} [key {}]: {} embedded, {} pending{}",
                    m["model"].as_str().unwrap_or("?"),
                    m["key"].as_str().unwrap_or("?"),
                    m["embedded"].as_u64().unwrap_or(0),
                    m["pending"].as_u64().unwrap_or(0),
                    if m["enabled"].as_bool().unwrap_or(false) {
                        ""
                    } else {
                        ", disabled"
                    }
                );
            }
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
    // Export is always compact JSON: the backup targets programs (import), not human readers.
    let mut body = match serde_json::to_string(&dump) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("failed to serialize export: {e}");
            return 1;
        }
    };
    body.push('\n');
    // Refuse to overwrite existing files: for backups, it is better to ask the operator for another name than to silently destroy the old backup.
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
