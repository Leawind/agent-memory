//! 端到端集成测试：真实启动 HTTP 服务器二进制与 CLI 子命令。
//!
//! - agent 的 MCP 端点（POST /mcp，Streamable HTTP 无状态模式）
//! - 管理后端 /api/*（供内嵌管理界面使用）
//! - 静态托管的管理界面（/）
//! - token 鉴权全流程与能力边界
//! - 多进程共享同一数据库的并发正确性
//! - CLI 运维子命令（stats / doctor / export / import / token reset）
//! - 语义搜索：hybrid 召回、服务不可用的回退与补跑闭环
//!
//! HTTP 客户端用 std::net 手写最小实现，不引入 dev 依赖（见 `common`）。

mod auth;
mod cli;
mod common;
mod mcp;
mod rest;
mod semantic;
mod shared;
