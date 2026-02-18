# 苍梧（TsangWu）系统 Rust 技术方案

> 版本：v1.0 | 更新日期：2026-02-17 | 状态：详细设计阶段
> 基于：苍梧（TsangWu）产品需求文档 v2.0 & Go 版详细设计方案 v1.0
> 技术栈：Rust 生态

---

## 目录

1. [方案概述与设计原则](#1-方案概述与设计原则)
2. [Rust 技术选型详解](#2-rust-技术选型详解)
3. [项目工程结构（Workspace + Crate 拆分）](#3-项目工程结构workspace--crate-拆分)
4. [多部署模式架构设计（本地/单机/分布式）](#4-多部署模式架构设计本地单机分布式)
5. [核心框架与公共库设计](#5-核心框架与公共库设计)
6. [微服务/模块详细设计](#6-微服务模块详细设计)
7. [数据库与存储层（SQLx/SeaORM + OpenDAL）](#7-数据库与存储层sqlxseaorm--opendal)
8. [API 层设计（Axum + Tonic）](#8-api-层设计axum--tonic)
9. [工作流引擎设计（自研状态机方案）](#9-工作流引擎设计自研状态机方案)
10. [AI Agent 系统 Rust 实现](#10-ai-agent-系统-rust-实现)
11. [生成管线 Rust 实现（FFI + PyO3）](#11-生成管线-rust-实现ffi--pyo3)
12. [文化适配引擎 Rust 实现](#12-文化适配引擎-rust-实现)
13. [部署与运维（Docker/K8s/本地一键启动）](#13-部署与运维dockerk8s本地一键启动)
14. [安全与性能](#14-安全与性能)
15. [监控与可观测性（OpenTelemetry Rust SDK）](#15-监控与可观测性opentelemetry-rust-sdk)

---

## 1. 方案概述与设计原则

### 1.1 方案背景

苍梧系统旨在构建一个以中国传统文化为内核的多模态全链路影像生成平台。本方案在原 Go 技术栈详细设计基础上，全面切换至 Rust 技术生态，并新增对本地部署、单机部署、分布式部署等多种部署模式的支持。

### 1.2 设计目标

| 目标 | 描述 |
|------|------|
| 文化精准 | 国风元素生成准确率 ≥ 90%，覆盖服饰形制、建筑结构、传统色彩等维度 |
| 全链路贯通 | 打通图文→短剧→电影→电视剧四大场景，素材跨场景复用 |
| 分层适配 | 零基础用户一键生成、专业用户全参数可控，渐进式功能暴露 |
| 高可用 | 系统可用性 ≥ 99.9%，支持 1000+ 并发用户 |
| 弹性伸缩 | 算力按需分配，支持突发流量弹性扩容 |
| 安全合规 | 符合国家数据安全法规，内置版权校验与内容审核 |
| 多模式部署 | 支持本地开发/演示、单机生产、分布式集群三种部署形态 |
| 极致性能 | 利用 Rust 零成本抽象与内存安全，实现低延迟高吞吐 |

### 1.3 设计原则

| 原则 | 说明 |
|------|------|
| 模块化单体优先 | 采用 Rust Workspace 单仓多 crate，通过 feature flag 控制编译产物形态，避免过早微服务化 |
| 零成本抽象 | 充分利用 Rust 泛型、trait、编译期单态化，运行时无额外开销 |
| 编译期保障 | 利用类型系统、生命周期、SQLx 编译期 SQL 校验，将错误前移到编译阶段 |
| 异步优先 | 全链路基于 Tokio 异步运行时，避免阻塞线程 |
| 存储抽象 | 通过 trait 抽象存储后端，同一代码适配 SQLite/PostgreSQL、本地文件/OSS |
| 渐进式复杂度 | 本地模式开箱即用，生产模式按需引入外部中间件 |
| 安全内建 | Rust 内存安全 + 应用层安全策略，双重保障 |

### 1.4 设计约束

| 约束类型 | 具体约束 |
|----------|----------|
| 技术约束 | 依赖第三方大模型（豆包/DeepSeek/SD/Gemini），需适配多模型接口差异 |
| 性能约束 | 图文生成 ≤ 10s（零基础）/ ≤ 30s（专业）；短剧 1min 内容生成 ≤ 5min |
| 合规约束 | 符合《数据安全法》《个人信息保护法》《生成式AI管理办法》 |
| 成本约束 | GPU 算力成本需通过差异化调度与缓存机制控制在可盈利范围 |
| 分期约束 | Phase 1-2 优先图文+短剧，电影/电视剧模块延后 |
| Rust 生态约束 | Temporal 无官方 Rust SDK，需自研轻量工作流引擎替代 |

### 1.5 设计范围

本方案覆盖苍梧系统全部技术层面，包括：
- Rust 技术选型与工程结构
- 多部署模式架构（本地/单机/分布式）
- 核心框架与公共库
- 微服务/模块详细设计
- 数据库模型与存储抽象层
- API 层（REST + gRPC）
- 自研工作流引擎
- AI Agent 编排与生成管线
- Python AI 模型桥接（PyO3/FFI）
- 文化适配引擎
- 部署运维、安全性能、监控可观测性

---

## 2. Rust 技术选型详解

### 2.1 技术选型总览

| 领域 | 原方案 (Go) | Rust 方案 | 选型理由 |
|------|------------|-----------|----------|
| HTTP 框架 | Gin | **Axum** | Tokio 生态核心，Tower 中间件复用，类型安全路由，提取器模式 |
| gRPC | grpc-go | **Tonic** | 纯 Rust 实现，与 Tokio 深度集成，Protobuf codegen |
| 异步运行时 | goroutine | **Tokio** | 高性能异步运行时，零成本抽象，work-stealing 调度器 |
| ORM/数据库 | GORM | **SeaORM + SQLx** | SeaORM 异步 ORM 适合 CRUD；SQLx 编译期 SQL 校验适合复杂查询 |
| 消息队列 | sarama | **rdkafka** | librdkafka 绑定，生产级可靠，高吞吐 |
| Redis | go-redis | **fred** | 全异步，Cluster/Sentinel 支持，连接池管理 |
| MongoDB | mongo-go-driver | **mongodb (官方驱动)** | 官方异步驱动，BSON 原生支持 |
| 序列化 | encoding/json | **serde + serde_json** | 零拷贝反序列化，编译期校验，生态统一 |
| 工作流引擎 | Temporal Go SDK | **自研 (状态机 + Kafka)** | Temporal 无官方 Rust SDK，自研更可控，支持本地模式降级 |
| API 网关 | APISIX | **Pingora / 自研基于 Axum** | Cloudflare 开源 Rust 代理框架，高性能低内存 |
| 对象存储 | aliyun-oss-go | **OpenDAL** | Apache 孵化项目，统一存储抽象层，支持 OSS/S3/MinIO/本地文件系统 |
| 搜索引擎客户端 | olivere/elastic | **elasticsearch-rs** | 官方 Rust 客户端，异步支持 |
| 向量数据库 | milvus-sdk-go | **Milvus gRPC (Tonic)** | 通过 Tonic 直接调用 Milvus gRPC API |
| 日志 | zap | **tracing + tracing-subscriber** | 结构化日志 + 分布式追踪统一框架 |
| 配置管理 | viper | **config-rs** | 多源配置合并，支持文件/环境变量/远程配置 |
| 错误处理 | pkg/errors | **thiserror + anyhow** | thiserror 定义业务错误；anyhow 处理临时错误 |
| HTTP 客户端 | net/http | **reqwest** | 基于 Tokio，连接池，自动重试 |
| 密码哈希 | bcrypt | **argon2** | Argon2id 算法，抗 GPU/ASIC 攻击，OWASP 推荐 |
| JWT | golang-jwt | **jsonwebtoken** | 成熟的 JWT 编解码库 |
| 定时任务 | robfig/cron | **tokio-cron-scheduler** | 基于 Tokio 的 cron 调度 |
| Python 桥接 | — | **PyO3 + pyo3-asyncio** | Rust ↔ Python 双向调用，AI 模型集成 |
| FFmpeg 绑定 | — | **ffmpeg-next (rsmpeg)** | FFmpeg C API 的安全 Rust 绑定 |

### 2.2 核心依赖版本矩阵

```toml
# Cargo.toml workspace dependencies
[workspace.dependencies]
# 异步运行时
tokio = { version = "1.43", features = ["full"] }
# HTTP 框架
axum = { version = "0.8", features = ["macros", "ws"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip", "limit"] }
# gRPC
tonic = "0.12"
prost = "0.13"
# 数据库
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "sqlite", "uuid", "chrono", "json"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "sqlx-sqlite", "runtime-tokio-rustls"] }
# Redis
fred = { version = "9", features = ["subscriber-client", "enable-rustls"] }
# MongoDB
mongodb = { version = "3", features = ["tokio-runtime"] }
# Kafka
rdkafka = { version = "0.37", features = ["tokio"] }
# 对象存储
opendal = { version = "0.51", features = ["services-s3", "services-fs", "services-memory"] }
# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# 日志与追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.28"
opentelemetry = "0.27"
opentelemetry-otlp = "0.27"
# 错误处理
thiserror = "2"
anyhow = "1"
# HTTP 客户端
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
# 认证
jsonwebtoken = "9"
argon2 = "0.5"
# 配置
config = "0.14"
# Python 桥接
pyo3 = { version = "0.23", features = ["auto-initialize"] }
# 工具
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### 2.3 选型深度分析

#### Axum vs Actix-web

| 维度 | Axum | Actix-web |
|------|------|-----------|
| 生态整合 | Tower 中间件生态，与 Tonic/Hyper 共享 | 自有中间件体系 |
| 类型安全 | 提取器模式，编译期路由校验 | 宏驱动，运行时错误 |
| 学习曲线 | 中等，符合 Rust 惯用模式 | 较低，类 Express 风格 |
| 性能 | 极高（Hyper 底层） | 极高（自有运行时） |
| 维护状态 | Tokio 团队维护，活跃 | 社区维护，活跃 |

**选择 Axum**：与 Tonic (gRPC) 共享 Tower 中间件层，认证、限流、追踪等中间件一次编写，HTTP/gRPC 双协议复用。

#### SeaORM + SQLx 组合策略

- **SeaORM**：用于标准 CRUD 操作（用户、项目、订阅等），提供 ActiveModel 模式，减少样板代码
- **SQLx**：用于复杂查询（报表统计、批量操作、窗口函数），编译期 SQL 校验，零运行时开销
- 两者共享同一连接池（`sqlx::PgPool`），无额外资源消耗

#### OpenDAL 统一存储抽象

OpenDAL 是 Apache 孵化项目，提供统一的存储访问接口：

```rust
use opendal::Operator;

// 本地模式：文件系统后端
let op = Operator::new(opendal::services::Fs::default().root("/data/tsangwu"))?;

// 生产模式：阿里云 OSS 后端
let op = Operator::new(opendal::services::S3::default()
    .endpoint("https://oss-cn-shanghai.aliyuncs.com")
    .bucket("tsangwu-assets")
    .access_key_id(&key)
    .secret_access_key(&secret))?;

// 统一 API，业务代码无需感知后端差异
op.write("assets/thumb/xxx.jpg", bytes).await?;
let data = op.read("assets/thumb/xxx.jpg").await?;
```

---

## 3. 项目工程结构（Workspace + Crate 拆分）

### 3.1 Workspace 总体布局

```
tsangwu/
├── Cargo.toml                    # Workspace 根配置
├── Cargo.lock
├── .cargo/
│   └── config.toml               # 编译配置（链接器、目标平台）
├── proto/                        # Protobuf 定义（gRPC 接口）
│   ├── user.proto
│   ├── auth.proto
│   ├── project.proto
│   ├── asset.proto
│   ├── generation.proto
│   ├── agent.proto
│   ├── culture.proto
│   └── common.proto
├── crates/
│   ├── tsangwu-common/           # 公共类型、错误定义、工具函数
│   ├── tsangwu-config/           # 配置管理（多环境、多部署模式）
│   ├── tsangwu-db/               # 数据库抽象层（SQLx/SeaORM/SQLite）
│   ├── tsangwu-storage/          # 存储抽象层（OpenDAL 封装）
│   ├── tsangwu-auth/             # 认证与权限（JWT/RBAC）
│   ├── tsangwu-mq/               # 消息队列抽象（Kafka/内存队列）
│   ├── tsangwu-cache/            # 缓存抽象（Redis/内存缓存）
│   ├── tsangwu-workflow/         # 自研工作流引擎
│   ├── tsangwu-api-proto/        # Protobuf 生成代码（tonic-build）
│   ├── tsangwu-user-svc/         # 用户服务
│   ├── tsangwu-auth-svc/         # 认证服务
│   ├── tsangwu-project-svc/      # 项目服务
│   ├── tsangwu-asset-svc/        # 素材服务
│   ├── tsangwu-template-svc/     # 模板服务
│   ├── tsangwu-gen-svc/          # 生成任务服务
│   ├── tsangwu-agent-svc/        # Agent 编排服务
│   ├── tsangwu-culture-svc/      # 文化适配服务
│   ├── tsangwu-copyright-svc/    # 版权服务
│   ├── tsangwu-audit-svc/        # 审核服务
│   ├── tsangwu-dist-svc/         # 分发服务
│   ├── tsangwu-pay-svc/          # 支付服务
│   ├── tsangwu-notif-svc/        # 通知服务
│   ├── tsangwu-gateway/          # API 网关（基于 Pingora/Axum）
│   └── tsangwu-pybridge/         # Python AI 模型桥接（PyO3）
├── bins/
│   ├── tsangwu-server/           # 单体模式入口（编译所有 crate 为一个二进制）
│   ├── tsangwu-local/            # 本地模式入口（嵌入 SQLite + 本地文件系统）
│   └── tsangwu-migrate/          # 数据库迁移工具
├── deploy/
│   ├── docker/
│   │   ├── Dockerfile.server     # 单体/单机模式镜像
│   │   ├── Dockerfile.svc        # 微服务模式镜像（多阶段构建）
│   │   └── docker-compose.yml    # 本地开发环境编排
│   ├── k8s/                      # Kubernetes 部署清单
│   │   ├── base/
│   │   ├── overlays/
│   │   │   ├── dev/
│   │   │   ├── staging/
│   │   │   └── prod/
│   │   └── kustomization.yaml
│   └── scripts/
│       ├── local-setup.sh        # 本地一键启动脚本
│       └── migrate.sh            # 数据库迁移脚本
├── tests/
│   ├── integration/              # 集成测试
│   └── e2e/                      # 端到端测试
└── docs/
    └── api/                      # API 文档
```

### 3.2 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",
    "bins/*",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "proprietary"
repository = "https://git.tsangwu.com/tsangwu/tsangwu"

[workspace.dependencies]
# ... 见 2.2 节版本矩阵

[workspace.features]
# 部署模式 feature flags
local-mode = []       # 本地模式：SQLite + 本地文件 + 内存队列
standalone-mode = []  # 单机模式：PostgreSQL + Redis + 本地文件/OSS
distributed-mode = [] # 分布式模式：全外部中间件
```

### 3.3 Crate 依赖关系

```
tsangwu-common ◄─── 所有 crate 依赖
     │
     ├── tsangwu-config ◄─── 所有服务 crate
     ├── tsangwu-db ◄─── 所有业务服务
     ├── tsangwu-storage ◄─── asset-svc, gen-svc, dist-svc
     ├── tsangwu-auth ◄─── 所有业务服务（鉴权中间件）
     ├── tsangwu-mq ◄─── gen-svc, agent-svc, audit-svc, notif-svc
     ├── tsangwu-cache ◄─── 所有业务服务
     └── tsangwu-workflow ◄─── agent-svc, gen-svc

tsangwu-api-proto ◄─── 所有服务（gRPC 接口定义）

tsangwu-pybridge ◄─── agent-svc, gen-svc, culture-svc（AI 模型调用）
```

### 3.4 Feature Flag 控制编译

```toml
# crates/tsangwu-db/Cargo.toml
[features]
default = ["postgres"]
postgres = ["sqlx/postgres", "sea-orm/sqlx-postgres"]
sqlite = ["sqlx/sqlite", "sea-orm/sqlx-sqlite"]

# crates/tsangwu-mq/Cargo.toml
[features]
default = ["kafka"]
kafka = ["rdkafka"]
in-memory = []  # 本地模式使用 tokio::sync::mpsc 替代 Kafka

# crates/tsangwu-cache/Cargo.toml
[features]
default = ["redis"]
redis = ["fred"]
in-memory = ["moka"]  # 本地模式使用 moka 内存缓存

# crates/tsangwu-storage/Cargo.toml
[features]
default = ["s3"]
s3 = ["opendal/services-s3"]
local-fs = ["opendal/services-fs"]
```

---

## 4. 多部署模式架构设计（本地/单机/分布式）

### 4.1 三种部署模式总览

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        苍梧系统部署模式                                   │
├───────────────────┬───────────────────┬─────────────────────────────────┤
│   本地模式         │   单机模式         │   分布式模式                     │
│   (Local)         │   (Standalone)    │   (Distributed)                │
├───────────────────┼───────────────────┼─────────────────────────────────┤
│ 单一二进制         │ 单一二进制         │ 多个独立二进制                    │
│ 嵌入 SQLite       │ 外部 PostgreSQL   │ 外部 PostgreSQL (主从)           │
│ 本地文件系统       │ 外部 Redis        │ Redis Cluster                   │
│ 内存消息队列       │ 可选 Kafka        │ Kafka Cluster                   │
│ 内存缓存 (moka)   │ 本地/OSS 存储     │ 阿里云 OSS                      │
│ 无需外部依赖       │ 可选 ES/Milvus    │ ES + Milvus + MongoDB           │
│                   │                   │ K8s 编排                        │
├───────────────────┼───────────────────┼─────────────────────────────────┤
│ 适用：开发/演示    │ 适用：中小规模生产  │ 适用：大规模生产                  │
│ 用户：1-5         │ 用户：5-200       │ 用户：200-10000+                │
└───────────────────┴───────────────────┴─────────────────────────────────┘
```

### 4.2 本地模式详细设计

本地模式目标：下载单个二进制文件，无需安装任何外部依赖，即可启动完整系统。

#### 架构图

```
┌──────────────────────────────────────────────┐
│           tsangwu-local (单一二进制)            │
│                                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐      │
│  │ Axum    │  │ 业务模块 │  │ Agent   │      │
│  │ HTTP    │  │ (全部)   │  │ 编排    │      │
│  │ Server  │  │         │  │         │      │
│  └────┬────┘  └────┬────┘  └────┬────┘      │
│       │            │            │             │
│  ┌────▼────────────▼────────────▼────┐       │
│  │         内部路由层 (直接函数调用)     │       │
│  └────┬─────────┬──────────┬─────────┘       │
│       │         │          │                  │
│  ┌────▼───┐ ┌──▼────┐ ┌──▼──────┐           │
│  │ SQLite │ │ moka  │ │ 本地FS  │           │
│  │        │ │ 缓存   │ │ 存储    │           │
│  └────────┘ └───────┘ └─────────┘           │
│                                               │
│  ┌──────────────────────────────────┐        │
│  │  内存消息队列 (tokio::mpsc)       │        │
│  └──────────────────────────────────┘        │
│  ┌──────────────────────────────────┐        │
│  │  内嵌工作流引擎 (状态机 + SQLite)  │        │
│  └──────────────────────────────────┘        │
└──────────────────────────────────────────────┘
```

#### 关键实现

```rust
// bins/tsangwu-local/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化本地模式配置
    let config = TsangwuConfig::load_local()?;

    // SQLite 数据库（自动创建 + 迁移）
    let db = tsangwu_db::init_sqlite(&config.data_dir).await?;

    // 本地文件存储
    let storage = tsangwu_storage::init_local_fs(&config.data_dir)?;

    // 内存缓存
    let cache = tsangwu_cache::init_memory();

    // 内存消息队列
    let mq = tsangwu_mq::init_memory();

    // 内嵌工作流引擎
    let workflow = tsangwu_workflow::init_embedded(db.clone()).await?;

    // 构建应用状态（所有服务共享）
    let state = AppState::new(db, storage, cache, mq, workflow);

    // 构建统一路由（所有服务路由合并）
    let app = Router::new()
        .merge(tsangwu_user_svc::routes())
        .merge(tsangwu_auth_svc::routes())
        .merge(tsangwu_project_svc::routes())
        .merge(tsangwu_asset_svc::routes())
        .merge(tsangwu_gen_svc::routes())
        .merge(tsangwu_agent_svc::routes())
        .merge(tsangwu_culture_svc::routes())
        // ... 其他服务路由
        .with_state(state);

    // 启动 HTTP 服务
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!("苍梧本地模式已启动: http://127.0.0.1:8080");
    axum::serve(listener, app).await?;
    Ok(())
}
```

#### 本地模式限制

| 功能 | 本地模式行为 |
|------|-------------|
| 全文检索 | 降级为 SQLite FTS5 全文索引 |
| 向量检索 | 降级为内存暴力搜索（小规模数据集） |
| 视频生成 | 需本地 GPU + Python 环境（PyO3 桥接） |
| 多用户 | 支持但不推荐，无水平扩展能力 |
| WebSocket | 支持，单进程内直接推送 |
| 定时任务 | tokio-cron-scheduler 内嵌调度 |

### 4.3 单机模式详细设计

单机模式目标：单一二进制 + 外部 PostgreSQL/Redis，适合中小规模生产环境。

#### 架构图

```
┌──────────────────────────────────────────────────────┐
│              tsangwu-server (单一二进制)                │
│                                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │ Axum HTTP│  │ Tonic    │  │ WS 推送  │            │
│  │ :8080    │  │ gRPC     │  │ :8081    │            │
│  │          │  │ :9090    │  │          │            │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘            │
│       └──────────────┼────────────┘                   │
│                      │                                │
│  ┌───────────────────▼───────────────────┐            │
│  │          业务模块层（全部服务内嵌）       │            │
│  └───────────────────┬───────────────────┘            │
│                      │                                │
└──────────────────────┼────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
    ┌─────────┐  ┌─────────┐  ┌─────────┐
    │PostgreSQL│  │  Redis  │  │ OSS/本地 │
    │         │  │         │  │ 文件存储  │
    └─────────┘  └─────────┘  └─────────┘
         │
    ┌────┴────┐
    │ 可选     │
    ├─────────┤
    │ Kafka   │  ← 有则用 Kafka，无则内存队列
    │ ES      │  ← 有则用 ES，无则 PG 全文索引
    │ MongoDB │  ← 有则用 MongoDB，无则 PG JSONB
    └─────────┘
```

#### 中间件降级策略

```rust
// crates/tsangwu-config/src/lib.rs
#[derive(Debug, Deserialize)]
pub struct TsangwuConfig {
    pub deploy_mode: DeployMode,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub message_queue: MqConfig,
    pub search: SearchConfig,
    // ...
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployMode {
    Local,
    Standalone,
    Distributed,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SearchConfig {
    #[serde(rename = "elasticsearch")]
    Elasticsearch { url: String },
    #[serde(rename = "postgres")]
    PostgresFts,           // 降级：使用 PG tsvector
    #[serde(rename = "sqlite")]
    SqliteFts,             // 本地模式：SQLite FTS5
}
```

### 4.4 分布式模式详细设计

分布式模式目标：各服务独立编译部署，K8s 编排，支持大规模生产。

#### 架构图

```
用户请求 → CDN → API网关(Pingora/Axum) → 服务路由
                                            │
                    ┌───────────────────────┼───────────────────────┐
                    ▼                       ▼                       ▼
              user-svc (Pod)         project-svc (Pod)       asset-svc (Pod)
              ┌──────────┐           ┌──────────┐           ┌──────────┐
              │Axum+Tonic│           │Axum+Tonic│           │Axum+Tonic│
              └────┬─────┘           └────┬─────┘           └────┬─────┘
                   │                      │                      │
                   └──────────┬───────────┘                      │
                              ▼                                  │
                    gen-svc (Pod) ◄── Kafka ──► agent-svc (Pod)  │
                              │                      │           │
                              ▼                      ▼           │
                    ┌──────────────────────────────────┐         │
                    │        共享基础设施                 │         │
                    │  PostgreSQL (主从) │ Redis Cluster │         │
                    │  Kafka Cluster    │ MongoDB RS    │         │
                    │  Elasticsearch    │ Milvus        │         │
                    │  阿里云 OSS       │ GPU 推理集群   │         │
                    └──────────────────────────────────┘         │
```

#### 微服务独立编译

```toml
# bins/tsangwu-user-svc-bin/Cargo.toml
[package]
name = "tsangwu-user-svc-bin"
version.workspace = true
edition.workspace = true

[[bin]]
name = "user-svc"

[dependencies]
tsangwu-common.workspace = true
tsangwu-config.workspace = true
tsangwu-db = { workspace = true, features = ["postgres"] }
tsangwu-cache = { workspace = true, features = ["redis"] }
tsangwu-auth.workspace = true
tsangwu-user-svc.workspace = true
tsangwu-api-proto.workspace = true
tokio.workspace = true
axum.workspace = true
tonic.workspace = true
```

### 4.5 部署模式对比矩阵

| 维度 | 本地模式 | 单机模式 | 分布式模式 |
|------|---------|---------|-----------|
| 二进制数量 | 1 | 1 | N（每服务一个） |
| 数据库 | SQLite（嵌入） | PostgreSQL | PostgreSQL 主从 |
| 缓存 | moka（内存） | Redis | Redis Cluster |
| 消息队列 | tokio::mpsc | 可选 Kafka | Kafka Cluster |
| 对象存储 | 本地文件系统 | 本地FS / OSS | 阿里云 OSS |
| 全文检索 | SQLite FTS5 | PG tsvector / ES | Elasticsearch |
| 向量检索 | 内存暴力搜索 | 可选 Milvus | Milvus Cluster |
| 文档存储 | SQLite JSON | PG JSONB / MongoDB | MongoDB RS |
| 服务间通信 | 直接函数调用 | 直接函数调用 | gRPC + Kafka |
| 工作流引擎 | 内嵌状态机+SQLite | 内嵌状态机+PG | 状态机+PG+Kafka |
| 水平扩展 | 不支持 | 不支持 | K8s HPA |
| 启动依赖 | 无 | PG + Redis | 全套中间件 |
| 适用场景 | 开发/演示/个人 | 中小团队 | 大规模生产 |

### 4.6 配置文件示例

```yaml
# config/local.yaml
deploy_mode: local
server:
  host: "127.0.0.1"
  port: 8080
database:
  type: sqlite
  path: "./data/tsangwu.db"
cache:
  type: memory
  max_capacity: 10000
storage:
  type: local_fs
  root: "./data/storage"
message_queue:
  type: memory
  channel_size: 1000
search:
  type: sqlite
workflow:
  type: embedded
  persistence: sqlite

---
# config/standalone.yaml
deploy_mode: standalone
server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090
database:
  type: postgres
  url: "postgres://tsangwu:password@localhost:5432/tsangwu"
  max_connections: 20
cache:
  type: redis
  url: "redis://localhost:6379"
storage:
  type: s3
  endpoint: "https://oss-cn-shanghai.aliyuncs.com"
  bucket: "tsangwu-assets"
message_queue:
  type: kafka
  brokers: ["localhost:9092"]
search:
  type: elasticsearch
  url: "http://localhost:9200"
workflow:
  type: embedded
  persistence: postgres

---
# config/distributed.yaml
deploy_mode: distributed
# 各服务独立配置，通过环境变量覆盖
database:
  type: postgres
  url: "${DATABASE_URL}"
  max_connections: 50
cache:
  type: redis
  url: "${REDIS_URL}"
  cluster: true
storage:
  type: s3
  endpoint: "${OSS_ENDPOINT}"
  bucket: "${OSS_BUCKET}"
message_queue:
  type: kafka
  brokers: "${KAFKA_BROKERS}"
search:
  type: elasticsearch
  url: "${ES_URL}"
workflow:
  type: distributed
  persistence: postgres
  event_bus: kafka
```

---

## 5. 核心框架与公共库设计

### 5.1 tsangwu-common：公共类型与错误

```rust
// crates/tsangwu-common/src/error.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("未找到资源: {0}")]
    NotFound(String),
    #[error("未授权")]
    Unauthorized,
    #[error("权限不足")]
    Forbidden,
    #[error("参数校验失败: {0}")]
    Validation(String),
    #[error("业务冲突: {0}")]
    Conflict(String),
    #[error("限流")]
    RateLimited,
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn code(&self) -> u32 {
        match self {
            Self::NotFound(_) => 40400,
            Self::Unauthorized => 40100,
            Self::Forbidden => 40300,
            Self::Validation(_) => 42200,
            Self::Conflict(_) => 40900,
            Self::RateLimited => 42900,
            Self::Internal(_) => 50000,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = serde_json::json!({
            "code": self.code(),
            "message": self.to_string(),
            "data": null
        });
        (status, axum::Json(body)).into_response()
    }
}

// crates/tsangwu-common/src/response.rs
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T: serde::Serialize> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: serde::Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { code: 0, message: "ok".into(), data: Some(data) }
    }
}

// crates/tsangwu-common/src/pagination.rs
#[derive(Debug, serde::Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct PageResult<T: serde::Serialize> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub has_more: bool,
}
```

### 5.2 tsangwu-db：数据库抽象层

```rust
// crates/tsangwu-db/src/lib.rs
use sqlx::Pool;

/// 数据库后端抽象，通过 feature flag 切换
pub enum DatabasePool {
    #[cfg(feature = "postgres")]
    Postgres(Pool<sqlx::Postgres>),
    #[cfg(feature = "sqlite")]
    Sqlite(Pool<sqlx::Sqlite>),
}

impl DatabasePool {
    pub async fn init(config: &DatabaseConfig) -> anyhow::Result<Self> {
        match config {
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres { url, max_connections } => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(*max_connections)
                    .connect(url).await?;
                sqlx::migrate!("./migrations/postgres").run(&pool).await?;
                Ok(Self::Postgres(pool))
            }
            #[cfg(feature = "sqlite")]
            DatabaseConfig::Sqlite { path } => {
                std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap())?;
                let url = format!("sqlite:{}?mode=rwc", path);
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&url).await?;
                sqlx::migrate!("./migrations/sqlite").run(&pool).await?;
                Ok(Self::Sqlite(pool))
            }
        }
    }
}
```

### 5.3 tsangwu-mq：消息队列抽象层

```rust
// crates/tsangwu-mq/src/lib.rs
use async_trait::async_trait;

#[async_trait]
pub trait MessageProducer: Send + Sync {
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()>;
}

#[async_trait]
pub trait MessageConsumer: Send + Sync {
    async fn subscribe(&self, topic: &str, group: &str) -> anyhow::Result<MessageStream>;
}

pub struct MessageStream {
    inner: tokio::sync::mpsc::Receiver<Message>,
}

pub struct Message {
    pub topic: String,
    pub key: String,
    pub payload: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Kafka 实现
#[cfg(feature = "kafka")]
pub mod kafka_impl {
    use rdkafka::producer::FutureProducer;
    use rdkafka::consumer::StreamConsumer;
    // ... Kafka 生产者/消费者实现
}

// 内存队列实现（本地模式）
#[cfg(feature = "in-memory")]
pub mod memory_impl {
    use std::collections::HashMap;
    use tokio::sync::{broadcast, RwLock};

    pub struct InMemoryBroker {
        topics: RwLock<HashMap<String, broadcast::Sender<super::Message>>>,
    }
    // ... 基于 tokio broadcast channel 实现
}
```

### 5.4 tsangwu-cache：缓存抽象层

```rust
// crates/tsangwu-cache/src/lib.rs
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> anyhow::Result<()>;
    async fn del(&self, key: &str) -> anyhow::Result<()>;
    async fn exists(&self, key: &str) -> anyhow::Result<bool>;
}

// Redis 实现
#[cfg(feature = "redis")]
pub mod redis_impl {
    use fred::prelude::*;
    pub struct RedisCache { client: RedisClient }
}

// 内存缓存实现（本地模式）
#[cfg(feature = "in-memory")]
pub mod memory_impl {
    use moka::future::Cache;
    pub struct MemoryCache { inner: Cache<String, Vec<u8>> }
}
```

### 5.5 tsangwu-auth：认证与权限

```rust
// crates/tsangwu-auth/src/jwt.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,        // user_uid
    pub roles: Vec<String>, // 角色列表
    pub org_id: Option<i64>,
    pub plan: String,       // 订阅套餐
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl: chrono::Duration,
    refresh_ttl: chrono::Duration,
}

impl JwtManager {
    pub fn issue_access_token(&self, claims: &Claims) -> anyhow::Result<String> { /* ... */ }
    pub fn verify_access_token(&self, token: &str) -> anyhow::Result<Claims> { /* ... */ }
}

// crates/tsangwu-auth/src/middleware.rs
use axum::{extract::Request, middleware::Next, response::Response};

/// Axum 认证中间件
pub async fn auth_middleware(
    State(jwt): State<Arc<JwtManager>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = jwt.verify_access_token(token)?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

// crates/tsangwu-auth/src/rbac.rs
/// RBAC 权限校验
pub async fn require_permission(
    claims: &Claims,
    resource: &str,
    action: &str,
    db: &DatabasePool,
) -> Result<(), AppError> {
    let has_perm = sqlx::query_scalar!(
        "SELECT EXISTS(
            SELECT 1 FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = (SELECT id FROM users WHERE uid = $1)
            AND p.resource = $2 AND p.action = $3
        )",
        claims.sub, resource, action
    )
    .fetch_one(db.pg_pool())
    .await?;

    if !has_perm.unwrap_or(false) {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
```

### 5.6 tsangwu-storage：存储抽象层

```rust
// crates/tsangwu-storage/src/lib.rs
use opendal::Operator;

pub struct StorageService {
    operator: Operator,
    base_url: String,
}

impl StorageService {
    /// 根据配置初始化存储后端
    pub fn new(config: &StorageConfig) -> anyhow::Result<Self> {
        let operator = match config {
            StorageConfig::LocalFs { root } => {
                let builder = opendal::services::Fs::default().root(root);
                Operator::new(builder)?.finish()
            }
            StorageConfig::S3 { endpoint, bucket, access_key, secret_key, .. } => {
                let builder = opendal::services::S3::default()
                    .endpoint(endpoint)
                    .bucket(bucket)
                    .access_key_id(access_key)
                    .secret_access_key(secret_key);
                Operator::new(builder)?.finish()
            }
        };
        Ok(Self { operator, base_url: config.base_url().into() })
    }

    pub async fn upload(&self, path: &str, data: Vec<u8>) -> anyhow::Result<String> {
        self.operator.write(path, data).await?;
        Ok(format!("{}/{}", self.base_url, path))
    }

    pub async fn download(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        Ok(self.operator.read(path).await?.to_vec())
    }

    pub async fn delete(&self, path: &str) -> anyhow::Result<()> {
        self.operator.delete(path).await?;
        Ok(())
    }

    pub async fn presign_read(&self, path: &str, ttl: Duration) -> anyhow::Result<String> {
        let url = self.operator.presign_read(path, ttl).await?;
        Ok(url.uri().to_string())
    }
}
```

---

## 6. 微服务/模块详细设计

### 6.1 服务全景

```
┌─────────────────────────────────────────────────────────┐
│                    API Gateway (Pingora/Axum)             │
├──────────┬──────────┬──────────┬──────────┬─────────────┤
│ user-svc │ proj-svc │asset-svc │ gen-svc  │ dist-svc    │
├──────────┼──────────┼──────────┼──────────┼─────────────┤
│ auth-svc │ tmpl-svc │ copy-svc │agent-svc │ audit-svc   │
├──────────┼──────────┼──────────┴──────────┴─────────────┤
│ pay-svc  │notif-svc │         culture-svc                │
└──────────┴──────────┴────────────────────────────────────┘
```

### 6.2 各服务职责与 Rust 实现要点

| 服务名 | 职责 | Rust 实现要点 | 依赖 crate |
|--------|------|--------------|-----------|
| **user-svc** | 用户账号管理 | SeaORM ActiveModel CRUD；argon2 密码哈希 | tsangwu-db, tsangwu-auth |
| **auth-svc** | 认证与权限 | jsonwebtoken JWT 签发；RBAC 中间件 | tsangwu-auth, tsangwu-cache |
| **proj-svc** | 项目与创作管理 | JSONB 配置存储；版本快照；WebSocket 协同 | tsangwu-db, tsangwu-mq |
| **asset-svc** | 素材库管理 | OpenDAL 上传；ES 全文检索；Milvus 向量检索 | tsangwu-storage, tsangwu-db |
| **tmpl-svc** | 模板引擎 | MongoDB 灵活 Schema；推荐算法 | mongodb, tsangwu-cache |
| **gen-svc** | 生成任务管理 | 任务状态机；Kafka 异步调度；进度 WebSocket | tsangwu-mq, tsangwu-workflow |
| **agent-svc** | Agent 编排 | 工作流引擎驱动；PyO3 调用 AI 模型 | tsangwu-workflow, tsangwu-pybridge |
| **culture-svc** | 文化适配 | 规则引擎；知识图谱查询；向量匹配 | mongodb, tsangwu-pybridge |
| **copy-svc** | 版权保护 | 感知哈希(pHash)；特征向量比对 | tsangwu-db, tsangwu-storage |
| **audit-svc** | 内容审核 | Kafka 消费审核事件；外部审核 API 对接 | tsangwu-mq, reqwest |
| **dist-svc** | 输出分发 | FFmpeg 格式转换；CDN 推送；平台 API 对接 | tsangwu-storage, ffmpeg-next |
| **pay-svc** | 支付与订阅 | 支付网关对接；用量计费状态机 | tsangwu-db, reqwest |
| **notif-svc** | 通知服务 | Kafka 事件消费；WebSocket/邮件/短信多通道 | tsangwu-mq, tsangwu-cache |

### 6.3 服务间通信方式

```
user-svc ──gRPC(Tonic)──► auth-svc          (鉴权校验)
proj-svc ──gRPC(Tonic)──► asset-svc         (素材引用)
proj-svc ──Kafka────────► gen-svc           (触发生成)
gen-svc  ──Workflow─────► agent-svc         (Agent编排)
agent-svc──gRPC(Tonic)──► culture-svc       (文化校验)
agent-svc──gRPC(Tonic)──► asset-svc         (素材检索)
gen-svc  ──Kafka────────► audit-svc         (内容审核)
gen-svc  ──Kafka────────► copy-svc          (版权校验)
gen-svc  ──Kafka────────► dist-svc          (输出分发)
pay-svc  ──gRPC(Tonic)──► user-svc          (权益查询)
notif-svc◄─Kafka──────── *                  (事件通知)
```

**本地/单机模式下**：gRPC 调用降级为直接 Rust 函数调用（同进程内 trait 方法调用），Kafka 降级为内存 channel，零网络开销。

```rust
// 服务间调用抽象
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn verify_token(&self, token: &str) -> Result<Claims, AppError>;
    async fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool, AppError>;
}

// 分布式模式：gRPC 远程调用
pub struct RemoteAuthService { client: AuthServiceClient<Channel> }

// 本地/单机模式：直接函数调用
pub struct LocalAuthService { jwt: Arc<JwtManager>, db: DatabasePool }
```

### 6.4 服务分期上线策略

| 阶段 | 上线服务 | 说明 |
|------|----------|------|
| Phase 1 | user-svc, auth-svc, asset-svc, tmpl-svc, notif-svc | 基础设施与账号体系 |
| Phase 2 | proj-svc, gen-svc, agent-svc, culture-svc, audit-svc, copy-svc, dist-svc | 图文+短剧生成全链路 |
| Phase 3 | pay-svc；gen-svc 扩展电影管线 | 商业化+电影模块 |
| Phase 4 | gen-svc 扩展电视剧管线；proj-svc 扩展协同编辑 | 电视剧+团队协同 |

---

## 7. 数据库与存储层（SQLx/SeaORM + OpenDAL）

### 7.1 存储策略总览

| 存储类型 | 分布式模式 | 单机模式 | 本地模式 | 存储内容 |
|----------|-----------|---------|---------|----------|
| 关系型 | PostgreSQL | PostgreSQL | SQLite | 用户、权限、项目、订单、版权等 |
| 文档型 | MongoDB | PG JSONB | SQLite JSON | 素材元数据、模板配置、Agent 上下文 |
| 缓存 | Redis Cluster | Redis | moka 内存 | 会话、热点数据、分布式锁 |
| 对象存储 | 阿里云 OSS | 本地FS/OSS | 本地文件系统 | 图片、视频、音频等二进制文件 |
| 全文检索 | Elasticsearch | PG tsvector/ES | SQLite FTS5 | 素材标签检索、内容搜索 |
| 向量存储 | Milvus | 可选 Milvus | 内存暴力搜索 | 素材语义向量、风格特征向量 |

### 7.2 核心数据模型（SeaORM Entity）

#### 用户与权限

```rust
// crates/tsangwu-db/src/entities/user.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uid: Uuid,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub account_type: i16,    // 0:个人 1:企业 2:专业团队
    pub status: i16,          // 0:禁用 1:正常
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project::Entity")]
    Projects,
    #[sea_orm(has_many = "super::subscription::Entity")]
    Subscriptions,
}
```

#### 项目与创作

```rust
// crates/tsangwu-db/src/entities/project.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uid: Uuid,
    pub owner_id: i64,
    pub org_id: Option<i64>,
    pub title: String,
    pub scene_type: i16,      // 0:图文 1:短剧 2:电影 3:电视剧
    pub user_mode: i16,       // 0:零基础 1:进阶 2:专业
    pub status: i16,          // 0:草稿 1:生成中 2:已完成 3:已发布
    pub config: serde_json::Value,  // JSONB 项目配置
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}
```

#### 生成任务

```rust
// crates/tsangwu-db/src/entities/generation_task.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "generation_tasks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uid: Uuid,
    pub project_id: i64,
    pub user_id: i64,
    pub task_type: i16,       // 0:图文 1:短剧 2:电影 3:电视剧
    pub priority: i16,        // 0:低 1:中 2:高
    pub status: i16,          // 0:待处理 1:排队中 2:生成中 3:已完成 4:失败
    pub input_params: serde_json::Value,
    pub output_urls: Option<serde_json::Value>,
    pub progress: Option<i16>,
    pub error_msg: Option<String>,
    pub started_at: Option<DateTimeWithTimeZone>,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
}
```

### 7.3 SQL 迁移管理

采用 SQLx 内置迁移机制，PostgreSQL 和 SQLite 各维护一套迁移脚本：

```
crates/tsangwu-db/migrations/
├── postgres/
│   ├── 20260217_000001_create_users.sql
│   ├── 20260217_000002_create_organizations.sql
│   ├── 20260217_000003_create_roles_permissions.sql
│   ├── 20260217_000004_create_projects.sql
│   ├── 20260217_000005_create_generation_tasks.sql
│   ├── 20260217_000006_create_subscriptions.sql
│   ├── 20260217_000007_create_copyright_reviews.sql
│   └── 20260217_000008_create_workflow_tables.sql
└── sqlite/
    ├── 20260217_000001_init.sql    # SQLite 合并为较少文件
    └── 20260217_000002_workflow.sql
```

PostgreSQL 迁移示例（与原 Go 方案 SQL 一致）：

```sql
-- migrations/postgres/20260217_000001_create_users.sql
CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    phone           VARCHAR(20) UNIQUE,
    email           VARCHAR(128) UNIQUE,
    nickname        VARCHAR(64) NOT NULL,
    avatar_url      VARCHAR(512),
    account_type    SMALLINT NOT NULL DEFAULT 0,
    status          SMALLINT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE organizations (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(128) NOT NULL,
    license_no      VARCHAR(64),
    owner_id        BIGINT NOT NULL REFERENCES users(id),
    max_members     INT NOT NULL DEFAULT 5,
    status          SMALLINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE roles (
    id              SERIAL PRIMARY KEY,
    code            VARCHAR(32) NOT NULL UNIQUE,
    name            VARCHAR(64) NOT NULL,
    description     TEXT
);

CREATE TABLE user_roles (
    user_id         BIGINT NOT NULL REFERENCES users(id),
    role_id         INT NOT NULL REFERENCES roles(id),
    org_id          BIGINT REFERENCES organizations(id),
    PRIMARY KEY (user_id, role_id, org_id)
);

CREATE TABLE permissions (
    id              SERIAL PRIMARY KEY,
    code            VARCHAR(64) NOT NULL UNIQUE,
    resource        VARCHAR(64) NOT NULL,
    action          VARCHAR(32) NOT NULL
);

CREATE TABLE role_permissions (
    role_id         INT NOT NULL REFERENCES roles(id),
    permission_id   INT NOT NULL REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);
```

### 7.4 文档数据模型（MongoDB / PG JSONB）

素材元数据与 Agent 上下文采用灵活 Schema 存储：

```rust
// crates/tsangwu-db/src/models/asset_meta.rs
/// 素材元数据（MongoDB collection: assets / PG JSONB 降级）
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetMeta {
    pub asset_uid: Uuid,
    pub name: String,
    pub category: AssetCategory,    // costume/scene/prop/color/pattern/narrative
    pub sub_category: String,
    pub tags: Vec<String>,
    pub dynasty: Option<String>,
    pub cultural_labels: Vec<String>,
    pub file_urls: FileUrls,
    pub metadata: FileMetadata,
    pub embedding_id: Option<String>,
    pub copyright: CopyrightInfo,
    pub usage_count: u64,
    pub status: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileUrls {
    pub thumbnail: String,
    pub preview: String,
    pub original: String,
}

/// Agent 上下文（MongoDB collection: agent_contexts）
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentContext {
    pub task_id: Uuid,
    pub workflow_run_id: String,
    pub agents: std::collections::HashMap<String, AgentState>,
    pub shared_context: SharedContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: String,
    pub progress: Option<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SharedContext {
    pub style_profile: serde_json::Value,
    pub character_refs: Vec<String>,
    pub scene_refs: Vec<String>,
}
```

### 7.5 文档存储抽象

```rust
// crates/tsangwu-db/src/doc_store.rs
/// 文档存储抽象，支持 MongoDB 和 PG JSONB 两种后端
#[async_trait]
pub trait DocumentStore: Send + Sync {
    async fn insert<T: serde::Serialize + Send>(&self, collection: &str, doc: &T) -> anyhow::Result<String>;
    async fn find_one<T: serde::de::DeserializeOwned>(&self, collection: &str, filter: serde_json::Value) -> anyhow::Result<Option<T>>;
    async fn find_many<T: serde::de::DeserializeOwned>(&self, collection: &str, filter: serde_json::Value, limit: u64) -> anyhow::Result<Vec<T>>;
    async fn update_one(&self, collection: &str, filter: serde_json::Value, update: serde_json::Value) -> anyhow::Result<bool>;
    async fn delete_one(&self, collection: &str, filter: serde_json::Value) -> anyhow::Result<bool>;
}

/// MongoDB 实现
pub struct MongoDocStore { db: mongodb::Database }

/// PG JSONB 降级实现（单机/本地模式）
/// 使用 documents 表：(id, collection, data JSONB, created_at)
pub struct PgDocStore { pool: sqlx::PgPool }

/// SQLite JSON 降级实现（本地模式）
pub struct SqliteDocStore { pool: sqlx::SqlitePool }
```

### 7.6 数据关系总览

```
users ──1:N──► projects ──1:N──► generation_tasks
  │                                    │
  ├──1:N──► subscriptions              ├──1:1──► copyright_records
  ├──1:N──► usage_records              ├──1:N──► content_reviews
  └──N:M──► roles ──N:M──► permissions └──1:1──► agent_contexts (文档存储)
                                                       │
organizations ──1:N──► users                    assets (文档存储)
                                                  │
project_versions ◄──N:1── projects          Milvus (向量索引)
```

---

## 8. API 层设计（Axum + Tonic）

### 8.1 RESTful 接口规范

| 规范项 | 约定 |
|--------|------|
| 基础路径 | `https://api.tsangwu.com/v1` |
| 认证方式 | Bearer Token (JWT)；开放 API 使用 API Key + HMAC 签名 |
| 请求格式 | `Content-Type: application/json`；文件上传使用 `multipart/form-data` |
| 响应格式 | 统一 JSON 信封：`{ "code": 0, "message": "ok", "data": {} }` |
| 分页 | `?page=1&page_size=20`，响应含 `total` / `has_more` |
| 错误码 | 业务错误码 5 位数字，HTTP 状态码遵循 RESTful 语义 |
| 版本控制 | URL 路径版本 `/v1`，大版本不兼容时升级 |
| 限流 | 按用户套餐分级限流，响应头返回 `X-RateLimit-*` |

### 8.2 Axum 路由设计

```rust
// crates/tsangwu-user-svc/src/routes.rs
use axum::{Router, routing::{get, post, put, delete}, middleware};

pub fn routes() -> Router<AppState> {
    Router::new()
        // 认证（无需登录）
        .route("/v1/auth/register", post(handlers::register))
        .route("/v1/auth/login", post(handlers::login))
        .route("/v1/auth/refresh", post(handlers::refresh_token))
        .route("/v1/auth/oauth/{provider}", post(handlers::oauth_login))
        // 用户（需登录）
        .route("/v1/users/me", get(handlers::get_me).put(handlers::update_me))
        .route("/v1/users/me/subscription", get(handlers::get_subscription))
        .route("/v1/users/me/usage", get(handlers::get_usage))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

// crates/tsangwu-project-svc/src/routes.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/projects", post(handlers::create).get(handlers::list))
        .route("/v1/projects/{id}", get(handlers::get).put(handlers::update).delete(handlers::remove))
        .route("/v1/projects/{id}/versions", get(handlers::list_versions).post(handlers::create_version))
        .route("/v1/projects/{id}/generate", post(handlers::submit_generation))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

// crates/tsangwu-gen-svc/src/routes.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/tasks", get(handlers::list_tasks))
        .route("/v1/tasks/{task_id}", get(handlers::get_task))
        .route("/v1/tasks/{task_id}/result", get(handlers::get_result))
        .route("/v1/tasks/{task_id}/cancel", post(handlers::cancel))
        .route("/v1/tasks/{task_id}/retry", post(handlers::retry))
        .route("/v1/tasks/{task_id}/export", post(handlers::export))
        .route("/v1/tasks/{task_id}/exports", get(handlers::list_exports))
        .route("/v1/tasks/{task_id}/publish", post(handlers::publish))
        .route("/v1/tasks/{task_id}/copyright", get(handlers::get_copyright))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

// crates/tsangwu-asset-svc/src/routes.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/assets", get(handlers::list))
        .route("/v1/assets/{id}", get(handlers::get).delete(handlers::remove))
        .route("/v1/assets/search", post(handlers::semantic_search))
        .route("/v1/assets/upload", post(handlers::upload))
        .route("/v1/assets/{id}/favorite", post(handlers::favorite))
        .route("/v1/assets/favorites", get(handlers::list_favorites))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
}

// 开放 API（API Key 认证）
pub fn open_api_routes() -> Router<AppState> {
    Router::new()
        .route("/v1/open/generate/image", post(open_handlers::gen_image))
        .route("/v1/open/generate/video", post(open_handlers::gen_video))
        .route("/v1/open/tasks/{task_id}", get(open_handlers::get_task))
        .route("/v1/open/assets/search", post(open_handlers::search_assets))
        .route("/v1/open/quota", get(open_handlers::get_quota))
        .route_layer(middleware::from_fn(api_key_auth_middleware))
}
```

### 8.3 Handler 示例：提交生成任务

```rust
// crates/tsangwu-gen-svc/src/handlers.rs
use axum::{extract::{State, Path, Json}, Extension};

#[derive(Debug, serde::Deserialize)]
pub struct GenerateRequest {
    pub scene_type: String,
    pub mode: String,
    pub input: GenerateInput,
    pub config: GenerateConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct GenerateInput {
    pub prompt: String,
    pub reference_images: Option<Vec<String>>,
    pub template_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct GenerateConfig {
    pub duration_minutes: Option<u32>,
    pub resolution: Option<String>,
    pub style: Option<String>,
    pub music_style: Option<String>,
    pub dynasty: Option<String>,
}

pub async fn submit_generation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<ApiResponse<GenerateResponse>>, AppError> {
    // 1. 权限校验
    require_permission(&claims, "task", "execute", &state.db).await?;

    // 2. 用量检查
    let usage = state.pay_service.check_quota(&claims.sub, &req.scene_type).await?;
    if !usage.has_quota {
        return Err(AppError::Forbidden);
    }

    // 3. 创建任务记录
    let task = GenerationTask::new_active_model(project_id, &claims, &req);
    let task = task.insert(&state.db).await?;

    // 4. 发送到消息队列
    state.mq.publish(
        "generation.tasks",
        &task.uid.to_string(),
        &serde_json::to_vec(&task)?,
    ).await?;

    // 5. 返回任务信息
    Ok(Json(ApiResponse::ok(GenerateResponse {
        task_id: task.uid,
        status: "queued".into(),
        ws_channel: format!("wss://ws.tsangwu.com/tasks/{}/progress", task.uid),
    })))
}
```

### 8.4 Tonic gRPC 服务定义

```protobuf
// proto/generation.proto
syntax = "proto3";
package tsangwu.generation;

service GenerationService {
    rpc SubmitTask (SubmitTaskRequest) returns (SubmitTaskResponse);
    rpc GetTaskStatus (GetTaskStatusRequest) returns (TaskStatus);
    rpc CancelTask (CancelTaskRequest) returns (CancelTaskResponse);
    rpc StreamProgress (StreamProgressRequest) returns (stream ProgressUpdate);
}

message SubmitTaskRequest {
    int64 project_id = 1;
    string scene_type = 2;
    string mode = 3;
    string input_json = 4;
    string config_json = 5;
}

message TaskStatus {
    string task_id = 1;
    string status = 2;
    int32 progress = 3;
    string error_msg = 4;
}

message ProgressUpdate {
    string task_id = 1;
    int32 progress = 2;
    string stage = 3;
    string message = 4;
}
```

```rust
// crates/tsangwu-gen-svc/src/grpc.rs
use tonic::{Request, Response, Status};
use tsangwu_api_proto::generation::*;

pub struct GenerationGrpcService {
    state: AppState,
}

#[tonic::async_trait]
impl generation_service_server::GenerationService for GenerationGrpcService {
    async fn submit_task(
        &self,
        request: Request<SubmitTaskRequest>,
    ) -> Result<Response<SubmitTaskResponse>, Status> {
        let req = request.into_inner();
        // ... 业务逻辑复用 HTTP handler 的核心逻辑
        Ok(Response::new(SubmitTaskResponse { task_id, status: "queued".into() }))
    }

    type StreamProgressStream = tokio_stream::wrappers::ReceiverStream<Result<ProgressUpdate, Status>>;

    async fn stream_progress(
        &self,
        request: Request<StreamProgressRequest>,
    ) -> Result<Response<Self::StreamProgressStream>, Status> {
        let task_id = request.into_inner().task_id;
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // 订阅任务进度更新
        tokio::spawn(async move {
            // 从 Redis pub/sub 或内存 channel 接收进度
            // 转发到 gRPC stream
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
```

### 8.5 Tower 中间件栈

```rust
// bins/tsangwu-server/src/main.rs
use tower_http::{cors::CorsLayer, trace::TraceLayer, compression::CompressionLayer, limit::RequestBodyLimitLayer};

fn build_app(state: AppState) -> Router {
    Router::new()
        .merge(tsangwu_user_svc::routes())
        .merge(tsangwu_project_svc::routes())
        .merge(tsangwu_asset_svc::routes())
        .merge(tsangwu_gen_svc::routes())
        .merge(tsangwu_template_svc::routes())
        .merge(open_api_routes())
        // 健康检查（无需认证）
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(readiness_check))
        // 中间件栈（从下往上执行）
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(100 * 1024 * 1024)) // 100MB
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(RateLimitLayer::new(state.cache.clone()))
        .with_state(state)
}

// HTTP + gRPC 共享端口（通过 content-type 路由）
async fn start_server(state: AppState) -> anyhow::Result<()> {
    let http_app = build_app(state.clone());
    let grpc_app = tonic::transport::Server::builder()
        .add_service(GenerationGrpcService::new(state.clone()))
        .add_service(AuthGrpcService::new(state.clone()))
        .into_router();

    // 合并 HTTP 和 gRPC 路由
    let app = Router::new()
        .merge(http_app)
        .merge(grpc_app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### 8.6 WebSocket 实时进度推送

```rust
// crates/tsangwu-gen-svc/src/ws.rs
use axum::{extract::{ws::{WebSocket, WebSocketUpgrade}, Path, State}, response::Response};

pub async fn task_progress_ws(
    ws: WebSocketUpgrade,
    Path(task_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_progress(socket, task_id, state))
}

async fn handle_progress(mut socket: WebSocket, task_id: Uuid, state: AppState) {
    // 订阅任务进度 channel
    let mut rx = state.progress_bus.subscribe(task_id).await;

    while let Ok(update) = rx.recv().await {
        let msg = serde_json::to_string(&update).unwrap();
        if socket.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
            break;
        }
        if update.progress >= 100 {
            break;
        }
    }
}
```

---

## 9. 工作流引擎设计（自研状态机方案）

### 9.1 设计背景

Temporal 无官方 Rust SDK，社区绑定不够成熟。自研轻量工作流引擎，基于有限状态机 + 事件溯源模式，满足苍梧 Agent 编排需求。

### 9.2 核心架构

```
┌──────────────────────────────────────────────────────────┐
│                  苍梧工作流引擎 (tsangwu-workflow)          │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ 工作流定义    │  │ 执行引擎      │  │ 状态持久化    │   │
│  │ (Rust DSL)   │  │ (状态机驱动)  │  │ (PG/SQLite)  │   │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘   │
│         │                 │                  │            │
│  ┌──────▼─────────────────▼──────────────────▼───────┐   │
│  │              事件总线 (Kafka / 内存 channel)         │   │
│  └───────────────────────┬───────────────────────────┘   │
│                          │                                │
│  ┌───────────────────────▼───────────────────────────┐   │
│  │              调度器 (重试 / 超时 / 补偿)             │   │
│  └───────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### 9.3 核心数据模型

```sql
-- 工作流定义表
CREATE TABLE workflow_definitions (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(128) NOT NULL UNIQUE,
    version         INT NOT NULL DEFAULT 1,
    definition_json JSONB NOT NULL,       -- 状态机定义（状态、转换、步骤）
    status          SMALLINT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (name, version)
);

-- 工作流实例表
CREATE TABLE workflow_instances (
    id              BIGSERIAL PRIMARY KEY,
    instance_id     UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    definition_id   BIGINT NOT NULL REFERENCES workflow_definitions(id),
    correlation_id  VARCHAR(256),         -- 关联业务ID（如 task_uid）
    current_state   VARCHAR(64) NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:运行中 1:已完成 2:失败 3:已取消 4:暂停
    context_json    JSONB NOT NULL DEFAULT '{}',  -- 工作流上下文数据
    retry_count     INT NOT NULL DEFAULT 0,
    max_retries     INT NOT NULL DEFAULT 3,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_wf_instance_correlation ON workflow_instances(correlation_id);
CREATE INDEX idx_wf_instance_status ON workflow_instances(status);

-- 工作流事件表（事件溯源）
CREATE TABLE workflow_events (
    id              BIGSERIAL PRIMARY KEY,
    instance_id     UUID NOT NULL REFERENCES workflow_instances(instance_id),
    event_type      VARCHAR(64) NOT NULL,   -- state_entered/state_exited/step_completed/step_failed/timeout
    from_state      VARCHAR(64),
    to_state        VARCHAR(64),
    step_name       VARCHAR(128),
    payload_json    JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_wf_events_instance ON workflow_events(instance_id, created_at);

-- 步骤执行表
CREATE TABLE workflow_steps (
    id              BIGSERIAL PRIMARY KEY,
    instance_id     UUID NOT NULL REFERENCES workflow_instances(instance_id),
    step_name       VARCHAR(128) NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:待执行 1:执行中 2:已完成 3:失败 4:已跳过
    input_json      JSONB,
    output_json     JSONB,
    error_msg       TEXT,
    retry_count     INT NOT NULL DEFAULT 0,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    timeout_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_wf_steps_instance ON workflow_steps(instance_id);
```

### 9.4 Rust 状态机实现

```rust
// crates/tsangwu-workflow/src/definition.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工作流定义（Rust DSL）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub version: u32,
    pub initial_state: String,
    pub states: HashMap<String, StateDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDefinition {
    pub state_type: StateType,
    pub steps: Vec<StepDefinition>,
    pub transitions: Vec<Transition>,
    pub on_error: Option<ErrorHandler>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Initial,
    Processing,     // 执行步骤
    WaitForEvent,   // 等待外部事件（如用户审批）
    Parallel,       // 并行执行多个分支
    Decision,       // 条件分支
    Compensation,   // 补偿回滚
    Terminal,       // 终态（成功/失败）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub name: String,
    pub handler: String,          // 步骤处理器名称
    pub retry_policy: RetryPolicy,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub event: String,            // 触发事件
    pub target_state: String,     // 目标状态
    pub condition: Option<String>,// 条件表达式（可选）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_interval: Duration,
    pub backoff_multiplier: f64,
    pub max_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandler {
    pub strategy: ErrorStrategy,
    pub target_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    Retry,
    Compensate,
    FailWorkflow,
    GoToState,
}
```

### 9.5 工作流执行引擎

```rust
// crates/tsangwu-workflow/src/engine.rs
use std::sync::Arc;

pub struct WorkflowEngine {
    persistence: Arc<dyn WorkflowPersistence>,
    event_bus: Arc<dyn EventBus>,
    step_registry: Arc<StepHandlerRegistry>,
    scheduler: Arc<WorkflowScheduler>,
}

/// 步骤处理器 trait
#[async_trait]
pub trait StepHandler: Send + Sync {
    async fn execute(&self, ctx: &mut StepContext) -> Result<StepOutput, StepError>;
    async fn compensate(&self, ctx: &mut StepContext) -> Result<(), StepError>;
}

/// 步骤上下文
pub struct StepContext {
    pub instance_id: Uuid,
    pub step_name: String,
    pub input: serde_json::Value,
    pub workflow_context: serde_json::Value,
    pub shared_state: Arc<tokio::sync::RwLock<serde_json::Value>>,
}

impl WorkflowEngine {
    /// 启动新工作流实例
    pub async fn start_workflow(
        &self,
        definition_name: &str,
        correlation_id: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let definition = self.persistence.get_definition(definition_name).await?;
        let instance_id = Uuid::new_v4();

        // 持久化工作流实例
        self.persistence.create_instance(&WorkflowInstance {
            instance_id,
            definition_id: definition.id,
            correlation_id: correlation_id.into(),
            current_state: definition.initial_state.clone(),
            status: WorkflowStatus::Running,
            context_json: input,
            ..Default::default()
        }).await?;

        // 发送启动事件
        self.event_bus.publish("workflow.events", &WorkflowEvent {
            instance_id,
            event_type: "workflow_started".into(),
            to_state: Some(definition.initial_state),
            ..Default::default()
        }).await?;

        Ok(instance_id)
    }

    /// 驱动工作流状态转换（核心循环）
    pub async fn process_event(&self, event: WorkflowEvent) -> anyhow::Result<()> {
        let instance = self.persistence.get_instance(event.instance_id).await?;
        let definition = self.persistence.get_definition_by_id(instance.definition_id).await?;
        let state_def = &definition.states[&instance.current_state];

        match state_def.state_type {
            StateType::Processing => {
                // 顺序执行当前状态的所有步骤
                for step_def in &state_def.steps {
                    let handler = self.step_registry.get(&step_def.handler)?;
                    let mut ctx = StepContext::from_instance(&instance, step_def);

                    match handler.execute(&mut ctx).await {
                        Ok(output) => {
                            self.persistence.complete_step(instance.instance_id, &step_def.name, &output).await?;
                        }
                        Err(e) => {
                            return self.handle_step_error(&instance, step_def, state_def, e).await;
                        }
                    }
                }
                // 所有步骤完成，触发转换
                self.transition(&instance, &definition, "steps_completed").await?;
            }
            StateType::Parallel => {
                // 并行执行所有步骤
                let mut handles = Vec::new();
                for step_def in &state_def.steps {
                    let handler = self.step_registry.get(&step_def.handler)?;
                    let ctx = StepContext::from_instance(&instance, step_def);
                    handles.push(tokio::spawn(async move { handler.execute(&mut ctx).await }));
                }
                let results = futures::future::join_all(handles).await;
                // 汇总结果，全部成功则转换
            }
            StateType::WaitForEvent => {
                // 等待外部信号（如用户审批），不主动推进
            }
            StateType::Decision => {
                // 根据条件选择转换路径
                for transition in &state_def.transitions {
                    if self.evaluate_condition(&transition.condition, &instance.context_json) {
                        self.transition(&instance, &definition, &transition.event).await?;
                        break;
                    }
                }
            }
            StateType::Terminal => {
                self.persistence.complete_instance(instance.instance_id).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 9.6 生成任务工作流定义示例

```rust
// crates/tsangwu-agent-svc/src/workflows/generation.rs
pub fn generation_workflow_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        name: "generation_pipeline".into(),
        version: 1,
        initial_state: "parsing".into(),
        states: HashMap::from([
            ("parsing".into(), StateDefinition {
                state_type: StateType::Processing,
                steps: vec![step("parse_requirement", "parser_agent", 2, 30)],
                transitions: vec![Transition::on("steps_completed", "planning")],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::Retry, target_state: None }),
                timeout: Some(Duration::from_secs(60)),
            }),
            ("planning".into(), StateDefinition {
                state_type: StateType::Processing,
                steps: vec![step("create_plan", "planner_agent", 2, 120)],
                transitions: vec![
                    Transition::on_condition("steps_completed", "user_approval", "mode == 'professional'"),
                    Transition::on("steps_completed", "generating"),
                ],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::Retry, target_state: None }),
                timeout: Some(Duration::from_secs(180)),
            }),
            ("user_approval".into(), StateDefinition {
                state_type: StateType::WaitForEvent,
                steps: vec![],
                transitions: vec![
                    Transition::on("approved", "generating"),
                    Transition::on("rejected", "planning"),
                ],
                on_error: None,
                timeout: Some(Duration::from_secs(86400)), // 24h 超时
            }),
            ("generating".into(), StateDefinition {
                state_type: StateType::Parallel,
                steps: vec![step("generate_content", "generator_agent", 3, 300)],
                transitions: vec![Transition::on("steps_completed", "editing")],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::Retry, target_state: None }),
                timeout: Some(Duration::from_secs(600)),
            }),
            ("editing".into(), StateDefinition {
                state_type: StateType::Processing,
                steps: vec![step("optimize_content", "editor_agent", 2, 120)],
                transitions: vec![Transition::on("steps_completed", "reviewing")],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::Retry, target_state: None }),
                timeout: Some(Duration::from_secs(180)),
            }),
            ("reviewing".into(), StateDefinition {
                state_type: StateType::Processing,
                steps: vec![step("review_content", "reviewer_agent", 1, 60)],
                transitions: vec![
                    Transition::on_condition("steps_completed", "generating", "needs_revision"),
                    Transition::on("steps_completed", "compliance_check"),
                ],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::FailWorkflow, target_state: None }),
                timeout: Some(Duration::from_secs(120)),
            }),
            ("compliance_check".into(), StateDefinition {
                state_type: StateType::Parallel,
                steps: vec![
                    step("copyright_check", "copyright_handler", 1, 30),
                    step("content_audit", "audit_handler", 1, 30),
                ],
                transitions: vec![
                    Transition::on("steps_completed", "completed"),
                    Transition::on("audit_failed", "failed"),
                ],
                on_error: Some(ErrorHandler { strategy: ErrorStrategy::FailWorkflow, target_state: None }),
                timeout: Some(Duration::from_secs(60)),
            }),
            ("completed".into(), StateDefinition {
                state_type: StateType::Terminal,
                steps: vec![],
                transitions: vec![],
                on_error: None,
                timeout: None,
            }),
            ("failed".into(), StateDefinition {
                state_type: StateType::Terminal,
                steps: vec![],
                transitions: vec![],
                on_error: None,
                timeout: None,
            }),
        ]),
    }
}
```

### 9.7 部署模式适配

| 维度 | 本地模式 | 单机模式 | 分布式模式 |
|------|---------|---------|-----------|
| 状态持久化 | SQLite | PostgreSQL | PostgreSQL |
| 事件总线 | tokio::mpsc channel | 可选 Kafka | Kafka |
| 调度器 | 单线程 Tokio task | 多线程 Tokio task | 多实例竞争消费 |
| 并发工作流 | ≤ 10 | ≤ 100 | ≤ 10000+ |
| 超时检测 | tokio::time::sleep | tokio::time::sleep | 独立超时检测服务 |

### 9.8 与 Temporal 方案对比

| 维度 | Temporal (原方案) | 自研状态机引擎 |
|------|------------------|---------------|
| 语言支持 | Go/Java/Python/TypeScript（无 Rust） | 原生 Rust |
| 部署复杂度 | 需独立部署 Temporal Server + DB | 嵌入应用，零额外部署 |
| 本地模式 | 不支持嵌入 | 完美支持 |
| 功能完整度 | 企业级全功能 | 覆盖核心需求（状态机/重试/补偿/超时/并行） |
| 可观测性 | 自带 Web UI | 需自建（集成 tracing） |
| 维护成本 | 低（社区维护） | 中（需自行维护） |
| 性能 | 高 | 极高（零序列化开销，同进程调用） |

---

## 10. AI Agent 系统 Rust 实现

### 10.1 五大 Agent 总览

```
用户输入
   │
   ▼
┌──────────────┐    ┌──────────────┐    ┌───────────────┐
│  需求解析     │───►│  创意规划     │───►│  内容生成      │
│ Parser Agent │    │Planner Agent │    │Generator Agent│
└──────────────┘    └──────────────┘    └──────┬────────┘
                                               │
                                               ▼
                                        ┌──────────────┐    ┌──────────────┐
                                        │  编辑优化     │───►│  审片校验     │
                                        │ Editor Agent │    │Reviewer Agent│
                                        └──────────────┘    └──────────────┘
                                                                   │
                                                                   ▼
                                                              最终输出
```

### 10.2 Agent Trait 抽象

```rust
// crates/tsangwu-agent-svc/src/agent.rs
use async_trait::async_trait;

/// Agent 统一 trait
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent 名称
    fn name(&self) -> &str;

    /// 执行 Agent 逻辑
    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError>;

    /// 补偿逻辑（回滚）
    async fn compensate(&self, ctx: &mut AgentContext) -> Result<(), AgentError> {
        Ok(()) // 默认无补偿
    }
}

/// Agent 执行上下文
pub struct AgentContext {
    pub task_id: Uuid,
    pub instance_id: Uuid,
    pub input: serde_json::Value,
    pub shared: SharedAgentState,
    pub progress_tx: tokio::sync::watch::Sender<ProgressUpdate>,
}

/// Agent 间共享状态
pub struct SharedAgentState {
    pub style_profile: serde_json::Value,
    pub character_refs: Vec<CharacterRef>,
    pub scene_refs: Vec<SceneRef>,
    pub cultural_labels: Vec<String>,
    pub dynasty: Option<String>,
}

#[derive(Debug)]
pub struct AgentOutput {
    pub data: serde_json::Value,
    pub artifacts: Vec<Artifact>,  // 生成的文件产物
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct Artifact {
    pub name: String,
    pub content_type: String,
    pub storage_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("模型调用失败: {0}")]
    ModelError(String),
    #[error("文化校验不通过: {0}")]
    CultureValidation(String),
    #[error("需要用户确认")]
    NeedUserApproval,
    #[error("超时")]
    Timeout,
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}
```

### 10.3 各 Agent 实现

#### Parser Agent（需求解析）

```rust
// crates/tsangwu-agent-svc/src/agents/parser.rs
pub struct ParserAgent {
    llm_client: Arc<LlmClient>,
    culture_svc: Arc<dyn CultureService>,
}

#[async_trait]
impl Agent for ParserAgent {
    fn name(&self) -> &str { "parser" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        let raw_input: ParserInput = serde_json::from_value(ctx.input.clone())?;

        // 1. 多模态输入预处理
        let text = match &raw_input.input_type {
            InputType::Text(t) => t.clone(),
            InputType::Image(urls) => self.llm_client.describe_images(urls).await?,
            InputType::Voice(url) => self.llm_client.transcribe(url).await?,
        };

        // 2. LLM 意图识别 + 结构化提取
        let prompt = format!(
            "分析以下用户需求，提取：场景类型、风格、朝代、文化元素、关键参数。\n\
             用户输入：{}\n用户模式：{:?}\n\
             输出 JSON 格式。",
            text, raw_input.user_mode
        );
        let structured: ParsedRequirement = self.llm_client
            .chat_json(&prompt)
            .await?;

        // 3. 文化语义增强（调用文化适配服务）
        let cultural_tags = self.culture_svc
            .identify_cultural_elements(&structured)
            .await?;

        // 4. 模式适配（零基础自动填充默认参数）
        let enriched = match raw_input.user_mode {
            UserMode::Beginner => structured.with_defaults(&cultural_tags),
            _ => structured,
        };

        ctx.shared.cultural_labels = cultural_tags.labels;
        ctx.shared.dynasty = enriched.dynasty.clone();

        Ok(AgentOutput {
            data: serde_json::to_value(&enriched)?,
            artifacts: vec![],
            metadata: serde_json::json!({ "cultural_tags": cultural_tags }),
        })
    }
}
```

#### Planner Agent（创意规划）

```rust
// crates/tsangwu-agent-svc/src/agents/planner.rs
pub struct PlannerAgent {
    llm_client: Arc<LlmClient>,
    asset_svc: Arc<dyn AssetService>,
    culture_svc: Arc<dyn CultureService>,
}

#[async_trait]
impl Agent for PlannerAgent {
    fn name(&self) -> &str { "planner" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        let req: ParsedRequirement = serde_json::from_value(ctx.input.clone())?;

        let plan = match req.scene_type {
            SceneType::Image => self.plan_image(ctx, &req).await?,
            SceneType::ShortDrama => self.plan_short_drama(ctx, &req).await?,
            SceneType::Film => self.plan_film(ctx, &req).await?,
            SceneType::Series => self.plan_series(ctx, &req).await?,
        };

        // 文化校验规划方案
        let validation = self.culture_svc.validate_plan(&plan).await?;
        if validation.has_critical_issues() {
            let revised = self.revise_plan(&plan, &validation).await?;
            return Ok(AgentOutput { data: serde_json::to_value(&revised)?, ..Default::default() });
        }

        Ok(AgentOutput {
            data: serde_json::to_value(&plan)?,
            artifacts: vec![],
            metadata: serde_json::json!({ "validation": validation }),
        })
    }
}

impl PlannerAgent {
    async fn plan_short_drama(&self, ctx: &AgentContext, req: &ParsedRequirement) -> Result<DramaPlan, AgentError> {
        // 1. 生成剧本（含台词/情绪/运镜标注）
        let script = self.llm_client.generate_script(req).await?;

        // 2. 分镜规划
        let storyboard = self.llm_client.generate_storyboard(&script).await?;

        // 3. 素材匹配（角色/场景/道具）
        let assets = self.asset_svc.match_assets(&req.cultural_labels, &script).await?;

        // 4. 配乐方案
        let music_plan = self.plan_music(&req.style, &script).await?;

        Ok(DramaPlan { script, storyboard, assets, music_plan })
    }
}
```

#### Generator Agent（内容生成）

```rust
// crates/tsangwu-agent-svc/src/agents/generator.rs
pub struct GeneratorAgent {
    pybridge: Arc<PyBridge>,       // PyO3 桥接 Python AI 模型
    storage: Arc<StorageService>,
}

#[async_trait]
impl Agent for GeneratorAgent {
    fn name(&self) -> &str { "generator" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        let plan: GenerationPlan = serde_json::from_value(ctx.input.clone())?;
        let mut artifacts = Vec::new();

        match plan.scene_type {
            SceneType::Image => {
                // 调用 Stable Diffusion（通过 PyO3）
                let params = ImageGenParams::from_plan(&plan);
                let image_bytes = self.pybridge.generate_image(&params).await?;

                let path = format!("gen/{}/{}.png", ctx.task_id, Uuid::new_v4());
                self.storage.upload(&path, image_bytes).await?;
                artifacts.push(Artifact { name: "main_image".into(), content_type: "image/png".into(), storage_path: path });
            }
            SceneType::ShortDrama => {
                let total_scenes = plan.storyboard.len();
                for (i, scene) in plan.storyboard.iter().enumerate() {
                    // 逐镜头生成视频片段
                    let video_bytes = self.pybridge.generate_video_segment(scene).await?;
                    let path = format!("gen/{}/scene_{:03}.mp4", ctx.task_id, i);
                    self.storage.upload(&path, video_bytes).await?;
                    artifacts.push(Artifact {
                        name: format!("scene_{:03}", i),
                        content_type: "video/mp4".into(),
                        storage_path: path,
                    });

                    // 更新进度
                    let progress = ((i + 1) as f32 / total_scenes as f32 * 80.0) as u8;
                    ctx.progress_tx.send(ProgressUpdate { progress, stage: "generating".into(), message: format!("生成第 {}/{} 个镜头", i + 1, total_scenes) })?;
                }
            }
            _ => { /* 电影/电视剧管线类似，增加更多步骤 */ }
        }

        Ok(AgentOutput { data: serde_json::json!({}), artifacts, metadata: serde_json::json!({}) })
    }
}
```

#### Editor Agent（编辑优化）与 Reviewer Agent（审片校验）

```rust
// crates/tsangwu-agent-svc/src/agents/editor.rs
pub struct EditorAgent {
    pybridge: Arc<PyBridge>,
    storage: Arc<StorageService>,
}

#[async_trait]
impl Agent for EditorAgent {
    fn name(&self) -> &str { "editor" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        let input: EditorInput = serde_json::from_value(ctx.input.clone())?;

        // 1. 画面精修（超分辨率、色调统一）
        let enhanced = self.pybridge.enhance_visuals(&input.artifacts).await?;

        // 2. 视频剪辑（片段拼接、转场特效）
        let edited = self.pybridge.edit_video(&enhanced, &input.transitions).await?;

        // 3. 音画同步（配音 + 配乐 + 音效混合）
        let final_output = self.pybridge.mix_audio_video(&edited, &input.audio_plan).await?;

        let path = format!("gen/{}/final.mp4", ctx.task_id);
        self.storage.upload(&path, final_output).await?;

        Ok(AgentOutput {
            data: serde_json::json!({}),
            artifacts: vec![Artifact { name: "final_output".into(), content_type: "video/mp4".into(), storage_path: path }],
            metadata: serde_json::json!({}),
        })
    }
}

// crates/tsangwu-agent-svc/src/agents/reviewer.rs
pub struct ReviewerAgent {
    culture_svc: Arc<dyn CultureService>,
    pybridge: Arc<PyBridge>,
}

#[async_trait]
impl Agent for ReviewerAgent {
    fn name(&self) -> &str { "reviewer" }

    async fn execute(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
        let input: ReviewInput = serde_json::from_value(ctx.input.clone())?;

        // 1. 一致性检查（角色外观跨镜头一致性）
        let consistency = self.pybridge.check_consistency(&input.artifacts).await?;

        // 2. 文化准确性校验
        let culture_check = self.culture_svc.validate_output(&input.artifacts, &ctx.shared).await?;

        // 3. 画面质量评估
        let quality = self.pybridge.assess_quality(&input.artifacts).await?;

        let needs_revision = consistency.score < 0.8 || culture_check.has_issues() || quality.score < 0.7;

        Ok(AgentOutput {
            data: serde_json::json!({
                "passed": !needs_revision,
                "needs_revision": needs_revision,
                "consistency_score": consistency.score,
                "culture_issues": culture_check.issues,
                "quality_score": quality.score,
                "revision_suggestions": culture_check.suggestions,
            }),
            artifacts: vec![],
            metadata: serde_json::json!({}),
        })
    }
}
```

### 10.4 Agent 注册与工作流集成

```rust
// crates/tsangwu-agent-svc/src/registry.rs
pub struct AgentRegistry {
    agents: HashMap<String, Arc<dyn Agent>>,
}

impl AgentRegistry {
    pub fn new(deps: &AgentDependencies) -> Self {
        let mut agents = HashMap::new();
        agents.insert("parser_agent".into(), Arc::new(ParserAgent::new(deps)) as Arc<dyn Agent>);
        agents.insert("planner_agent".into(), Arc::new(PlannerAgent::new(deps)) as Arc<dyn Agent>);
        agents.insert("generator_agent".into(), Arc::new(GeneratorAgent::new(deps)) as Arc<dyn Agent>);
        agents.insert("editor_agent".into(), Arc::new(EditorAgent::new(deps)) as Arc<dyn Agent>);
        agents.insert("reviewer_agent".into(), Arc::new(ReviewerAgent::new(deps)) as Arc<dyn Agent>);
        Self { agents }
    }
}

/// 将 Agent 注册为工作流步骤处理器
impl StepHandler for AgentStepAdapter {
    async fn execute(&self, ctx: &mut StepContext) -> Result<StepOutput, StepError> {
        let agent = self.registry.get(&ctx.step_name)?;
        let mut agent_ctx = AgentContext::from_step_context(ctx);
        let output = agent.execute(&mut agent_ctx).await?;
        Ok(StepOutput { data: output.data, artifacts: output.artifacts })
    }

    async fn compensate(&self, ctx: &mut StepContext) -> Result<(), StepError> {
        let agent = self.registry.get(&ctx.step_name)?;
        let mut agent_ctx = AgentContext::from_step_context(ctx);
        agent.compensate(&mut agent_ctx).await?;
        Ok(())
    }
}
```

### 10.5 Agent 间通信协议

Agent 间通过工作流引擎传递结构化消息，格式统一：

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentMessage {
    pub message_id: Uuid,
    pub from_agent: String,
    pub to_agent: String,
    pub task_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub payload_type: String,
    pub payload: serde_json::Value,
    pub context_ref: Option<String>,  // 文档存储中的上下文引用
}
```

共享上下文存储在文档存储（MongoDB / PG JSONB）的 `agent_contexts` 集合中，各 Agent 可读写共享的角色引用、场景引用、风格配置等信息。

---

## 11. 生成管线 Rust 实现（FFI + PyO3）

### 11.1 Python 桥接架构

苍梧的 AI 模型（Stable Diffusion、视频生成、TTS 等）主要基于 Python 生态。通过 PyO3 实现 Rust ↔ Python 双向调用，兼顾 Rust 的高性能调度与 Python 的 AI 模型生态。

```
┌──────────────────────────────────────────────────────────┐
│                    Rust 应用层                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ Agent 编排│  │ 任务调度  │  │ 进度追踪  │               │
│  └─────┬────┘  └─────┬────┘  └─────┬────┘               │
│        └─────────────┼─────────────┘                     │
│                      ▼                                    │
│  ┌───────────────────────────────────────┐               │
│  │        PyBridge (tsangwu-pybridge)     │               │
│  │        PyO3 + pyo3-asyncio             │               │
│  └───────────────────┬───────────────────┘               │
│                      │ GIL-aware 异步调用                  │
├──────────────────────┼───────────────────────────────────┤
│                      ▼                                    │
│  ┌───────────────────────────────────────┐               │
│  │          Python 运行时                  │               │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐│               │
│  │  │ SD 推理  │ │ 视频生成  │ │ TTS    ││               │
│  │  │ Pipeline │ │ Pipeline │ │ Engine ││               │
│  │  └──────────┘ └──────────┘ └────────┘│               │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐│               │
│  │  │ 超分辨率  │ │ 风格迁移  │ │ CLIP   ││               │
│  │  │ Model    │ │ Model    │ │ Encoder││               │
│  │  └──────────┘ └──────────┘ └────────┘│               │
│  └───────────────────────────────────────┘               │
└──────────────────────────────────────────────────────────┘
```

### 11.2 PyBridge 核心实现

```rust
// crates/tsangwu-pybridge/src/lib.rs
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tokio::sync::Semaphore;

/// Python AI 模型桥接层
pub struct PyBridge {
    /// GPU 并发控制信号量
    gpu_semaphore: Arc<Semaphore>,
    /// Python 模块缓存
    modules: tokio::sync::RwLock<HashMap<String, Py<PyModule>>>,
}

impl PyBridge {
    pub fn new(max_gpu_concurrent: usize) -> anyhow::Result<Self> {
        // 初始化 Python 解释器
        pyo3::prepare_freethreaded_python();
        Ok(Self {
            gpu_semaphore: Arc::new(Semaphore::new(max_gpu_concurrent)),
            modules: Default::default(),
        })
    }

    /// 图像生成（Stable Diffusion）
    pub async fn generate_image(&self, params: &ImageGenParams) -> Result<Vec<u8>, AgentError> {
        let params_json = serde_json::to_string(params)?;
        let _permit = self.gpu_semaphore.acquire().await?;

        // 在独立线程中执行 Python 调用（避免阻塞 Tokio 运行时）
        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> PyResult<Vec<u8>> {
                let module = py.import("tsangwu_ai.image_gen")?;
                let result = module.call_method1("generate", (params_json,))?;
                let bytes: Vec<u8> = result.extract()?;
                Ok(bytes)
            })
        }).await??;

        Ok(result)
    }

    /// 视频片段生成
    pub async fn generate_video_segment(&self, scene: &SceneSpec) -> Result<Vec<u8>, AgentError> {
        let scene_json = serde_json::to_string(scene)?;
        let _permit = self.gpu_semaphore.acquire().await?;

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> PyResult<Vec<u8>> {
                let module = py.import("tsangwu_ai.video_gen")?;
                let result = module.call_method1("generate_segment", (scene_json,))?;
                result.extract()
            })
        }).await??;

        Ok(result)
    }

    /// 画面增强（超分辨率 + 色调统一）
    pub async fn enhance_visuals(&self, artifacts: &[Artifact]) -> Result<Vec<Vec<u8>>, AgentError> {
        let paths_json = serde_json::to_string(artifacts)?;
        let _permit = self.gpu_semaphore.acquire().await?;

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> PyResult<Vec<Vec<u8>>> {
                let module = py.import("tsangwu_ai.enhance")?;
                let result = module.call_method1("enhance_batch", (paths_json,))?;
                result.extract()
            })
        }).await??;

        Ok(result)
    }

    /// TTS 语音合成
    pub async fn synthesize_speech(&self, text: &str, voice: &str) -> Result<Vec<u8>, AgentError> {
        let text = text.to_string();
        let voice = voice.to_string();

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> PyResult<Vec<u8>> {
                let module = py.import("tsangwu_ai.tts")?;
                let result = module.call_method1("synthesize", (text, voice))?;
                result.extract()
            })
        }).await??;

        Ok(result)
    }

    /// CLIP 语义编码（用于向量检索）
    pub async fn clip_encode(&self, input: ClipInput) -> Result<Vec<f32>, AgentError> {
        let input_json = serde_json::to_string(&input)?;

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| -> PyResult<Vec<f32>> {
                let module = py.import("tsangwu_ai.clip_encoder")?;
                let result = module.call_method1("encode", (input_json,))?;
                result.extract()
            })
        }).await??;

        Ok(result)
    }
}
```

### 11.3 Python 端模块结构

```
tsangwu_ai/                      # Python 包（pip install -e .）
├── __init__.py
├── image_gen.py                  # SD 图像生成
│   ├── generate(params_json)     # 主入口
│   ├── load_lora(style)          # 加载国风 LoRA
│   └── apply_controlnet(image)   # ControlNet 控制
├── video_gen.py                  # 视频生成
│   ├── generate_segment(scene)   # 逐镜头生成
│   └── interpolate(frames)       # 帧插值
├── enhance.py                    # 画面增强
│   ├── super_resolve(image)      # 超分辨率 (Real-ESRGAN)
│   ├── style_transfer(image)     # 风格迁移
│   └── enhance_batch(artifacts)  # 批量增强
├── tts.py                        # 语音合成
├── music_gen.py                  # 音乐生成
├── clip_encoder.py               # CLIP 语义编码
├── video_edit.py                 # 视频剪辑
│   ├── edit_video(segments)      # 片段拼接 + 转场
│   └── mix_audio_video(v, a)     # 音画合成
└── models/                       # 模型权重管理
    ├── model_registry.py
    └── download.py
```

### 11.4 四条生成管线

```
                    ┌─────────────────────────────────────────┐
                    │            统一入口 (gen-svc)             │
                    └──────────┬──────────┬──────────┬────────┘
                               │          │          │
              ┌────────────────┼──────────┼──────────┼────────────────┐
              ▼                ▼          ▼          ▼                │
        ┌──────────┐   ┌──────────┐ ┌──────────┐ ┌──────────┐       │
        │ 图文管线  │   │ 短剧管线  │ │ 电影管线  │ │电视剧管线 │       │
        │ Image    │   │ Short   │ │ Film    │ │ Series  │       │
        │ Pipeline │   │ Drama   │ │ Pipeline│ │ Pipeline│       │
        └──────────┘   └──────────┘ └──────────┘ └──────────┘       │
              │                │          │          │                │
              └────────────────┴──────────┴──────────┘                │
                               │  共享组件                            │
                    ┌──────────┴──────────┐                          │
                    │ 文化适配引擎 | 素材库  │                          │
                    │ 角色一致性 | 音频引擎  │                          │
                    └─────────────────────┘                          │
```

### 11.5 图文生成管线

```
输入(文本/图片/模板) → 需求解析 → 构图规划 → 图像生成 → 后处理 → 输出
                         │           │           │          │
                    文化语义识别   中式构图引擎  SD+LoRA   色调/水印
                    风格匹配      元素布局      ControlNet  格式转换
```

| 阶段 | 处理内容 | Rust/Python 分工 |
|------|----------|-----------------|
| 需求解析 | 提取主题/风格/元素/色调 | Rust: LLM API 调用；Python: CLIP 图像理解 |
| 构图规划 | 生成构图方案 | Rust: 规则引擎 + LLM 布局生成 |
| 图像生成 | 生成主体图像 | Python: SD + 国风 LoRA + ControlNet |
| 后处理 | 色调统一、文字排版、水印 | Rust: image crate 处理；Python: 国风色调 LUT |
| 输出 | 多格式/多分辨率导出 | Rust: image crate 格式转换 |

### 11.6 短剧生成管线

| 阶段 | 处理内容 | 耗时预估（1min 内容） | Rust/Python 分工 |
|------|----------|---------------------|-----------------|
| 剧本生成 | 含台词/情绪/运镜标注的剧本 | ~30s | Rust: LLM API 调用 |
| 分镜规划 | 逐镜头分镜图 + 参数标注 | ~60s | Rust: LLM + 规则引擎 |
| 素材准备 | 角色模型生成/检索 + 场景匹配 | ~30s | Rust: 检索调度；Python: 角色生成 |
| 视频生成 | 逐镜头视频片段生成 | ~120s | Python: 视频生成模型 |
| 音频合成 | 配音(TTS) + 配乐 + 音效 | ~30s | Python: TTS + 音乐生成 |
| 剪辑合成 | 片段拼接 + 转场 + 音画同步 | ~20s | Rust: FFmpeg 绑定 (ffmpeg-next) |
| 审片校验 | 一致性/文化准确性/质量检查 | ~10s | Rust: 规则引擎；Python: 模型评估 |

### 11.7 电影/电视剧管线增强

| 增强点 | 电影管线 | 电视剧管线 |
|--------|---------|-----------|
| 剧本 | 90-120 分钟三幕式结构 | 多集剧本大纲 + 单集剧本 |
| 分镜 | 电影级运镜（航拍/长镜头/慢镜头） | 多集批量分镜，统一风格 |
| 生成 | 基础 1080p + 超分至 4K | 优先核心片段（名场面） |
| 特效 | 独立特效合成（剑气/水墨过渡） | 跨集角色状态追踪 |
| 协同 | 按场次分段，支持单场重新生成 | 多人分集编辑，冲突检测 |

### 11.8 管线共享组件

| 组件 | 功能 | Rust 实现 |
|------|------|----------|
| 角色一致性引擎 | 跨镜头/跨集角色外观一致 | PyO3 调用 IP-Adapter + 角色特征向量锁定 |
| 音频引擎 | TTS 配音 + 国风配乐 + 音效 | PyO3 调用语音合成 + 音乐生成模型 |
| 超分辨率模块 | 1080p → 4K 提升 | PyO3 调用 Real-ESRGAN / SwinIR |
| 视频编码器 | 格式转换与压缩 | ffmpeg-next (Rust FFmpeg 绑定) + 硬件加速 |
| 进度追踪器 | 实时进度计算与推送 | Rust: tokio::sync::watch + WebSocket |

### 11.9 GPU 资源管理

```rust
// crates/tsangwu-pybridge/src/gpu.rs
/// GPU 资源池管理
pub struct GpuResourcePool {
    /// 每个 GPU 设备的信号量
    devices: Vec<GpuDevice>,
    /// 任务优先级队列
    queue: Arc<tokio::sync::Mutex<BinaryHeap<GpuTask>>>,
}

pub struct GpuDevice {
    pub id: usize,
    pub memory_total: u64,
    pub semaphore: Semaphore,  // 控制单 GPU 并发任务数
}

impl GpuResourcePool {
    /// 申请 GPU 资源（按优先级排队）
    pub async fn acquire(&self, task: &GpuTask) -> Result<GpuPermit, AgentError> {
        // 选择负载最低的 GPU
        let device = self.select_device().await?;
        let permit = device.semaphore.acquire().await?;
        Ok(GpuPermit { device_id: device.id, _permit: permit })
    }
}
```

---

## 12. 文化适配引擎 Rust 实现

### 12.1 引擎架构

```
                    ┌─────────────────────────────┐
                    │       文化适配引擎            │
                    │      culture-svc             │
                    ├──────────┬──────────┬────────┤
                    │ 识别模块  │ 校验模块  │ 匹配模块│
                    └────┬─────┴────┬─────┴───┬────┘
                         │          │         │
                    ┌────▼────┐ ┌───▼────┐ ┌──▼──────┐
                    │文化知识库│ │校验规则库│ │风格模型库│
                    │(文档存储)│ │(规则引擎)│ │(Milvus) │
                    └─────────┘ └────────┘ └─────────┘
```

### 12.2 文化识别模块

```rust
// crates/tsangwu-culture-svc/src/identify.rs

/// 文化识别服务
pub struct CultureIdentifier {
    llm_client: Arc<LlmClient>,
    knowledge_store: Arc<dyn DocumentStore>,
    pybridge: Arc<PyBridge>,
}

/// 文化标签集
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CulturalTags {
    pub dynasty: Option<String>,
    pub style: Vec<String>,          // 水墨风/工笔画/敦煌风/武侠风
    pub elements: Vec<CulturalElement>,
    pub narrative_type: Option<String>, // 家国情怀/江湖侠义/古典爱情
    pub color_palette: Vec<String>,    // 黛青/朱红/鎏金
    pub labels: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CulturalElement {
    pub category: String,   // costume/scene/prop/pattern
    pub name: String,
    pub dynasty: Option<String>,
    pub confidence: f32,
}

impl CultureIdentifier {
    /// 从用户输入中识别文化元素
    pub async fn identify(&self, input: &IdentifyInput) -> Result<CulturalTags, AppError> {
        // 1. 文本文化关键词提取（LLM NER）
        let text_tags = self.extract_from_text(&input.text).await?;

        // 2. 图片文化元素识别（CLIP + 分类模型）
        let image_tags = if let Some(images) = &input.images {
            self.extract_from_images(images).await?
        } else {
            Default::default()
        };

        // 3. 合并与去重
        let merged = self.merge_tags(text_tags, image_tags);

        // 4. 知识库增强（补充关联信息）
        let enriched = self.enrich_from_knowledge(&merged).await?;

        Ok(enriched)
    }

    async fn extract_from_text(&self, text: &str) -> Result<CulturalTags, AppError> {
        let prompt = format!(
            "从以下文本中识别中国传统文化元素，输出 JSON：\n\
             - dynasty: 朝代\n- style: 艺术风格\n- elements: 文化元素列表\n\
             - narrative_type: 叙事类型\n- color_palette: 色彩体系\n\n\
             文本：{}", text
        );
        self.llm_client.chat_json(&prompt).await
    }

    async fn extract_from_images(&self, images: &[String]) -> Result<CulturalTags, AppError> {
        // 通过 PyO3 调用 CLIP + 文化分类模型
        let embeddings = self.pybridge.clip_encode(ClipInput::Images(images.to_vec())).await?;
        // 在知识库向量空间中检索最近邻
        let matches = self.knowledge_store.vector_search("cultural_knowledge", &embeddings, 10).await?;
        // 聚合匹配结果为文化标签
        Ok(CulturalTags::from_matches(&matches))
    }
}
```

### 12.3 文化校验模块（规则引擎）

```rust
// crates/tsangwu-culture-svc/src/validate.rs

/// 校验规则严重级别
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,       // 建议
    Warning,    // 警告（建议修改）
    Critical,   // 严重（必须修改）
}

/// 校验结果
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationIssue {
    pub rule_id: String,
    pub category: String,     // costume/architecture/etiquette/prop/color/pattern
    pub severity: Severity,
    pub message: String,
    pub suggestion: String,
}

impl ValidationResult {
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Critical)
    }
    pub fn has_issues(&self) -> bool { !self.issues.is_empty() }
}

/// 文化校验引擎
pub struct CultureValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    knowledge_store: Arc<dyn DocumentStore>,
}

/// 校验规则 trait
#[async_trait]
pub trait ValidationRule: Send + Sync {
    fn rule_id(&self) -> &str;
    fn category(&self) -> &str;
    async fn validate(&self, ctx: &ValidationContext) -> Vec<ValidationIssue>;
}

/// 服饰形制校验规则
pub struct CostumeValidationRule {
    knowledge_store: Arc<dyn DocumentStore>,
}

#[async_trait]
impl ValidationRule for CostumeValidationRule {
    fn rule_id(&self) -> &str { "costume_dynasty_match" }
    fn category(&self) -> &str { "costume" }

    async fn validate(&self, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if let (Some(dynasty), Some(costumes)) = (&ctx.dynasty, &ctx.costumes) {
            for costume in costumes {
                // 查询知识库：该服饰是否属于该朝代
                let knowledge = self.knowledge_store.find_one::<CulturalKnowledge>(
                    "cultural_knowledge",
                    serde_json::json!({ "name": costume, "category": "costume" }),
                ).await;

                if let Ok(Some(k)) = knowledge {
                    if k.dynasty != *dynasty {
                        issues.push(ValidationIssue {
                            rule_id: self.rule_id().into(),
                            category: self.category().into(),
                            severity: Severity::Critical,
                            message: format!("{}属于{}，与设定朝代{}不符", costume, k.dynasty, dynasty),
                            suggestion: format!("建议使用{}时期的服饰：{}", dynasty, k.alternatives.join("、")),
                        });
                    }
                    // 检查禁止组合
                    for forbidden in &k.attributes.forbidden_combinations {
                        if costumes.contains(forbidden) {
                            issues.push(ValidationIssue {
                                rule_id: "costume_forbidden_combo".into(),
                                category: self.category().into(),
                                severity: Severity::Critical,
                                message: format!("{}不应与{}同时出现", costume, forbidden),
                                suggestion: format!("请移除{}", forbidden),
                            });
                        }
                    }
                }
            }
        }
        issues
    }
}

impl CultureValidator {
    pub fn new(knowledge_store: Arc<dyn DocumentStore>) -> Self {
        let ks = knowledge_store.clone();
        Self {
            rules: vec![
                Box::new(CostumeValidationRule { knowledge_store: ks.clone() }),
                Box::new(ArchitectureValidationRule { knowledge_store: ks.clone() }),
                Box::new(PropValidationRule { knowledge_store: ks.clone() }),
                Box::new(ColorValidationRule { knowledge_store: ks.clone() }),
                Box::new(PatternValidationRule { knowledge_store: ks }),
            ],
            knowledge_store,
        }
    }

    pub async fn validate(&self, ctx: &ValidationContext) -> ValidationResult {
        let mut all_issues = Vec::new();
        for rule in &self.rules {
            let issues = rule.validate(ctx).await;
            all_issues.extend(issues);
        }
        all_issues.sort_by(|a, b| b.severity.cmp(&a.severity));
        let passed = !all_issues.iter().any(|i| i.severity == Severity::Critical);
        ValidationResult { passed, issues: all_issues, suggestions: vec![] }
    }
}
```

### 12.4 朝代特征矩阵

| 朝代 | 服饰特征 | 建筑特征 | 色彩偏好 | 典型元素 |
|------|----------|----------|----------|----------|
| 汉 | 曲裾深衣、直裾 | 高台建筑、阙楼 | 玄黑、朱红 | 玉璧、博山炉 |
| 唐 | 齐胸襦裙、圆领袍 | 斗拱飞檐、大明宫式 | 鎏金、石榴红 | 团扇、铜镜 |
| 宋 | 褙子、直裰 | 素雅园林、瓦舍 | 天青、月白 | 折扇、香炉 |
| 明 | 立领袄裙、飞鱼服 | 硬山顶、牌坊 | 大红、明黄 | 补子、玉带 |
| 清 | 旗装、马褂 | 满式宫殿、圆明园式 | 宝蓝、明黄 | 朝珠、如意 |

### 12.5 文化匹配模块

```rust
// crates/tsangwu-culture-svc/src/matching.rs

pub struct CultureMatcher {
    pybridge: Arc<PyBridge>,
    asset_svc: Arc<dyn AssetService>,
    knowledge_store: Arc<dyn DocumentStore>,
}

impl CultureMatcher {
    /// 根据文化标签匹配最优素材、模板与风格参数
    pub async fn match_resources(
        &self,
        tags: &CulturalTags,
    ) -> Result<MatchResult, AppError> {
        // 1. 文化标签向量编码
        let tag_text = tags.to_search_text();
        let embedding = self.pybridge.clip_encode(ClipInput::Text(tag_text)).await?;

        // 2. 并行检索：素材 + 模板 + 风格参数
        let (assets, templates, style_params) = tokio::try_join!(
            self.asset_svc.vector_search(&embedding, 20),
            self.match_templates(tags),
            self.match_style_params(tags),
        )?;

        // 3. 融合排序
        Ok(MatchResult { assets, templates, style_params })
    }

    async fn match_style_params(&self, tags: &CulturalTags) -> Result<StyleParams, AppError> {
        // 根据朝代和风格查询预设风格参数（LoRA 权重、色调 LUT、运镜模板）
        let dynasty = tags.dynasty.as_deref().unwrap_or("general");
        let style = tags.style.first().map(|s| s.as_str()).unwrap_or("ink_wash");

        let params = self.knowledge_store.find_one::<StyleParams>(
            "style_presets",
            serde_json::json!({ "dynasty": dynasty, "style": style }),
        ).await?.unwrap_or_default();

        Ok(params)
    }
}
```

---

## 13. 部署与运维（Docker/K8s/本地一键启动）

### 13.1 本地一键启动

```bash
#!/bin/bash
# deploy/scripts/local-setup.sh
# 苍梧本地模式一键启动脚本

set -e

TSANGWU_VERSION="${1:-latest}"
DATA_DIR="${TSANGWU_DATA_DIR:-$HOME/.tsangwu}"
PORT="${TSANGWU_PORT:-8080}"

echo "=== 苍梧系统本地模式启动 ==="

# 1. 下载预编译二进制（如未安装）
if ! command -v tsangwu-local &> /dev/null; then
    echo "下载 tsangwu-local..."
    curl -fsSL https://releases.tsangwu.com/$TSANGWU_VERSION/tsangwu-local-$(uname -s)-$(uname -m) \
        -o /usr/local/bin/tsangwu-local
    chmod +x /usr/local/bin/tsangwu-local
fi

# 2. 初始化数据目录
mkdir -p "$DATA_DIR"/{db,storage,logs}

# 3. 启动（自动创建 SQLite 数据库 + 运行迁移）
TSANGWU_CONFIG=local \
TSANGWU_DATA_DIR="$DATA_DIR" \
TSANGWU_PORT="$PORT" \
    tsangwu-local

# 访问 http://localhost:8080
```

### 13.2 Docker 部署

#### 单体模式 Dockerfile

```dockerfile
# deploy/docker/Dockerfile.server
# 多阶段构建，最终镜像极小
FROM rust:1.84-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin tsangwu-server \
    --features "postgres,kafka,redis,s3"

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libpq5 python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# 安装 Python AI 依赖
COPY tsangwu_ai/requirements.txt /opt/tsangwu_ai/
RUN pip3 install --no-cache-dir -r /opt/tsangwu_ai/requirements.txt
COPY tsangwu_ai/ /opt/tsangwu_ai/

COPY --from=builder /build/target/release/tsangwu-server /usr/local/bin/
COPY config/ /etc/tsangwu/

ENV TSANGWU_CONFIG=standalone
ENV PYTHONPATH=/opt/tsangwu_ai
EXPOSE 8080 9090
HEALTHCHECK CMD curl -f http://localhost:8080/healthz || exit 1
ENTRYPOINT ["tsangwu-server"]
```

#### Docker Compose 本地开发环境

```yaml
# deploy/docker/docker-compose.yml
version: "3.9"
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: tsangwu
      POSTGRES_USER: tsangwu
      POSTGRES_PASSWORD: tsangwu_dev
    ports: ["5432:5432"]
    volumes: ["pg_data:/var/lib/postgresql/data"]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  kafka:
    image: bitnami/kafka:3.7
    environment:
      KAFKA_CFG_NODE_ID: 0
      KAFKA_CFG_PROCESS_ROLES: controller,broker
      KAFKA_CFG_CONTROLLER_QUORUM_VOTERS: 0@kafka:9093
      KAFKA_CFG_LISTENERS: PLAINTEXT://:9092,CONTROLLER://:9093
      KAFKA_CFG_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_CFG_CONTROLLER_LISTENER_NAMES: CONTROLLER
    ports: ["9092:9092"]

  mongodb:
    image: mongo:7
    ports: ["27017:27017"]
    volumes: ["mongo_data:/data/db"]

  elasticsearch:
    image: elasticsearch:8.15.0
    environment:
      discovery.type: single-node
      xpack.security.enabled: "false"
      ES_JAVA_OPTS: "-Xms512m -Xmx512m"
    ports: ["9200:9200"]

  milvus:
    image: milvusdb/milvus:v2.4-latest
    ports: ["19530:19530"]
    volumes: ["milvus_data:/var/lib/milvus"]

  tsangwu:
    build:
      context: ../..
      dockerfile: deploy/docker/Dockerfile.server
    depends_on: [postgres, redis, kafka, mongodb, elasticsearch]
    environment:
      TSANGWU_CONFIG: standalone
      DATABASE_URL: postgres://tsangwu:tsangwu_dev@postgres:5432/tsangwu
      REDIS_URL: redis://redis:6379
      KAFKA_BROKERS: kafka:9092
      MONGODB_URL: mongodb://mongodb:27017/tsangwu
      ES_URL: http://elasticsearch:9200
    ports:
      - "8080:8080"
      - "9090:9090"

volumes:
  pg_data:
  mongo_data:
  milvus_data:
```

### 13.3 Kubernetes 分布式部署

#### 目录结构（Kustomize）

```
deploy/k8s/
├── base/
│   ├── kustomization.yaml
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── user-svc/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   └── hpa.yaml
│   ├── auth-svc/
│   ├── project-svc/
│   ├── asset-svc/
│   ├── gen-svc/
│   ├── agent-svc/
│   ├── culture-svc/
│   ├── gateway/
│   └── ...
└── overlays/
    ├── dev/
    │   ├── kustomization.yaml
    │   └── patches/
    ├── staging/
    │   ├── kustomization.yaml
    │   └── patches/
    └── prod/
        ├── kustomization.yaml
        ├── patches/
        └── secrets/
```

#### 微服务 Deployment 示例

```yaml
# deploy/k8s/base/gen-svc/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gen-svc
  namespace: tsangwu
  labels:
    app: gen-svc
    version: v1
spec:
  replicas: 3
  selector:
    matchLabels:
      app: gen-svc
  strategy:
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: gen-svc
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9100"
    spec:
      containers:
      - name: gen-svc
        image: registry.tsangwu.com/gen-svc:v1.0.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: grpc
        - containerPort: 9100
          name: metrics
        env:
        - name: TSANGWU_CONFIG
          value: distributed
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: tsangwu-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: tsangwu-secrets
              key: redis-url
        - name: KAFKA_BROKERS
          valueFrom:
            configMapKeyRef:
              name: tsangwu-config
              key: kafka-brokers
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 15
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

#### HPA 弹性伸缩

```yaml
# deploy/k8s/base/gen-svc/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: gen-svc-hpa
  namespace: tsangwu
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: gen-svc
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Pods
    pods:
      metric:
        name: kafka_consumer_lag
      target:
        type: AverageValue
        averageValue: "100"
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Pods
        value: 3
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Pods
        value: 1
        periodSeconds: 120
```

### 13.4 K8s 集群规划

| 节点池 | 机型 | 数量 | 用途 |
|--------|------|------|------|
| 业务节点池 | ecs.g7.2xlarge (8C32G) | 6-20 | 业务微服务 |
| GPU 推理池 | ecs.gn7i-c16g1.4xlarge (A10) | 4-16 | 模型推理（常规） |
| GPU 训练池 | ecs.gn7-c12g1.3xlarge (A100) | 2-8 | 高精度生成/微调 |
| 中间件节点池 | ecs.r7.2xlarge (8C64G) | 3-6 | Kafka/ES/Milvus |

### 13.5 CI/CD 流水线

```
代码提交 → GitLab CI → 单元测试 → clippy 检查 → cargo build → 构建镜像 → 推送 Registry
                          │            │                                        │
                     cargo test   cargo clippy                                  ▼
                                  cargo fmt --check                    ArgoCD 监听 → 自动同步
                                                                                │
                                                                ┌───────────────┼───────────┐
                                                                ▼               ▼           ▼
                                                             dev 环境       staging 环境   prod 环境
                                                            (自动部署)     (自动部署)    (审批后灰度)
```

```yaml
# .gitlab-ci.yml 核心阶段
stages: [check, test, build, deploy]

check:
  stage: check
  script:
    - cargo fmt --all -- --check
    - cargo clippy --all-targets --all-features -- -D warnings

test:
  stage: test
  script:
    - cargo test --all --all-features
  services:
    - postgres:16-alpine
    - redis:7-alpine

build:
  stage: build
  script:
    - cargo build --release --bin tsangwu-server
    - docker build -t registry.tsangwu.com/tsangwu-server:$CI_COMMIT_SHA .
    - docker push registry.tsangwu.com/tsangwu-server:$CI_COMMIT_SHA
  only: [main, release/*]

deploy-prod:
  stage: deploy
  script:
    - kustomize edit set image registry.tsangwu.com/tsangwu-server:$CI_COMMIT_TAG
    - git push  # ArgoCD 自动同步
  only: [tags]
  when: manual
```

### 13.6 GPU 弹性伸缩策略

| 策略 | 说明 |
|------|------|
| 预热池 | 保持 2 个 GPU 节点常驻，避免冷启动延迟 |
| 定时扩容 | 工作日 9:00-22:00 自动扩容至基准的 1.5 倍 |
| 突发扩容 | 队列深度超阈值时 3 分钟内扩容新节点 |
| 缩容冷却 | 缩容前等待 15 分钟，避免频繁伸缩 |

---

## 14. 安全与性能

### 14.1 安全架构总览

```
┌─────────────────────────────────────────────────────────┐
│                     安全防护体系                          │
├──────────────┬──────────────┬──────────────┬────────────┤
│   网络安全    │   数据安全    │   应用安全    │  内容安全   │
├──────────────┼──────────────┼──────────────┼────────────┤
│ · WAF 防护   │ · 传输加密   │ · 身份认证   │ · 内容审核  │
│ · DDoS 防御  │ · 存储加密   │ · 权限控制   │ · 版权校验  │
│ · VPC 隔离   │ · 数据脱敏   │ · API 限流   │ · 敏感词过滤│
│ · 安全组     │ · 备份恢复   │ · 注入防护   │ · 人工复审  │
│              │              │ · Rust 内存安全│            │
└──────────────┴──────────────┴──────────────┴────────────┘
```

### 14.2 Rust 语言级安全优势

| 安全维度 | Rust 保障 | 对比 Go |
|----------|----------|---------|
| 内存安全 | 所有权系统 + 借用检查器，编译期消除 UAF/双重释放/缓冲区溢出 | GC 管理，运行时开销 |
| 线程安全 | Send/Sync trait 编译期保证无数据竞争 | goroutine 需手动加锁 |
| 空指针 | Option\<T\> 强制处理，无 null | nil panic 风险 |
| 错误处理 | Result\<T, E\> 强制处理，无未捕获异常 | error 可被忽略 |
| 类型安全 | 强类型 + 泛型，编译期校验 | interface{} 运行时断言 |

### 14.3 传输安全

| 场景 | 方案 |
|------|------|
| 客户端 ↔ 服务端 | TLS 1.3，强制 HTTPS（rustls 纯 Rust TLS 实现） |
| 服务间通信 | mTLS（Tonic 内置 TLS 支持）；分布式模式下通过 Istio 管理 |
| 数据库连接 | SSL 加密连接（SQLx 原生支持 rustls） |

### 14.4 存储安全

| 数据类型 | 加密方案 | Rust 实现 |
|----------|----------|----------|
| 用户密码 | Argon2id (OWASP 推荐) | `argon2` crate |
| 敏感字段（手机号/邮箱） | AES-256-GCM 应用层加密 | `aes-gcm` crate，密钥由 KMS 托管 |
| OSS 对象 | 服务端加密 (SSE-KMS) | OpenDAL 配置 |
| 数据库 | TDE 透明数据加密 | 数据库层面配置 |

### 14.5 数据备份

| 数据源 | 备份策略 | 保留周期 |
|--------|----------|----------|
| PostgreSQL | 每日全量 + 实时 WAL 归档 | 全量 30 天，WAL 7 天 |
| MongoDB | 每日全量 + Oplog 持续备份 | 30 天 |
| Redis | RDB 每日 + AOF 实时 | 7 天 |
| OSS | 跨区域复制 (CRR) | 永久 |
| SQLite（本地模式） | 定时文件备份 | 用户自行管理 |

### 14.6 API 限流

```rust
// crates/tsangwu-auth/src/rate_limit.rs
use tower::ServiceBuilder;

/// 基于用户套餐的分级限流
pub struct RateLimitConfig {
    pub limits: HashMap<String, PlanLimit>,
}

pub struct PlanLimit {
    pub requests_per_minute: u32,
    pub requests_per_day: u32,
    pub gen_tasks_per_day: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert("free".into(), PlanLimit { requests_per_minute: 30, requests_per_day: 1000, gen_tasks_per_day: 5 });
        limits.insert("personal".into(), PlanLimit { requests_per_minute: 60, requests_per_day: 5000, gen_tasks_per_day: 50 });
        limits.insert("advanced".into(), PlanLimit { requests_per_minute: 120, requests_per_day: 20000, gen_tasks_per_day: 200 });
        limits.insert("pro".into(), PlanLimit { requests_per_minute: 300, requests_per_day: 100000, gen_tasks_per_day: 1000 });
        limits.insert("enterprise".into(), PlanLimit { requests_per_minute: 600, requests_per_day: 500000, gen_tasks_per_day: 5000 });
        Self { limits }
    }
}

/// Tower 限流中间件
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let key = format!("rate_limit:{}:{}", claims.sub, chrono::Utc::now().format("%Y%m%d%H%M"));
    let count: u64 = state.cache.incr(&key, 60).await?;
    let limit = state.rate_config.limits.get(&claims.plan).unwrap_or(&PlanLimit::default());

    if count > limit.requests_per_minute as u64 {
        return Err(AppError::RateLimited);
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", limit.requests_per_minute.into());
    headers.insert("X-RateLimit-Remaining", (limit.requests_per_minute as u64 - count).into());
    Ok(response)
}
```

### 14.7 内容审核

```
生成内容 → 机器审核 → 判定
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
            通过     需人工复审    拒绝
              │         │         │
              ▼         ▼         ▼
           直接输出   人工审核队列  拦截+通知用户
```

| 维度 | 检测内容 | 处理方式 |
|------|----------|----------|
| 涉政 | 敏感政治人物/事件/标语 | 拦截 |
| 涉黄 | 色情/低俗内容 | 拦截 |
| 暴力 | 血腥/恐怖/极端暴力 | 拦截 |
| 违禁 | 毒品/武器/赌博相关 | 拦截 |
| 侵权 | 明星肖像/品牌商标 | 人工复审 |
| 文化敏感 | 宗教/民族/历史敏感内容 | 人工复审 |

### 14.8 版权保护体系

```
┌─────────────────────────────────────────┐
│              版权保护全链路               │
├───────────┬───────────┬─────────────────┤
│  素材入库   │  生成过程  │   输出分发       │
├───────────┼───────────┼─────────────────┤
│ 来源校验   │ 引用追踪   │ 侵权比对        │
│ 授权确认   │ 素材溯源   │ 版权证明生成     │
│ 版权标记   │ 水印嵌入   │ 商用授权确认     │
└───────────┴───────────┴─────────────────┘
```

- 数字水印：生成内容嵌入不可见水印（DWT 域水印），含任务 ID/用户 ID/时间戳
- 版权证明：商用版自动生成版权证明文件（PDF），含生成参数、素材来源、授权信息
- 侵权检测：基于感知哈希 + 深度特征的相似度检测，对接第三方版权库

### 14.9 合规要求

| 法规 | 合规措施 |
|------|----------|
| 《数据安全法》 | 数据分级分类管理，重要数据境内存储 |
| 《个人信息保护法》 | 最小必要原则采集，用户授权管理，数据可携带/可删除 |
| 《生成式AI管理办法》 | 内容审核前置，训练数据合规审查，生成内容标识 |
| 《网络安全法》 | 等保三级，安全事件应急响应，日志留存 ≥ 6 个月 |

### 14.10 性能优化策略

| 优化维度 | 策略 | Rust 实现 |
|----------|------|----------|
| 零拷贝序列化 | serde 零拷贝反序列化 + bytes crate | `#[serde(borrow)]` 避免内存分配 |
| 连接池复用 | 数据库/Redis/HTTP 连接池 | SQLx Pool / fred Pool / reqwest Pool |
| 异步 I/O | 全链路 Tokio 异步，无阻塞线程 | `async/await` + `spawn_blocking` 隔离 CPU 密集任务 |
| 内存管理 | 无 GC 停顿，确定性内存释放 | 所有权系统自动管理 |
| 编译优化 | LTO + codegen-units=1 + strip | `Cargo.toml [profile.release]` |
| 缓存策略 | 多级缓存（进程内 moka → Redis → DB） | 热点数据本地缓存，减少网络往返 |

```toml
# Cargo.toml 发布构建优化
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

---

## 15. 监控与可观测性（OpenTelemetry Rust SDK）

### 15.1 可观测性架构

Rust 生态采用 `tracing` 作为统一的日志 + 追踪框架，通过 OpenTelemetry 导出到后端。

```
┌─────────────────────────────────────────────────────────┐
│                    可观测性平台                           │
├──────────────────┬──────────────────┬───────────────────┤
│      日志         │      指标         │     链路追踪      │
│    (Logging)     │    (Metrics)     │    (Tracing)     │
├──────────────────┼──────────────────┼───────────────────┤
│ tracing crate    │ metrics crate    │ tracing crate     │
│ + tracing-sub    │ + prometheus     │ + tracing-otel    │
│   ┌──────────┐   │   exporter       │   ┌──────────┐   │
│   │ JSON 格式 │   │   ┌──────────┐  │   │ OTLP     │   │
│   │ 输出      │   │   │ /metrics │  │   │ Exporter │   │
│   └────┬─────┘   │   │ endpoint │  │   └────┬─────┘   │
│        │         │   └────┬─────┘  │        │         │
│        ▼         │        ▼        │        ▼         │
│   Filebeat/      │   Prometheus    │   OTel Collector  │
│   Vector         │   Scrape       │        │         │
│        │         │        │        │        ▼         │
│        ▼         │        ▼        │     Jaeger /     │
│   Elasticsearch  │   Grafana      │     Tempo        │
│   + Kibana       │                │                   │
└──────────────────┴──────────────────┴───────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   告警中心    │
                    │ Alertmanager │
                    │ + 飞书/钉钉   │
                    └──────────────┘
```

### 15.2 tracing 初始化

```rust
// crates/tsangwu-common/src/telemetry.rs
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_telemetry(config: &TelemetryConfig) -> anyhow::Result<()> {
    // 1. OpenTelemetry Tracer（OTLP 导出到 Jaeger/Tempo）
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint)
        .build()?;

    let tracer_provider = opentelemetry::sdk::trace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(opentelemetry::sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .build();

    let tracer = tracer_provider.tracer(config.service_name.clone());
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // 2. 日志层（JSON 结构化输出）
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // 3. 环境过滤器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // 4. 组合所有层
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(())
}

/// 优雅关闭（刷新未发送的 span）
pub fn shutdown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
}
```

### 15.3 业务埋点示例

```rust
use tracing::{instrument, info, warn, error};

/// 自动记录函数调用为 span，包含参数
#[instrument(skip(state), fields(user_id = %claims.sub))]
pub async fn submit_generation(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<i64>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<ApiResponse<GenerateResponse>>, AppError> {
    info!(scene_type = %req.scene_type, mode = %req.mode, "收到生成任务请求");

    let task = create_task(&state, &claims, project_id, &req).await?;
    info!(task_id = %task.uid, "生成任务已创建");

    state.mq.publish("generation.tasks", &task.uid.to_string(), &serde_json::to_vec(&task)?).await
        .map_err(|e| {
            error!(task_id = %task.uid, error = %e, "消息队列发送失败");
            AppError::Internal(e.into())
        })?;

    Ok(Json(ApiResponse::ok(GenerateResponse { task_id: task.uid, status: "queued".into(), .. })))
}

/// Agent 执行追踪
#[instrument(skip(self, ctx), fields(agent = %self.name(), task_id = %ctx.task_id))]
async fn execute_agent(&self, ctx: &mut AgentContext) -> Result<AgentOutput, AgentError> {
    let start = std::time::Instant::now();
    let result = self.execute(ctx).await;
    let duration = start.elapsed();

    match &result {
        Ok(_) => info!(duration_ms = duration.as_millis(), "Agent 执行成功"),
        Err(e) => warn!(duration_ms = duration.as_millis(), error = %e, "Agent 执行失败"),
    }

    // 记录 Prometheus 指标
    metrics::counter!("agent_executions_total", "agent" => self.name(), "status" => if result.is_ok() { "success" } else { "failure" }).increment(1);
    metrics::histogram!("agent_execution_duration_seconds", "agent" => self.name()).record(duration.as_secs_f64());

    result
}
```

### 15.4 日志格式（结构化 JSON）

```json
{
    "timestamp": "2026-02-17T10:30:00.123Z",
    "level": "INFO",
    "target": "tsangwu_gen_svc::handlers",
    "span": {
        "name": "submit_generation",
        "user_id": "user_001",
        "project_id": 42
    },
    "fields": {
        "task_id": "550e8400-e29b-41d4-a716-446655440000",
        "scene_type": "short_drama",
        "message": "生成任务已创建"
    },
    "trace_id": "abc123def456",
    "span_id": "span789",
    "thread.id": 7,
    "file": "src/handlers.rs",
    "line": 85
}
```

### 15.5 Prometheus 指标体系

```rust
// crates/tsangwu-common/src/metrics.rs
use axum::{Router, routing::get};
use metrics_exporter_prometheus::PrometheusBuilder;

pub fn init_metrics() -> anyhow::Result<PrometheusHandle> {
    let builder = PrometheusBuilder::new();
    let handle = builder.install_recorder()?;
    Ok(handle)
}

/// 指标暴露端点
pub fn metrics_routes(handle: PrometheusHandle) -> Router {
    Router::new().route("/metrics", get(move || {
        let handle = handle.clone();
        async move { handle.render() }
    }))
}
```

#### 基础设施指标

| 指标 | 采集方式 | 告警阈值 |
|------|----------|----------|
| CPU 利用率 | node_exporter | > 85% 持续 5min |
| 内存利用率 | node_exporter | > 90% |
| 磁盘使用率 | node_exporter | > 85% |
| GPU 利用率 | dcgm_exporter | > 95% 持续 10min |
| GPU 显存使用 | dcgm_exporter | > 90% |
| Pod 重启次数 | kube-state-metrics | > 3 次/小时 |

#### 业务指标

| 指标 | 说明 | 告警阈值 |
|------|------|----------|
| `gen_task_total` | 生成任务总数（按类型/状态） | — |
| `gen_task_duration_seconds` | 生成任务耗时分布 | P99 > SLA 2 倍 |
| `gen_task_failure_rate` | 生成任务失败率 | > 5% |
| `gen_queue_depth` | 生成队列深度 | > 200 |
| `api_request_total` | API 请求总数（按端点/状态码） | — |
| `api_latency_seconds` | API 响应延迟 | P99 > 3s |
| `api_error_rate` | API 错误率（5xx） | > 1% |
| `active_users` | 在线活跃用户数 | — |
| `asset_search_latency` | 素材检索延迟 | P99 > 500ms |
| `culture_check_accuracy` | 文化校验准确率 | < 85% |
| `copyright_risk_count` | 版权风险拦截数 | 日环比 > 200% |
| `agent_executions_total` | Agent 执行次数（按 Agent/状态） | — |
| `agent_execution_duration_seconds` | Agent 执行耗时 | — |
| `workflow_instances_active` | 活跃工作流实例数 | > 500 |
| `pybridge_gpu_utilization` | PyBridge GPU 占用率 | > 90% |

#### Grafana 核心面板

| 面板 | 包含内容 |
|------|----------|
| 系统总览 | QPS、错误率、延迟 P50/P99、活跃用户、任务队列 |
| 生成管线 | 各管线任务数/耗时/成功率、GPU 利用率、队列深度 |
| Agent 监控 | 各 Agent 执行耗时/成功率、工作流状态分布、重试次数 |
| 业务指标 | 注册量、生成量、付费转化、素材使用 TOP |
| 基础设施 | 节点资源、Pod 状态、中间件健康度 |
| PyBridge | Python 调用延迟、GPU 信号量等待时间、模型加载状态 |

### 15.6 链路追踪

基于 `tracing` + `tracing-opentelemetry` 自动埋点，覆盖全链路：

```
用户请求 → API网关 → 业务服务 → 工作流引擎 → Agent → PyBridge → 模型推理 → 响应
   │          │          │          │          │        │          │
   span1     span2      span3     span4      span5   span6      span7
   └──────────────────────────── trace_id ──────────────────────────┘
```

关键追踪点：
- Axum HTTP 入口出口（tower-http TraceLayer 自动埋点）
- Tonic gRPC 入口出口（tonic 内置 tracing 支持）
- Kafka 生产消费 trace 传播（通过 message header 携带 trace_id）
- 工作流引擎状态转换（每个步骤一个 span）
- PyO3 Python 调用耗时（spawn_blocking span）
- 外部模型 API 调用耗时与状态（reqwest tracing 中间件）

### 15.7 告警策略

| 告警级别 | 响应时间 | 通知方式 | 示例场景 |
|----------|----------|----------|----------|
| P0 - 致命 | 5 分钟内 | 电话 + 飞书 + 短信 | 服务不可用、数据库宕机、全量生成失败 |
| P1 - 严重 | 15 分钟内 | 飞书 + 短信 | 错误率飙升、GPU 节点故障、队列严重积压 |
| P2 - 警告 | 1 小时内 | 飞书群通知 | 延迟升高、磁盘空间不足、证书即将过期 |
| P3 - 提示 | 下个工作日 | 飞书群通知 | 非核心服务异常、日志量异常增长 |

告警收敛规则：
- 相同告警 5 分钟内去重
- 同一服务多个实例告警合并
- 告警恢复自动发送恢复通知
- 夜间（23:00-8:00）P2/P3 告警静默，P0/P1 正常通知

### 15.8 本地模式可观测性

本地模式下简化可观测性栈：

| 组件 | 分布式模式 | 本地模式 |
|------|-----------|---------|
| 日志 | JSON → Filebeat → ES | JSON → stdout / 本地文件 |
| 指标 | Prometheus scrape | `/metrics` 端点（可选 Prometheus） |
| 追踪 | OTLP → Jaeger | 禁用或输出到 stdout |
| 告警 | Alertmanager | 控制台日志告警 |

---

> 本文档为苍梧系统 Rust 技术方案 v1.0，覆盖原 Go 方案全部业务功能，新增多部署模式与自研工作流引擎设计。将随技术验证与实施进展持续迭代更新。
