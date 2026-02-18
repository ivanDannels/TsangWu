#!/usr/bin/env bash
set -euo pipefail

# 苍梧本地模式一键启动脚本
# 用法: ./local-setup.sh [--port 8080] [--data-dir ./data]

PORT=8080
DATA_DIR="./data"

while [[ $# -gt 0 ]]; do
    case $1 in
        --port) PORT="$2"; shift 2 ;;
        --data-dir) DATA_DIR="$2"; shift 2 ;;
        *) echo "未知参数: $1"; exit 1 ;;
    esac
done

echo "=== 苍梧 TsangWu 本地模式启动 ==="
echo "数据目录: $DATA_DIR"
echo "监听端口: $PORT"

# 创建数据目录
mkdir -p "$DATA_DIR/storage"

# 检查二进制是否存在
BIN="./target/release/tsangwu-local"
if [[ ! -f "$BIN" ]]; then
    echo "未找到编译产物，开始编译..."
    cargo build --release --bin tsangwu-local
fi

# 启动
export TSANGWU_DATA_DIR="$DATA_DIR"
export TSANGWU_PORT="$PORT"
export RUST_LOG="${RUST_LOG:-info}"

echo "启动服务: http://127.0.0.1:$PORT"
exec "$BIN"
