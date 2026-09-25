#!/usr/bin/env bash
# 构建 release 并部署到 bin/agent-memory.exe（ZCode MCP 配置指向该文件）。
# 每次改完 Rust 代码后运行 ./build.sh，否则 bin/ 里是旧二进制。
# 改了 ui/src 时先在 ui/ 里 npm run build，再跑本脚本重新嵌入。
set -euo pipefail
cd "$(dirname "$0")"

# 嵌入的管理界面产物必须存在（正常随仓库提交）
[ -f ui/dist/index.html ] || { echo "ui/dist/index.html missing; run: npm install && npm run build" >&2; exit 1; }

cargo build --release

# 兼容全局 CARGO_TARGET_DIR / build.target-dir 配置，从 cargo 元数据取真实输出目录
target_dir="$(cargo metadata --format-version 1 --no-deps \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' | head -n1)"
[ -n "$target_dir" ] || { echo "cannot locate cargo target directory" >&2; exit 1; }

# 二进制名随平台不同（Windows 带 .exe）
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*|Windows*) bin_name="agent-memory.exe" ;;
  *) bin_name="agent-memory" ;;
esac
src="$target_dir/release/$bin_name"
[ -f "$src" ] || { echo "binary not found at $src" >&2; exit 1; }

mkdir -p bin
cp "$src" "bin/$bin_name"
echo "deployed: bin/$bin_name ($(du -h "bin/$bin_name" | cut -f1))"
