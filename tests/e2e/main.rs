//! End-to-end integration tests: actually spawn the HTTP server binary and CLI subcommands.
//!
//! - the agent's MCP endpoint (POST /mcp; modern 2026-07-28 and legacy 2025-06-18 client flows)
//! - the admin backend /api/* (used by the embedded admin UI)
//! - the statically served admin UI (/)
//! - the full token auth flow and capability boundaries
//! - concurrency correctness with multiple processes sharing one database
//! - CLI ops subcommands (stats / doctor / export / import / token reset)
//! - semantic search: hybrid recall, fallback when the service is down, and the backfill loop
//!
//! The HTTP client is a minimal hand-written implementation on std::net, no dev dependencies (see `common`).

mod access;
mod auth;
mod cli;
mod common;
mod legacy;
mod lifecycle;
mod mcp;
mod resources;
mod rest;
mod semantic;
mod shared;
mod snapshots;
mod subscribe;
mod tag_rules;
