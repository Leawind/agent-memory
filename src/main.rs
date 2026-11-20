//! agent-memory: a self-hosted MCP memory server (HTTP) that lets multiple agents share a long-lived memory,
//! with an embedded Vue3 management UI.
//!
//! One process serves everything at once: the agents' MCP Streamable HTTP endpoint (/mcp),
//! the management UI (/) and the admin backend (/api/*). Data lives in a single SQLite file;
//! concurrent multi-process access is guaranteed by WAL + busy_timeout. Diagnostics go to stderr only.
//!
//! The CLI surface stays minimal: the server itself plus a token-reset recovery path. Statistics,
//! health checks and backups are REST endpoints, so they live in the Web UI rather than here.

#![forbid(unsafe_code)]

mod access;
mod adaptive;
mod api;
mod auth;
mod backfill;
mod chunk;
mod embed;
mod http;
mod lifecycle;
mod model;
mod named_predicates;
mod notify;
mod predicate;
mod protocol;
mod rerank;
mod resources;
mod search;
mod search_snapshot;
mod sql;
mod store;
mod tag_expr;
mod tokenizer;
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
        /// duration, caller identity and MCP call summaries
        #[arg(long)]
        verbose: bool,
    },
    /// Manage tokens (the token is the identity; day-to-day management happens in the Web UI)
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
    // The tokenizer disk cache lives next to the database; the server downloads into it while
    // backfilling, so resolve its location once here.
    crate::tokenizer::init_cache_dir(&db_path);
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
