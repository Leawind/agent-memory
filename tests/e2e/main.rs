//! End-to-end integration tests: actually spawn the HTTP server binary and CLI subcommands.
//!
//! - the agent's MCP endpoint (POST /mcp, stateless Streamable HTTP mode)
//! - the admin backend /api/* (used by the embedded admin UI)
//! - the statically served admin UI (/)
//! - the full token auth flow and capability boundaries
//! - concurrency correctness with multiple processes sharing one database
//! - CLI ops subcommands (stats / doctor / export / import / token reset)
//! - semantic search: hybrid recall, fallback when the service is down, and the backfill loop
//!
//! The HTTP client is a minimal hand-written implementation on std::net, no dev dependencies (see `common`).

mod auth;
mod cli;
mod common;
mod mcp;
mod rest;
mod semantic;
mod shared;
