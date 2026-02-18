# ---- 构建阶段 ----
FROM rust:1.82-bookworm AS builder

WORKDIR /build

# 先拷贝 workspace 配置和所有 Cargo.toml，利用 Docker 缓存依赖层
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY crates crates
COPY bins bins
COPY proto proto

# 构建 release（使用 --bin 指定目标）
ARG BIN_NAME=tsangwu-server
RUN cargo build --release --bin ${BIN_NAME}

# ---- 运行阶段 ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

ARG BIN_NAME=tsangwu-server
COPY --from=builder /build/target/release/${BIN_NAME} /usr/local/bin/tsangwu

# 配置文件
COPY config /etc/tsangwu/config

ENV TSANGWU_CONFIG_DIR=/etc/tsangwu/config
ENV RUST_LOG=info

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/tsangwu"]
