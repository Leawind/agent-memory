#!/usr/bin/env bash
# 构建 release 并部署到 bin/agent-memory.exe（ZCode MCP 配置指向该文件）。
# 每次改完代码后运行 ./build.sh，否则 bin/ 里是旧二进制。
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release

# 兼容全局 CARGO_TARGET_DIR / build.target-dir 配置，从 cargo 元数据取真实输出目录
target_dir="$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' | head -n1)"
[ -n "$target_dir" ] || { echo "cannot locate cargo target directory" >&2; exit 1; }

src="$target_dir/release/agent-memory.exe"
[ -f "$src" ] || { echo "binary not found at $src" >&2; exit 1; }

mkdir -p bin
cp "$src" bin/agent-memory.exe
echo "deployed: bin/agent-memory.exe ($(du -h bin/agent-memory.exe | cut -f1))"
