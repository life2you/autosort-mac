#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  echo "错误：未找到 cargo，请先安装 Rust。"
  exit 1
fi

cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release

echo "release 二进制路径：target/release/autosort-mac"
