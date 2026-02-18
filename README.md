# 苍梧 (TsangWu)

<p align="center">
  <strong>萃取华夏灵韵，生成东方影像</strong>
</p>

<p align="center">
  以中国传统文化为内核的多模态全链路影像生成平台
</p>

<p align="center">
  <a href="#核心功能">核心功能</a> •
  <a href="#技术架构">技术架构</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#api-文档">API 文档</a> •
  <a href="#部署指南">部署指南</a>
</p>

---

## 项目概述

苍梧（TsangWu）是一个基于 Rust 构建的高性能 AI 驱动影像生成平台，专注于中国传统文化内容创作。平台支持图文、短剧、电影、电视剧四大场景的全链路生成，为零基础用户到专业影视团队提供分层适配的创作工具。

### 核心特性

- **文化适配引擎**：深度优化国风元素生成，确保传统文化准确性
- **全链路覆盖**：图文 → 短剧 → 电影 → 电视剧全流程打通
- **分层适配**：零基础模式、进阶模式、专业模式三级用户适配
- **Agent 协作系统**：五大智能体协作完成内容生成全流程
- **高性能架构**：基于 Rust + Tokio 的异步高并发架构

---

## 核心功能

### 四大生成场景

| 场景 | 功能描述 | 适用用户 |
|------|----------|----------|
| **图文生成** | 国风插画、海报、分镜图文、文化科普图文 | 全用户 |
| **短剧生成** | 1-30 分钟国风短剧，全流程自动化 | 自媒体、创作者 |
| **电影生成** | 电影级剧本、分镜、影视预演，支持 4K 输出 | 影视工作室 |
| **电视剧生成** | 多集剧本、分镜批量生成，团队协同 | 专业团队 |

### 中国特色功能

- **中式叙事逻辑**：内置家国情怀、江湖侠义、古典爱情等叙事模板
- **国风元素精准生成**：古风服饰形制、中式建筑结构、传统道具细节
- **传统文化联动**：古典诗词、传统典故、非遗元素可视化生成
- **历史题材适配**：唐/宋/明/清等不同朝代场景、服饰、礼仪精准区分

---

## 技术架构

### 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      接入层 (Gateway)                        │
│           Web / Mobile / API / 第三方集成                    │
├─────────────────────────────────────────────────────────────┤
│                      应用服务层                              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │ User    │ │ Project │ │ Agent   │ │ Culture │           │
│  │ Service │ │ Service │ │ Service │ │ Service │           │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │ Gen     │ │ Asset   │ │ Auth    │ │ Pay     │           │
│  │ Service │ │ Service │ │ Service │ │ Service │           │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │
├─────────────────────────────────────────────────────────────┤
│                      核心能力层                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ PyBridge    │ │ Workflow    │ │ Cache       │           │
│  │ (AI 能力)   │ │ Engine      │ │ (Redis)     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│                      基础设施层                              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │
│  │PostgreSQL│ │ Redis   │ │ Kafka   │ │ MinIO   │           │
│  │ SQLite  │ │         │ │         │ │  S3     │           │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### 技术栈

| 类别 | 技术选型 |
|------|----------|
| **编程语言** | Rust (Edition 2021) |
| **异步运行时** | Tokio |
| **Web 框架** | Axum 0.8 |
| **gRPC** | Tonic + Prost |
| **数据库** | PostgreSQL / SQLite (SQLx) |
| **ORM** | SeaORM |
| **缓存** | Redis (fred) / 内存缓存 (moka) |
| **消息队列** | Kafka (rdkafka) / 内存队列 |
| **对象存储** | MinIO / S3 (opendal) |
| **文档存储** | MongoDB |
| **认证** | JWT (jsonwebtoken) + Argon2 |
| **可观测性** | Tracing + OpenTelemetry + Prometheus |

### 模块结构

```
tswu/
├── bins/                          # 可执行程序
│   ├── tsangwu-local/             # 本地模式入口
│   ├── tsangwu-server/            # 服务器模式入口
│   └── tsangwu-migrate/           # 数据库迁移工具
├── crates/                        # 核心库
│   ├── tsangwu-common/            # 公共工具库
│   ├── tsangwu-config/            # 配置管理
│   ├── tsangwu-db/                # 数据库层
│   ├── tsangwu-auth/              # 认证授权
│   ├── tsangwu-cache/             # 缓存层
│   ├── tsangwu-storage/           # 对象存储
│   ├── tsangwu-mq/                # 消息队列
│   ├── tsangwu-workflow/          # 工作流引擎
│   ├── tsangwu-gateway/           # API 网关
│   ├── tsangwu-pybridge/          # AI 能力桥接
│   ├── tsangwu-agent-svc/         # Agent 服务
│   ├── tsangwu-culture-svc/       # 文化引擎服务
│   ├── tsangwu-user-svc/          # 用户服务
│   ├── tsangwu-project-svc/       # 项目管理服务
│   ├── tsangwu-gen-svc/           # 内容生成服务
│   ├── tsangwu-asset-svc/         # 资产管理服务
│   └── ...                        # 其他服务模块
├── config/                        # 配置文件
│   ├── local.yaml                 # 本地模式配置
│   ├── standalone.yaml            # 单机模式配置
│   └── distributed.yaml           # 分布式模式配置
├── deploy/                        # 部署配置
│   ├── docker/                    # Docker 部署
│   └── k8s/                       # Kubernetes 部署
├── proto/                         # Protobuf 定义
└── docs/                          # 文档
```

---

## 快速开始

### 环境要求

- Rust 1.70+ (推荐 1.80+)
- SQLite 3.x (本地模式)
- 或 PostgreSQL 16+ (生产模式)

### 本地模式（快速体验）

```bash
# 克隆项目
git clone https://git.tsangwu.com/tsangwu/tsangwu.git
cd tsangwu

# 编译项目
cargo build --release

# 运行本地模式
./target/release/tsangwu-local.exe
```

服务将在 `http://127.0.0.1:8080` 启动。

### 验证安装

```bash
# 健康检查
curl http://127.0.0.1:8080/healthz

# 就绪检查
curl http://127.0.0.1:8080/readyz
```

### Docker 部署

```bash
# 启动完整服务栈
cd deploy/docker
docker-compose up -d

# 查看服务状态
docker-compose ps
```

Docker Compose 包含以下服务：
- PostgreSQL (端口 5432)
- Redis (端口 6379)
- MinIO (端口 9000, 控制台 9001)
- Kafka (端口 9092)
- TsangWu Server (端口 8080, gRPC 9090)

---

## 配置说明

### 配置文件

项目支持三种部署模式：

| 模式 | 配置文件 | 数据库 | 缓存 | 存储 |
|------|----------|--------|------|------|
| 本地模式 | `local.yaml` | SQLite | 内存 | 本地文件系统 |
| 单机模式 | `standalone.yaml` | PostgreSQL | Redis | MinIO |
| 分布式模式 | `distributed.yaml` | PostgreSQL 集群 | Redis 集群 | S3 |

### 本地模式配置示例

```yaml
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
telemetry:
  log_level: debug
  service_name: tsangwu-local
```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `TSANGWU_PROFILE` | 配置文件名称 | `local` |
| `TSANGWU_DATA_DIR` | 数据目录 | `./data` |
| `TSANGWU_JWT_SECRET` | JWT 密钥 | `tsangwu-local-dev-key` |

---

## API 文档

### Agent 服务 API

#### 创建 Agent

```http
POST /api/agents
Content-Type: application/json

{
  "project_id": "550e8400-e29b-41d4-a716-446655440000",
  "agent_type": "script",
  "config": {
    "model": "qwen-7b",
    "temperature": 0.7
  }
}
```

**Agent 类型**：
- `script` - 剧本生成
- `storyboard` - 分镜规划
- `visual` - 视觉生成
- `audio` - 音频合成
- `quality` - 质量审核

#### 执行任务

```http
POST /api/agents/{agent_id}/execute
Content-Type: application/json

{
  "task_type": "generate_script",
  "input": {
    "theme": "唐朝宫廷",
    "style": "历史剧",
    "duration": 120
  }
}
```

### 文化引擎 API

#### 创建朝代

```http
POST /api/culture/dynasties
Content-Type: application/json

{
  "name": "唐朝",
  "period": "盛唐",
  "start_year": 618,
  "end_year": 907,
  "description": "中国历史上的黄金时代",
  "cultural_features": ["开放包容", "诗歌繁荣"]
}
```

#### 验证内容文化准确性

```http
POST /api/culture/validate
Content-Type: application/json

{
  "dynasty_id": "dynasty-uuid",
  "content_type": "script",
  "content": {
    "text": "剧本内容..."
  }
}
```

#### 推荐文化元素

```http
POST /api/culture/recommend
Content-Type: application/json

{
  "dynasty_id": "dynasty-uuid",
  "element_type": "costume",
  "context": "宫廷宴会场景",
  "limit": 10
}
```

**文化元素类型**：
- `architecture` - 建筑
- `costume` - 服饰
- `music` - 音乐
- `art` - 艺术
- `literature` - 文学
- `custom` - 习俗

### 完整 API 端点列表

| 服务 | 端点 | 方法 | 说明 |
|------|------|------|------|
| Agent | `/api/agents` | POST | 创建 Agent |
| Agent | `/api/agents/:id` | GET | 获取 Agent 详情 |
| Agent | `/api/agents/project/:project_id` | GET | 列出项目的 Agent |
| Agent | `/api/agents/:id/execute` | POST | 执行任务 |
| Agent | `/api/agents/:id/tasks/:task_id` | GET | 获取任务详情 |
| Culture | `/api/culture/dynasties` | POST | 创建朝代 |
| Culture | `/api/culture/dynasties` | GET | 列出所有朝代 |
| Culture | `/api/culture/dynasties/:id` | GET | 获取朝代详情 |
| Culture | `/api/culture/elements` | POST | 创建文化元素 |
| Culture | `/api/culture/validate` | POST | 验证内容 |
| Culture | `/api/culture/recommend` | POST | 推荐元素 |

---

## 部署指南

### Kubernetes 部署

```bash
# 开发环境
kubectl apply -k deploy/k8s/overlays/dev

# 预发布环境
kubectl apply -k deploy/k8s/overlays/staging

# 生产环境
kubectl apply -k deploy/k8s/overlays/prod
```

### 生产环境检查清单

- [ ] 配置 PostgreSQL 高可用集群
- [ ] 配置 Redis Sentinel 或 Cluster
- [ ] 配置 Kafka 集群
- [ ] 配置对象存储（MinIO 或 S3）
- [ ] 配置 TLS 证书
- [ ] 配置 JWT 密钥（使用强密钥）
- [ ] 配置日志收集（ELK/Loki）
- [ ] 配置监控告警（Prometheus + Grafana）
- [ ] 配置分布式追踪（Jaeger/Zipkin）

---

## 开发指南

### 编译

```bash
# 开发模式
cargo build

# 发布模式（优化编译）
cargo build --release
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test -p tsangwu-db
```

### 代码规范

```bash
# 格式化代码
cargo fmt

# 静态检查
cargo clippy -- -D warnings
```

### 数据库迁移

```bash
# 运行迁移
./target/release/tsangwu-migrate
```

---

## 性能指标

| 指标 | 零基础模式 | 专业模式 |
|------|-----------|----------|
| 图文生成时间 | ≤10秒/张 | ≤30秒/张 |
| 短剧生成时间（1分钟） | ≤5分钟 | ≤15分钟 |
| 影视预演生成时间（1分钟） | - | ≤30分钟 |
| 角色一致性评分 | ≥85% | ≥95% |
| 音画同步误差 | ≤0.2秒 | ≤0.1秒 |
| 系统可用性 | ≥99.9% | ≥99.9% |

---

## 贡献指南

我们欢迎所有形式的贡献！

### 贡献流程

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 Bug
- `docs:` 文档更新
- `style:` 代码格式调整
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建/工具链相关

---

## 许可证

本项目采用专有许可证，未经授权不得用于商业用途。详见 [LICENSE](LICENSE) 文件。

---

## 联系方式

- **项目地址**：https://git.tsangwu.com/tsangwu/tsangwu
- **问题反馈**：https://git.tsangwu.com/tsangwu/tsangwu/issues

---

<p align="center">
  <strong>苍梧 TsangWu</strong> — 萃取华夏灵韵，生成东方影像
</p>
