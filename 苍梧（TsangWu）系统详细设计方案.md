# 苍梧（TsangWu）系统详细设计方案

> 版本：v1.0 | 更新日期：2026-02-17 | 状态：详细设计阶段
> 基于：苍梧（TsangWu）产品需求文档 v2.0

---

## 目录

1. [系统概述](#1-系统概述)
2. [总体架构设计](#2-总体架构设计)
3. [微服务拆分](#3-微服务拆分)
4. [数据库设计](#4-数据库设计)
5. [API设计](#5-api设计)
6. [AI Agent系统设计](#6-ai-agent系统设计)
7. [生成管线设计](#7-生成管线设计)
8. [文化适配引擎设计](#8-文化适配引擎设计)
9. [素材库系统设计](#9-素材库系统设计)
10. [用户与权限系统](#10-用户与权限系统)
11. [基础设施与部署](#11-基础设施与部署)
12. [安全与合规](#12-安全与合规)
13. [监控与可观测性](#13-监控与可观测性)

---

## 1. 系统概述

### 1.1 设计目标

苍梧系统旨在构建一个以中国传统文化为内核的多模态全链路影像生成平台，核心设计目标：

| 目标 | 描述 |
|------|------|
| 文化精准 | 国风元素生成准确率 ≥ 90%，覆盖服饰形制、建筑结构、传统色彩等维度 |
| 全链路贯通 | 打通图文→短剧→电影→电视剧四大场景，素材跨场景复用 |
| 分层适配 | 零基础用户一键生成、专业用户全参数可控，渐进式功能暴露 |
| 高可用 | 系统可用性 ≥ 99.9%，支持 1000+ 并发用户 |
| 弹性伸缩 | 算力按需分配，支持突发流量弹性扩容 |
| 安全合规 | 符合国家数据安全法规，内置版权校验与内容审核 |

### 1.2 设计范围

本方案覆盖苍梧系统全部技术层面，包括：
- 前后端应用架构与微服务拆分
- 数据库模型与存储方案
- AI Agent 编排与生成管线
- 文化适配引擎与素材库系统
- 基础设施、安全合规、监控体系

### 1.3 设计约束

| 约束类型 | 具体约束 |
|----------|----------|
| 技术约束 | 依赖第三方大模型（豆包/DeepSeek/SD/Gemini），需适配多模型接口差异 |
| 性能约束 | 图文生成 ≤ 10s（零基础）/ ≤ 30s（专业）；短剧 1min 内容生成 ≤ 5min |
| 合规约束 | 符合《数据安全法》《个人信息保护法》《生成式AI管理办法》 |
| 成本约束 | GPU 算力成本需通过差异化调度与缓存机制控制在可盈利范围 |
| 分期约束 | Phase 1-2 优先图文+短剧，电影/电视剧模块延后 |

### 1.4 技术选型总览

| 领域 | 技术选型 | 选型理由 |
|------|----------|----------|
| 后端框架 | Go (Gin/gRPC) | 高并发、低延迟，适合微服务架构 |
| 前端框架 | React + Next.js | SSR 支持、生态成熟、国际化友好 |
| 移动端 | React Native | 跨平台复用，与 Web 端技术栈统一 |
| API 网关 | Kong / APISIX | 国产开源，插件丰富，支持限流/鉴权/灰度 |
| 消息队列 | Apache Kafka | 高吞吐，适合异步生成任务调度 |
| 任务调度 | Temporal | 工作流编排，支持长时间运行的生成任务，自带重试与补偿 |
| 关系数据库 | PostgreSQL | 成熟稳定，JSON 支持好，适合复杂业务模型 |
| 文档数据库 | MongoDB | 灵活 Schema，适合素材元数据与生成配置存储 |
| 缓存 | Redis Cluster | 高性能缓存，支持分布式锁与会话管理 |
| 对象存储 | 阿里云 OSS / MinIO | 海量素材与生成产物存储 |
| 搜索引擎 | Elasticsearch | 素材全文检索、标签搜索 |
| 向量数据库 | Milvus | 素材语义检索、风格相似度匹配 |
| 容器编排 | Kubernetes (ACK) | 弹性伸缩、服务治理 |
| CI/CD | GitLab CI + ArgoCD | GitOps 流程，自动化部署 |
| 监控 | Prometheus + Grafana | 指标采集与可视化 |
| 日志 | ELK (Elasticsearch + Logstash + Kibana) | 集中式日志管理 |
| 链路追踪 | Jaeger / OpenTelemetry | 分布式链路追踪 |
| GPU 调度 | NVIDIA Triton + KubeFlow | 模型推理服务化、GPU 资源调度 |

---

## 2. 总体架构设计

### 2.1 五层架构详细展开

```
┌──────────────────────────────────────────────────────────────────────┐
│                        接入层 (Access Layer)                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐    │
│  │  Web SPA │  │ 移动端App │  │ API 网关  │  │ 第三方插件/SDK   │    │
│  │ (Next.js)│  │ (RN)     │  │(APISIX)  │  │ (PR/AE/平台)     │    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬──────────┘    │
│       └──────────────┴─────────────┴────────────────┘               │
│                          │  CDN (阿里云CDN)                          │
├──────────────────────────┼───────────────────────────────────────────┤
│                   应用层 (Application Layer)                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐       │
│  │ 需求解析   │ │ 创意规划   │ │ 内容生成   │ │ 编辑优化   │       │
│  │ Service    │ │ Service    │ │ Service    │ │ Service    │       │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘       │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐       │
│  │ 审片校验   │ │ 输出分发   │ │ 版权保护   │ │ 模板引擎   │       │
│  │ Service    │ │ Service    │ │ Service    │ │ Service    │       │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘       │
├─────────────────────────────────────────────────────────────────────┤
│                核心技术层 (Core AI Layer)                             │
│  ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐    │
│  │ Agent 协作引擎    │ │ 生成模型集群      │ │ 文化适配引擎      │    │
│  │ (Temporal编排)    │ │ (Triton推理)     │ │ (规则+模型)      │    │
│  └──────────────────┘ └──────────────────┘ └──────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                  基础层 (Infrastructure Layer)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ 大模型集群    │  │ 算力调度      │  │ 数据集管理    │              │
│  │ (多模型适配)  │  │ (GPU Pool)   │  │ (版本化管理)  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
├─────────────────────────────────────────────────────────────────────┤
│                安全与合规层 (Security & Compliance)                   │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐       │
│  │ 版权校验   │ │ 内容审核   │ │ 数据安全   │ │ 隐私保护   │       │
│  │ Engine     │ │ Engine     │ │ (加密/脱敏)│ │ (隔离存储) │       │
│  └────────────┘ └────────────┘ └────────────┘ └────────────┘       │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 系统拓扑与服务交互

```
用户请求 → CDN → API网关(APISIX) → 服务路由
                                        │
                    ┌───────────────────┼───────────────────┐
                    ▼                   ▼                   ▼
              用户服务            项目服务             素材服务
              (认证/权限)        (创作管理)           (检索/管理)
                    │                   │                   │
                    └───────────┬───────┘                   │
                                ▼                           │
                          任务调度中心 (Temporal)            │
                                │                           │
                    ┌───────────┼───────────┐               │
                    ▼           ▼           ▼               │
              需求解析      创意规划     内容生成 ◄──────────┘
              Agent        Agent        Agent
                                          │
                              ┌───────────┼───────────┐
                              ▼           ▼           ▼
                        文化适配引擎  生成模型集群   编辑优化Agent
                                          │
                                          ▼
                                    审片校验Agent
                                          │
                                          ▼
                              ┌───────────┼───────────┐
                              ▼           ▼           ▼
                        版权校验      内容审核     输出分发
                                                      │
                                          ┌───────────┼───────────┐
                                          ▼           ▼           ▼
                                        OSS        CDN        平台推送
```

### 2.3 通信机制

| 通信方式 | 适用场景 | 协议 |
|----------|----------|------|
| 同步 RPC | 服务间实时调用（用户鉴权、素材查询） | gRPC + Protobuf |
| 异步消息 | 生成任务下发、状态通知、事件驱动 | Kafka |
| 工作流编排 | Agent 多步骤协作、长时间生成任务 | Temporal Workflow |
| WebSocket | 前端实时进度推送、协同编辑 | WS over TLS |
| HTTP REST | 外部 API 开放接口、第三方集成 | HTTPS + JSON |

---

## 3. 微服务拆分

### 3.1 服务全景

```
┌─────────────────────────────────────────────────────────┐
│                    BFF / API Gateway                     │
├──────────┬──────────┬──────────┬──────────┬─────────────┤
│ 用户服务  │ 项目服务  │ 素材服务  │ 生成服务  │ 分发服务    │
│ user-svc │ proj-svc │ asset-svc│ gen-svc  │ dist-svc   │
├──────────┼──────────┼──────────┼──────────┼─────────────┤
│ 权限服务  │ 模板服务  │ 版权服务  │ Agent服务 │ 审核服务    │
│ auth-svc │ tmpl-svc │ copy-svc │ agent-svc│ audit-svc  │
├──────────┼──────────┼──────────┴──────────┴─────────────┤
│ 支付服务  │ 通知服务  │         文化适配服务               │
│ pay-svc  │ notif-svc│         culture-svc               │
└──────────┴──────────┴───────────────────────────────────┘
```

### 3.2 各服务职责与边界

| 服务名 | 职责 | 核心能力 | 依赖 |
|--------|------|----------|------|
| **user-svc** | 用户账号管理 | 注册/登录/个人信息/企业信息/OAuth | PostgreSQL, Redis |
| **auth-svc** | 认证与权限 | JWT签发/RBAC/资源鉴权/API Key管理 | PostgreSQL, Redis |
| **proj-svc** | 项目与创作管理 | 项目CRUD/版本管理/协同编辑/历史记录 | PostgreSQL, Redis, Kafka |
| **asset-svc** | 素材库管理 | 素材上传/检索/分类/标签/收藏 | MongoDB, OSS, ES, Milvus |
| **tmpl-svc** | 模板引擎 | 模板CRUD/推荐/组合/版本控制 | MongoDB, Redis |
| **gen-svc** | 生成任务管理 | 任务创建/调度/状态追踪/结果管理 | PostgreSQL, Kafka, Temporal |
| **agent-svc** | Agent编排 | 五大Agent调度/工作流执行/上下文管理 | Temporal, Kafka, Redis |
| **culture-svc** | 文化适配 | 国风识别/元素匹配/朝代校验/审美优化 | MongoDB, Milvus, 模型推理 |
| **copy-svc** | 版权保护 | 版权校验/侵权检测/版权证明生成 | PostgreSQL, ES |
| **audit-svc** | 内容审核 | 敏感内容检测/违规拦截/人工复审 | Kafka, 模型推理 |
| **dist-svc** | 输出分发 | 格式转换/平台适配/CDN推送/导出 | OSS, CDN, Kafka |
| **pay-svc** | 支付与订阅 | 订阅管理/用量计费/支付对接/发票 | PostgreSQL, 支付网关 |
| **notif-svc** | 通知服务 | 站内信/邮件/短信/WebSocket推送 | Kafka, Redis |

### 3.3 服务间通信方式

```
user-svc ──gRPC──► auth-svc          (鉴权校验)
proj-svc ──gRPC──► asset-svc         (素材引用)
proj-svc ──Kafka─► gen-svc           (触发生成)
gen-svc  ──Temporal► agent-svc       (Agent编排)
agent-svc──gRPC──► culture-svc       (文化校验)
agent-svc──gRPC──► asset-svc         (素材检索)
gen-svc  ──Kafka─► audit-svc         (内容审核)
gen-svc  ──Kafka─► copy-svc          (版权校验)
gen-svc  ──Kafka─► dist-svc          (输出分发)
pay-svc  ──gRPC──► user-svc          (权益查询)
notif-svc◄─Kafka── *                 (事件通知)
```

### 3.4 服务分期上线策略

| 阶段 | 上线服务 | 说明 |
|------|----------|------|
| Phase 1 | user-svc, auth-svc, asset-svc, tmpl-svc, notif-svc | 基础设施与账号体系 |
| Phase 2 | proj-svc, gen-svc, agent-svc, culture-svc, audit-svc, copy-svc, dist-svc | 图文+短剧生成全链路 |
| Phase 3 | pay-svc；gen-svc 扩展电影管线 | 商业化+电影模块 |
| Phase 4 | gen-svc 扩展电视剧管线；proj-svc 扩展协同编辑 | 电视剧+团队协同 |

---

## 4. 数据库设计

### 4.1 存储策略总览

| 存储类型 | 技术 | 存储内容 |
|----------|------|----------|
| 关系型 | PostgreSQL | 用户、权限、项目、订单、版权等结构化业务数据 |
| 文档型 | MongoDB | 素材元数据、模板配置、Agent上下文、生成参数 |
| 缓存 | Redis Cluster | 会话、热点数据、分布式锁、限流计数器 |
| 对象存储 | 阿里云 OSS | 图片、视频、音频等二进制素材与生成产物 |
| 全文检索 | Elasticsearch | 素材标签检索、内容搜索、版权库比对 |
| 向量存储 | Milvus | 素材语义向量、风格特征向量、相似度检索 |

### 4.2 核心数据模型（PostgreSQL）

#### 用户与权限

```sql
-- 用户表
CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    phone           VARCHAR(20) UNIQUE,
    email           VARCHAR(128) UNIQUE,
    nickname        VARCHAR(64) NOT NULL,
    avatar_url      VARCHAR(512),
    account_type    SMALLINT NOT NULL DEFAULT 0,  -- 0:个人 1:企业 2:专业团队
    status          SMALLINT NOT NULL DEFAULT 1,  -- 0:禁用 1:正常
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 企业信息表
CREATE TABLE organizations (
    id              BIGSERIAL PRIMARY KEY,
    name            VARCHAR(128) NOT NULL,
    license_no      VARCHAR(64),
    owner_id        BIGINT NOT NULL REFERENCES users(id),
    max_members     INT NOT NULL DEFAULT 5,
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:待审核 1:正常
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 角色表
CREATE TABLE roles (
    id              SERIAL PRIMARY KEY,
    code            VARCHAR(32) NOT NULL UNIQUE,   -- admin/creator/reviewer
    name            VARCHAR(64) NOT NULL,
    description     TEXT
);

-- 用户角色关联
CREATE TABLE user_roles (
    user_id         BIGINT NOT NULL REFERENCES users(id),
    role_id         INT NOT NULL REFERENCES roles(id),
    org_id          BIGINT REFERENCES organizations(id),  -- NULL表示全局角色
    PRIMARY KEY (user_id, role_id, org_id)
);

-- 权限表
CREATE TABLE permissions (
    id              SERIAL PRIMARY KEY,
    code            VARCHAR(64) NOT NULL UNIQUE,
    resource        VARCHAR(64) NOT NULL,
    action          VARCHAR(32) NOT NULL
);

-- 角色权限关联
CREATE TABLE role_permissions (
    role_id         INT NOT NULL REFERENCES roles(id),
    permission_id   INT NOT NULL REFERENCES permissions(id),
    PRIMARY KEY (role_id, permission_id)
);
```

#### 项目与创作

```sql
-- 项目表
CREATE TABLE projects (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    owner_id        BIGINT NOT NULL REFERENCES users(id),
    org_id          BIGINT REFERENCES organizations(id),
    title           VARCHAR(256) NOT NULL,
    scene_type      SMALLINT NOT NULL,  -- 0:图文 1:短剧 2:电影 3:电视剧
    user_mode       SMALLINT NOT NULL DEFAULT 0,  -- 0:零基础 1:进阶 2:专业
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:草稿 1:生成中 2:已完成 3:已发布
    config          JSONB,              -- 项目配置（风格/参数等）
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 项目版本表
CREATE TABLE project_versions (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT NOT NULL REFERENCES projects(id),
    version         INT NOT NULL,
    snapshot        JSONB NOT NULL,     -- 项目快照
    created_by      BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, version)
);
```

#### 生成任务与订单

```sql
-- 生成任务表
CREATE TABLE generation_tasks (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    project_id      BIGINT NOT NULL REFERENCES projects(id),
    user_id         BIGINT NOT NULL REFERENCES users(id),
    task_type       SMALLINT NOT NULL,  -- 0:图文 1:短剧 2:电影 3:电视剧
    priority        SMALLINT NOT NULL DEFAULT 1,  -- 0:低 1:中 2:高
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:待处理 1:排队中 2:生成中 3:已完成 4:失败
    input_params    JSONB NOT NULL,     -- 输入参数
    output_urls     JSONB,              -- 输出文件URL列表
    progress        SMALLINT DEFAULT 0, -- 进度百分比 0-100
    error_msg       TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_gen_tasks_user ON generation_tasks(user_id, status);
CREATE INDEX idx_gen_tasks_project ON generation_tasks(project_id);

-- 订阅与套餐表
CREATE TABLE subscriptions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    plan_code       VARCHAR(32) NOT NULL,  -- free/personal/advanced/pro/enterprise
    status          SMALLINT NOT NULL DEFAULT 1,  -- 0:过期 1:生效 2:取消
    started_at      TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    auto_renew      BOOLEAN DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 用量记录表
CREATE TABLE usage_records (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    task_id         BIGINT REFERENCES generation_tasks(id),
    usage_type      VARCHAR(32) NOT NULL,  -- image_gen/video_gen/4k_output/batch_gen
    quantity        INT NOT NULL DEFAULT 1,
    credits_cost    INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_usage_user_date ON usage_records(user_id, created_at);
```

#### 版权与审核

```sql
-- 版权记录表
CREATE TABLE copyright_records (
    id              BIGSERIAL PRIMARY KEY,
    task_id         BIGINT NOT NULL REFERENCES generation_tasks(id),
    check_status    SMALLINT NOT NULL DEFAULT 0,  -- 0:待检 1:通过 2:风险 3:侵权
    risk_score      DECIMAL(5,2),
    risk_details    JSONB,
    certificate_url VARCHAR(512),       -- 版权证明文件
    checked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 内容审核记录表
CREATE TABLE content_reviews (
    id              BIGSERIAL PRIMARY KEY,
    task_id         BIGINT NOT NULL REFERENCES generation_tasks(id),
    review_type     SMALLINT NOT NULL,  -- 0:自动 1:人工
    status          SMALLINT NOT NULL DEFAULT 0,  -- 0:待审 1:通过 2:拒绝 3:需人工复审
    risk_labels     JSONB,              -- 风险标签列表
    reviewer_id     BIGINT REFERENCES users(id),
    reviewed_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### 4.3 文档数据模型（MongoDB）

#### 素材元数据

```json
// collection: assets
{
    "_id": ObjectId,
    "asset_uid": UUID,
    "name": "汉服·唐制齐胸襦裙",
    "category": "costume",           // costume/scene/prop/color/pattern/narrative
    "sub_category": "hanfu_tang",
    "tags": ["汉服", "唐制", "齐胸襦裙", "女装", "正式"],
    "dynasty": "tang",               // 朝代标签
    "cultural_labels": ["传统服饰", "形制考据"],
    "file_urls": {
        "thumbnail": "oss://assets/thumb/xxx.jpg",
        "preview": "oss://assets/preview/xxx.jpg",
        "original": "oss://assets/original/xxx.png"
    },
    "metadata": {
        "format": "png",
        "width": 2048,
        "height": 2048,
        "file_size": 4521984
    },
    "embedding_id": "milvus_vector_id",  // Milvus向量ID
    "copyright": {
        "license": "commercial",
        "source": "internal",
        "expires_at": null
    },
    "usage_count": 12580,
    "status": 1,
    "created_at": ISODate,
    "updated_at": ISODate
}
```

#### Agent上下文

```json
// collection: agent_contexts
{
    "_id": ObjectId,
    "task_id": "gen_task_uuid",
    "workflow_run_id": "temporal_run_id",
    "agents": {
        "parser": {
            "input": { "raw_text": "...", "images": [] },
            "output": { "intent": "short_drama", "style": "wuxia", "elements": [] },
            "status": "completed"
        },
        "planner": {
            "input": {},
            "output": { "script": {}, "storyboard": [] },
            "status": "completed"
        },
        "generator": {
            "input": {},
            "output": { "segments": [] },
            "status": "running",
            "progress": 65
        }
    },
    "shared_context": {
        "style_profile": {},
        "character_refs": [],
        "scene_refs": []
    },
    "created_at": ISODate,
    "updated_at": ISODate
}
```

### 4.4 数据关系总览

```
users ──1:N──► projects ──1:N──► generation_tasks
  │                                    │
  ├──1:N──► subscriptions              ├──1:1──► copyright_records
  ├──1:N──► usage_records              ├──1:N──► content_reviews
  └──N:M──► roles ──N:M──► permissions └──1:1──► agent_contexts (MongoDB)
                                                       │
organizations ──1:N──► users                    assets (MongoDB)
                                                  │
project_versions ◄──N:1── projects          Milvus (向量索引)
```

---

## 5. API设计

### 5.1 RESTful 接口规范

| 规范项 | 约定 |
|--------|------|
| 基础路径 | `https://api.tsangwu.com/v1` |
| 认证方式 | Bearer Token (JWT)；开放API使用 API Key + HMAC签名 |
| 请求格式 | `Content-Type: application/json`；文件上传使用 `multipart/form-data` |
| 响应格式 | 统一 JSON 信封：`{ "code": 0, "message": "ok", "data": {} }` |
| 分页 | `?page=1&page_size=20`，响应含 `total` / `has_more` |
| 错误码 | 业务错误码 5 位数字，HTTP 状态码遵循 RESTful 语义 |
| 版本控制 | URL 路径版本 `/v1`，大版本不兼容时升级 |
| 限流 | 按用户套餐分级限流，响应头返回 `X-RateLimit-*` |

### 5.2 核心端点设计

#### 用户与认证

```
POST   /v1/auth/register              # 注册（手机号/邮箱）
POST   /v1/auth/login                 # 登录
POST   /v1/auth/refresh               # 刷新Token
POST   /v1/auth/oauth/{provider}      # 第三方登录（微信/微博）
GET    /v1/users/me                   # 获取当前用户信息
PUT    /v1/users/me                   # 更新用户信息
GET    /v1/users/me/subscription      # 查询订阅状态
GET    /v1/users/me/usage             # 查询用量统计
```

#### 项目管理

```
POST   /v1/projects                   # 创建项目
GET    /v1/projects                   # 项目列表（支持筛选/分页）
GET    /v1/projects/{id}              # 项目详情
PUT    /v1/projects/{id}              # 更新项目
DELETE /v1/projects/{id}              # 删除项目
GET    /v1/projects/{id}/versions     # 版本历史
POST   /v1/projects/{id}/versions     # 创建版本快照
```

#### 生成任务

```
POST   /v1/projects/{id}/generate     # 提交生成任务
GET    /v1/tasks/{task_id}            # 查询任务状态与进度
GET    /v1/tasks/{task_id}/result     # 获取生成结果
POST   /v1/tasks/{task_id}/cancel     # 取消任务
POST   /v1/tasks/{task_id}/retry      # 重试失败任务
GET    /v1/tasks                      # 任务列表（当前用户）
```

#### 素材库

```
GET    /v1/assets                     # 素材列表（分类/标签/搜索）
GET    /v1/assets/{id}                # 素材详情
POST   /v1/assets/search              # 语义搜索（向量检索）
POST   /v1/assets/upload              # 上传自定义素材
DELETE /v1/assets/{id}                # 删除自定义素材
POST   /v1/assets/{id}/favorite       # 收藏素材
GET    /v1/assets/favorites           # 收藏列表
```

#### 模板

```
GET    /v1/templates                  # 模板列表（按场景/风格筛选）
GET    /v1/templates/{id}             # 模板详情
GET    /v1/templates/recommend        # 个性化推荐
POST   /v1/templates/{id}/apply       # 应用模板到项目
```

#### 输出与分发

```
POST   /v1/tasks/{task_id}/export     # 导出（指定格式/分辨率）
GET    /v1/tasks/{task_id}/exports    # 导出记录
POST   /v1/tasks/{task_id}/publish    # 发布到平台（抖音/视频号等）
GET    /v1/tasks/{task_id}/copyright  # 版权校验结果与证明
```

#### 开放API（第三方集成）

```
POST   /v1/open/generate/image        # 图文生成
POST   /v1/open/generate/video        # 视频生成
GET    /v1/open/tasks/{task_id}       # 查询任务
POST   /v1/open/assets/search         # 素材检索
GET    /v1/open/quota                 # 查询配额
```

### 5.3 关键接口示例

#### 提交生成任务

```http
POST /v1/projects/{id}/generate
Authorization: Bearer <token>

{
    "scene_type": "short_drama",
    "mode": "beginner",
    "input": {
        "prompt": "一段发生在唐朝长安城的江湖侠义故事，主角是一位女侠",
        "reference_images": ["oss://user-uploads/ref1.jpg"],
        "template_id": "tmpl_wuxia_001"
    },
    "config": {
        "duration_minutes": 3,
        "resolution": "1080p",
        "style": "wuxia",
        "music_style": "guzheng",
        "dynasty": "tang"
    }
}
```

响应：

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "task_id": "550e8400-e29b-41d4-a716-446655440000",
        "status": "queued",
        "estimated_queue_position": 3,
        "ws_channel": "wss://ws.tsangwu.com/tasks/550e8400/progress"
    }
}
```

---

## 6. AI Agent系统设计

### 6.1 五大Agent总览

```
用户输入
   │
   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  需求解析     │───►│  创意规划     │───►│  内容生成     │
│  Parser Agent│    │ Planner Agent│    │Generator Agent│
└──────────────┘    └──────────────┘    └──────┬───────┘
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

### 6.2 各Agent职责与能力

#### Agent 1：需求解析 Agent (Parser)

| 项目 | 说明 |
|------|------|
| 输入 | 用户原始输入（文本/图片/语音）、用户模式、历史偏好 |
| 输出 | 结构化需求描述（意图、场景类型、风格、文化元素、参数） |
| 核心能力 | 自然语言理解、多模态输入解析、文化语义识别、场景路由 |
| 依赖模型 | DeepSeek（文本理解）、多模态模型（图片理解） |

关键逻辑：
```
1. 输入预处理：语音→文本转写；图片→视觉特征提取
2. 意图识别：判断场景类型（图文/短剧/电影/电视剧）
3. 文化语义解析：识别国风关键词，匹配朝代/风格/元素
4. 模式适配：根据用户模式补全默认参数（零基础自动填充，专业保留空白）
5. 输出结构化 JSON 传递给 Planner Agent
```

#### Agent 2：创意规划 Agent (Planner)

| 项目 | 说明 |
|------|------|
| 输入 | Parser 输出的结构化需求 |
| 输出 | 创意方案（剧本/分镜/构图方案/素材清单） |
| 核心能力 | 剧本生成、分镜规划、中式叙事逻辑、素材匹配 |
| 依赖模型 | DeepSeek（剧本生成）、文化适配引擎（叙事校验） |

按场景差异化输出：

| 场景 | 输出内容 |
|------|----------|
| 图文 | 构图方案、元素布局、色调方案、文案 |
| 短剧 | 剧本（含台词/情绪/运镜标注）、分镜列表、角色设定、配乐方案 |
| 电影 | 长篇剧本、电影级分镜、角色详细设定、场景设计、特效方案 |
| 电视剧 | 多集剧本大纲、单集剧本、分镜批量方案、角色成长线 |

#### Agent 3：内容生成 Agent (Generator)

| 项目 | 说明 |
|------|------|
| 输入 | Planner 输出的创意方案 + 素材引用 |
| 输出 | 原始生成内容（图片/视频片段/音频） |
| 核心能力 | 图像生成、视频生成、音频生成、角色一致性控制 |
| 依赖模型 | SD（图像）、视频生成模型、TTS/音乐生成模型 |

#### Agent 4：编辑优化 Agent (Editor)

| 项目 | 说明 |
|------|------|
| 输入 | Generator 输出的原始内容 |
| 输出 | 优化后的内容（精修图片/剪辑视频/混音音频） |
| 核心能力 | 画面精修、视频剪辑、音画同步、特效合成、色调统一 |
| 依赖模型 | 超分模型、风格迁移模型、音视频处理工具链 |

#### Agent 5：审片校验 Agent (Reviewer)

| 项目 | 说明 |
|------|------|
| 输入 | Editor 输出的优化内容 |
| 输出 | 审核报告（通过/需修改/拒绝）+ 修改建议 |
| 核心能力 | 一致性检查、文化准确性校验、画面质量评估、合规检查 |
| 依赖 | 文化适配引擎、版权校验服务、内容审核服务 |

### 6.3 Agent编排工作流（Temporal）

```go
// 主工作流定义（伪代码）
func GenerationWorkflow(ctx workflow.Context, req GenerationRequest) (Result, error) {
    // 1. 需求解析
    parsedReq := workflow.ExecuteActivity(ctx, ParserAgent.Parse, req)

    // 2. 创意规划
    plan := workflow.ExecuteActivity(ctx, PlannerAgent.Plan, parsedReq)

    // 3. 用户确认（专业模式下等待用户审批方案）
    if req.Mode == "professional" {
        plan = workflow.ExecuteActivity(ctx, WaitForUserApproval, plan)
    }

    // 4. 内容生成（可并行生成多个片段）
    var segments []Segment
    for _, scene := range plan.Scenes {
        future := workflow.ExecuteActivity(ctx, GeneratorAgent.Generate, scene)
        segments = append(segments, future)
    }

    // 5. 编辑优化
    optimized := workflow.ExecuteActivity(ctx, EditorAgent.Optimize, segments)

    // 6. 审片校验（可能触发回退重新生成）
    review := workflow.ExecuteActivity(ctx, ReviewerAgent.Review, optimized)
    if review.NeedRevision {
        // 回退到对应Agent重新处理，最多3轮
        return workflow.ExecuteChildWorkflow(ctx, RevisionWorkflow, review.Feedback)
    }

    // 7. 版权校验 + 内容审核（并行）
    copyrightFuture := workflow.ExecuteActivity(ctx, CopyrightCheck, optimized)
    auditFuture := workflow.ExecuteActivity(ctx, ContentAudit, optimized)

    return FinalResult{Content: optimized, Copyright: copyrightFuture, Audit: auditFuture}, nil
}
```

### 6.4 Agent间通信协议

Agent间通过 Temporal Activity 传递结构化消息，格式统一：

```json
{
    "message_id": "uuid",
    "from_agent": "parser",
    "to_agent": "planner",
    "task_id": "gen_task_uuid",
    "timestamp": "2026-02-17T10:00:00Z",
    "payload_type": "parsed_requirement",
    "payload": { ... },
    "context_ref": "mongo_context_id"
}
```

共享上下文存储在 MongoDB `agent_contexts` 集合中，各 Agent 可读写共享的角色引用、场景引用、风格配置等信息，避免重复传递大体积数据。

---

## 7. 生成管线设计

### 7.1 四条生成管线总览

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

### 7.2 图文生成管线

```
输入(文本/图片/模板) → 需求解析 → 构图规划 → 图像生成 → 后处理 → 输出
                         │           │           │          │
                    文化语义识别   中式构图引擎  SD+LoRA   色调/水印
                    风格匹配      元素布局      ControlNet  格式转换
```

| 阶段 | 处理内容 | 关键技术 |
|------|----------|----------|
| 需求解析 | 提取主题/风格/元素/色调 | LLM 文本理解 + CLIP 图像理解 |
| 构图规划 | 生成构图方案（对称/留白/三分法） | 中式构图规则引擎 + LLM 布局生成 |
| 图像生成 | 生成主体图像 | Stable Diffusion + 国风 LoRA + ControlNet |
| 后处理 | 色调统一、文字排版、水印 | 图像处理工具链 + 国风色调 LUT |
| 输出 | 多格式/多分辨率导出 | ImageMagick / Sharp |

零基础模式简化流程：`选模板 → 填文字 → 一键生成 → 微调 → 导出`

### 7.3 短剧生成管线

```
输入 → 需求解析 → 剧本生成 → 分镜规划 → 素材准备 → 视频生成 → 音频合成 → 剪辑合成 → 审片 → 输出
                     │          │          │          │          │          │
                 中式叙事模板  运镜标注    角色生成    逐镜头生成  配音+配乐  音画同步
                 情节结构      场景标注    场景生成    一致性控制  国风音效   转场特效
```

| 阶段 | 处理内容 | 耗时预估（1min内容） |
|------|----------|---------------------|
| 剧本生成 | 生成含台词/情绪/运镜标注的剧本 | ~30s |
| 分镜规划 | 逐镜头分镜图 + 参数标注 | ~60s |
| 素材准备 | 角色模型生成/检索 + 场景匹配 | ~30s |
| 视频生成 | 逐镜头视频片段生成 | ~120s |
| 音频合成 | 配音(TTS) + 配乐 + 音效 | ~30s |
| 剪辑合成 | 片段拼接 + 转场 + 音画同步 | ~20s |
| 审片校验 | 一致性/文化准确性/质量检查 | ~10s |

### 7.4 电影生成管线

在短剧管线基础上增强：

| 增强点 | 说明 |
|--------|------|
| 长篇剧本 | 支持 90-120 分钟剧本，三幕式结构 + 中式叙事融合 |
| 高精度分镜 | 电影级运镜设计（航拍/长镜头/慢镜头），含光影参数 |
| 4K 生成 | 基础生成 1080p + 超分辨率至 4K |
| 特效管线 | 独立特效合成阶段（剑气/水墨过渡/古风粒子） |
| 分段生成 | 按场次分段生成，支持单场重新生成 |
| 专业审片 | 多维度评分（画面/叙事/一致性/文化），支持人工介入 |

### 7.5 电视剧生成管线

在电影管线基础上增强：

| 增强点 | 说明 |
|--------|------|
| 多集管理 | 剧本大纲→单集剧本→分镜，全局一致性管控 |
| 角色成长线 | 跨集角色状态追踪（服饰变化/年龄变化/关系演变） |
| 批量分镜 | 多集分镜批量生成，统一风格与运镜逻辑 |
| 片段优先 | 优先生成核心片段（名场面），用于预演与宣传 |
| 团队协同 | 多人分集编辑，冲突检测与合并 |

### 7.6 管线共享组件

| 组件 | 功能 | 技术 |
|------|------|------|
| 角色一致性引擎 | 跨镜头/跨集角色外观一致 | IP-Adapter + 角色特征向量锁定 |
| 音频引擎 | TTS配音 + 国风配乐 + 音效 | 语音合成模型 + 音乐生成模型 |
| 超分辨率模块 | 1080p → 4K 提升 | Real-ESRGAN / SwinIR |
| 视频编码器 | 格式转换与压缩 | FFmpeg + 硬件加速编码 |
| 进度追踪器 | 实时进度计算与推送 | Redis + WebSocket |

---

## 8. 文化适配引擎设计

### 8.1 引擎定位

文化适配引擎是苍梧的核心壁垒组件，贯穿生成全链路，负责确保所有生成内容的文化准确性与中式审美合理性。

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
                    │(MongoDB)│ │(规则引擎)│ │(Milvus) │
                    └─────────┘ └────────┘ └─────────┘
```

### 8.2 三大核心模块

#### 8.2.1 文化识别模块

负责从用户输入中识别文化相关语义，输出结构化文化标签。

```
用户输入 → 文化关键词提取 → 朝代识别 → 风格分类 → 元素标注 → 文化标签集
```

| 识别维度 | 示例 | 技术手段 |
|----------|------|----------|
| 朝代识别 | "唐朝" / "大明" / 从服饰图片识别朝代 | NER + 图像分类模型 |
| 风格分类 | 水墨风 / 工笔画 / 敦煌风 / 武侠风 | 文本分类 + CLIP 风格匹配 |
| 元素识别 | 汉服 / 园林 / 油纸伞 / 回纹 | 实体识别 + 目标检测 |
| 叙事类型 | 家国情怀 / 江湖侠义 / 古典爱情 | 文本意图分类 |
| 色彩体系 | 黛青 / 朱红 / 鎏金 | 关键词匹配 + 色彩模型 |

#### 8.2.2 文化校验模块

基于规则引擎 + 知识图谱，校验生成内容的文化准确性。

校验规则分类：

| 规则类别 | 校验内容 | 严重级别 |
|----------|----------|----------|
| 服饰形制 | 朝代与服饰款式是否匹配（如唐制不应出现明制立领） | 严重 |
| 建筑结构 | 建筑风格与朝代/地域是否一致 | 严重 |
| 礼仪规范 | 人物行为是否符合对应朝代礼仪 | 中等 |
| 道具合理性 | 道具是否符合时代背景（如宋代不应出现辣椒） | 中等 |
| 色彩合理性 | 色彩搭配是否符合中式审美 | 轻微 |
| 纹样匹配 | 纹样使用是否符合身份/场合 | 中等 |

校验流程：
```
生成内容 → 元素提取 → 规则匹配 → 知识图谱验证 → 校验报告
                                                    │
                                          ┌─────────┼─────────┐
                                          ▼         ▼         ▼
                                        通过     警告(建议修改)  拒绝(必须修改)
```

#### 8.2.3 文化匹配模块

根据文化标签，从素材库中匹配最合适的素材、模板与风格参数。

```
文化标签集 → 向量编码 → Milvus相似度检索 → 候选排序 → 最优匹配
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                素材匹配    模板匹配    风格参数匹配
              (角色/场景/道具) (剧本/构图) (色调/LoRA/运镜)
```

### 8.3 文化知识库

```json
// collection: cultural_knowledge
{
    "_id": ObjectId,
    "category": "costume",
    "dynasty": "tang",
    "name": "唐制齐胸襦裙",
    "description": "唐代女性常见服饰，高腰束于胸上...",
    "attributes": {
        "gender": "female",
        "occasion": ["日常", "宴会"],
        "social_class": ["贵族", "平民"],
        "key_features": ["齐胸束带", "广袖", "长裙曳地"],
        "forbidden_combinations": ["立领", "马面裙", "盘扣"]
    },
    "visual_refs": ["oss://knowledge/tang_qixiong_01.jpg"],
    "sources": ["《新唐书·车服志》", "敦煌壁画"],
    "embedding_id": "milvus_id"
}
```

### 8.4 朝代特征矩阵

| 朝代 | 服饰特征 | 建筑特征 | 色彩偏好 | 典型元素 |
|------|----------|----------|----------|----------|
| 汉 | 曲裾深衣、直裾 | 高台建筑、阙楼 | 玄黑、朱红 | 玉璧、博山炉 |
| 唐 | 齐胸襦裙、圆领袍 | 斗拱飞檐、大明宫式 | 鎏金、石榴红 | 团扇、铜镜 |
| 宋 | 褙子、直裰 | 素雅园林、瓦舍 | 天青、月白 | 折扇、香炉 |
| 明 | 立领袄裙、飞鱼服 | 硬山顶、牌坊 | 大红、明黄 | 补子、玉带 |
| 清 | 旗装、马褂 | 满式宫殿、圆明园式 | 宝蓝、明黄 | 朝珠、如意 |

---

## 9. 素材库系统设计

### 9.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    素材库系统 (asset-svc)                 │
├────────────┬────────────┬────────────┬──────────────────┤
│  上传管理   │  存储管理   │  检索引擎   │  版权校验         │
└─────┬──────┴─────┬──────┴─────┬──────┴───────┬──────────┘
      │            │            │              │
      ▼            ▼            ▼              ▼
  ┌────────┐  ┌────────┐  ┌────────────┐  ┌────────┐
  │ 预处理  │  │  OSS   │  │ ES + Milvus│  │版权知识库│
  │ Pipeline│  │ 对象存储│  │ 双引擎检索  │  │        │
  └────────┘  └────────┘  └────────────┘  └────────┘
```

### 9.2 素材分类体系

```
素材库
├── 国风元素库
│   ├── 服饰 (costume)：汉服/唐装/官服/戏服/民族服饰
│   ├── 场景 (scene)：园林/古城/皇宫/书院/山水/街市
│   ├── 道具 (prop)：毛笔/油纸伞/折扇/玉佩/灯笼/古琴
│   └── 色调 (palette)：黛青/朱红/雅灰/鎏金/天青/月白
├── 传统文化库
│   ├── 典故模板 (allusion)：三顾茅庐/卧薪尝胆/嫦娥奔月
│   ├── 节日元素 (festival)：春节/端午/中秋/重阳/元宵
│   ├── 非遗元素 (heritage)：剪纸/刺绣/皮影/年画/漆器
│   ├── 纹样 (pattern)：回纹/云纹/牡丹纹/龙凤纹/缠枝纹
│   └── 诗词配图 (poetry)：唐诗/宋词/元曲配图模板
├── 影视专属库
│   ├── 分镜模板 (storyboard)：武侠/宫廷/田园/战争
│   ├── 剧本模板 (script)：家国情怀/江湖侠义/古典爱情
│   ├── 角色模型 (character)：书生/侠客/将军/仕女/帝王
│   └── 场景模型 (scene_model)：古战场/皇宫大殿/江南水乡
└── 用户自定义库
    ├── 上传素材 (user_upload)
    └── 生成素材 (user_generated)
```

### 9.3 存储方案

| 数据类型 | 存储位置 | 说明 |
|----------|----------|------|
| 原始文件 | OSS `assets/original/` | 原始高清素材，按类型分桶 |
| 缩略图 | OSS `assets/thumb/` | 自动生成多尺寸缩略图 (128/256/512) |
| 预览图 | OSS `assets/preview/` | 中等质量预览 |
| 元数据 | MongoDB `assets` | 名称/标签/分类/版权等结构化信息 |
| 搜索索引 | Elasticsearch | 全文检索索引（名称/标签/描述） |
| 语义向量 | Milvus | CLIP 编码的视觉语义向量 |

OSS 存储策略：
- 热数据（近30天高频访问）：标准存储
- 温数据（30-180天）：低频访问存储
- 冷数据（>180天未访问）：归档存储
- CDN 加速：缩略图与预览图全量缓存

### 9.4 检索引擎

支持三种检索模式：

```
┌──────────────────────────────────────────────┐
│                 检索入口                       │
├──────────────┬───────────────┬────────────────┤
│  关键词检索   │   标签筛选     │   语义检索      │
│  (ES全文)    │  (ES过滤)     │  (Milvus向量)  │
└──────┬───────┴───────┬───────┴───────┬────────┘
       │               │               │
       └───────────────┼───────────────┘
                       ▼
                  融合排序 (RRF)
                       │
                       ▼
                  结果返回（含相关度评分）
```

| 检索模式 | 适用场景 | 技术 |
|----------|----------|------|
| 关键词检索 | 精确搜索"唐代齐胸襦裙" | ES BM25 + 中文分词(IK) |
| 标签筛选 | 按朝代/类型/风格组合筛选 | ES Term Query + Aggregation |
| 语义检索 | 模糊描述"一个古风女子撑伞走在雨巷" | CLIP编码 → Milvus ANN检索 |
| 混合检索 | 关键词 + 语义融合 | RRF (Reciprocal Rank Fusion) |

### 9.5 素材上传与预处理

```
用户上传 → 格式校验 → 病毒扫描 → 版权预检 → 内容审核 → 预处理 → 入库
                                                          │
                                              ┌───────────┼───────────┐
                                              ▼           ▼           ▼
                                          生成缩略图   CLIP编码     元数据提取
                                          多尺寸裁剪   向量入Milvus  EXIF/尺寸/格式
```

### 9.6 版权校验机制

| 校验阶段 | 校验内容 | 处理方式 |
|----------|----------|----------|
| 素材入库 | 上传素材与版权库比对 | 感知哈希(pHash) + 特征向量相似度 |
| 生成前 | 引用素材的版权状态检查 | 查询素材版权记录，过期素材拦截 |
| 生成后 | 生成内容与版权库比对 | 视觉相似度检索，输出风险评分 |
| 导出时 | 商用场景二次确认 | 版权证明生成，水印嵌入 |

版权风险评分：
- 0-30：安全，可直接使用
- 31-70：低风险，建议人工复核
- 71-100：高风险，拦截并提示用户

---

## 10. 用户与权限系统

### 10.1 账号体系

```
┌─────────────────────────────────────────────┐
│                 账号体系                      │
├─────────────┬───────────────┬───────────────┤
│  个人账号    │   企业账号     │  专业团队账号   │
│  (免费/付费) │  (企业认证)    │  (团队协同)    │
├─────────────┼───────────────┼───────────────┤
│ · 个人创作   │ · 企业认证     │ · 多成员协同   │
│ · 基础功能   │ · 批量生成     │ · 角色分工     │
│ · 个人素材库 │ · 商用版权     │ · 影视级功能   │
│             │ · 企业素材库   │ · 4K输出       │
└─────────────┴───────────────┴───────────────┘
```

### 10.2 认证方式

| 认证方式 | 说明 | 适用场景 |
|----------|------|----------|
| 手机号 + 验证码 | 主要注册/登录方式 | 国内用户 |
| 邮箱 + 密码 | 备选登录方式 | 企业用户/国际用户 |
| 微信 OAuth | 第三方快捷登录 | 移动端用户 |
| 微博 OAuth | 第三方快捷登录 | 社交用户 |
| API Key + HMAC | 开放API认证 | 第三方集成 |

Token 机制：
- Access Token：JWT，有效期 2 小时，含用户ID/角色/套餐信息
- Refresh Token：不透明令牌，有效期 30 天，存储于 Redis
- Token 刷新：Access Token 过期后通过 Refresh Token 无感刷新

### 10.3 RBAC 权限模型

```
用户 (User) ──N:M──► 角色 (Role) ──N:M──► 权限 (Permission)
                        │
                   可绑定组织 (Org)
                   实现组织级角色隔离
```

#### 预置角色

| 角色 | 代码 | 权限范围 |
|------|------|----------|
| 平台管理员 | `platform_admin` | 全局管理：用户/内容/素材/系统配置 |
| 个人用户 | `personal_user` | 个人项目 CRUD、生成任务、个人素材库 |
| 企业管理员 | `org_admin` | 企业成员管理、企业素材库、批量生成、商用版权 |
| 创作者 | `creator` | 项目创建/编辑、生成任务、素材使用 |
| 审核员 | `reviewer` | 内容审核、审片校验、版权复核 |
| API 用户 | `api_user` | 开放API调用、配额管理 |

#### 权限粒度

```
权限 = 资源 (Resource) + 操作 (Action)

资源：project / task / asset / template / user / org / subscription
操作：create / read / update / delete / execute / export / publish

示例：
  project:create      - 创建项目
  task:execute         - 执行生成任务
  asset:upload         - 上传素材
  task:export:4k       - 导出4K内容（专业版权限）
  org:member:manage    - 管理企业成员
```

### 10.4 分层适配与功能暴露

| 用户模式 | 界面复杂度 | 可见功能 | 隐藏功能 |
|----------|-----------|----------|----------|
| 零基础 | 极简 | 模板选择、一键生成、简单编辑、基础导出 | 参数调整、分镜编辑、批量生成、4K、团队协同 |
| 进阶 | 中等 | +批量生成、角色定制、剧本修改、爆款分析、多平台分发 | 影视级分镜、4K输出、团队协同 |
| 专业 | 完整 | 全部功能开放 | 无 |

模式切换逻辑：
- 用户可在设置中手动切换模式
- 系统根据订阅套餐自动推荐默认模式
- 零基础模式下提供"解锁更多"入口，引导升级

### 10.5 团队协同权限

```
组织 (Organization)
├── 管理员 (org_admin)：成员管理、权限分配、企业素材库管理
├── 创作者 (creator)：项目创建/编辑、生成任务
└── 审核员 (reviewer)：审片、内容审核

项目级权限：
├── 所有者 (owner)：完全控制
├── 编辑者 (editor)：编辑项目内容
└── 查看者 (viewer)：只读访问
```

协同编辑冲突解决：
- 基于 OT (Operational Transformation) 算法实现剧本实时协同编辑
- 分镜编辑采用锁定机制：编辑时锁定单个分镜，避免冲突
- 版本快照：每次重大编辑自动创建版本，支持回滚

---

## 11. 基础设施与部署

### 11.1 云架构总览

基于阿里云构建，核心区域部署华东2（上海），灾备区域华北2（北京）。

```
┌─────────────────────────────────────────────────────────────────┐
│                        阿里云 (Alibaba Cloud)                    │
│                                                                  │
│  ┌──────────┐    ┌──────────────────────────────────────────┐   │
│  │ CDN      │    │          VPC (生产环境)                    │   │
│  │ 全球加速  │    │                                           │   │
│  └────┬─────┘    │  ┌─────────────────────────────────┐     │   │
│       │          │  │        公网子网                    │     │   │
│       │          │  │  ┌─────────┐  ┌──────────────┐  │     │   │
│       └──────────┼─►│  │ SLB/ALB │  │ API网关(APISIX)│  │     │   │
│                  │  │  └────┬────┘  └──────┬───────┘  │     │   │
│                  │  └───────┼──────────────┼──────────┘     │   │
│                  │          │              │                  │   │
│                  │  ┌───────┼──────────────┼──────────┐     │   │
│                  │  │       ▼  应用子网     ▼          │     │   │
│                  │  │  ┌────────────────────────┐     │     │   │
│                  │  │  │   ACK (Kubernetes集群)  │     │     │   │
│                  │  │  │                        │     │     │   │
│                  │  │  │  ┌──────┐ ┌──────┐    │     │     │   │
│                  │  │  │  │业务Pod│ │业务Pod│... │     │     │   │
│                  │  │  │  └──────┘ └──────┘    │     │     │   │
│                  │  │  └────────────────────────┘     │     │   │
│                  │  └────────────────────────────────┘     │   │
│                  │                                          │   │
│                  │  ┌──────────────────────────────────┐   │   │
│                  │  │        数据子网                    │   │   │
│                  │  │  ┌─────┐ ┌──────┐ ┌─────┐       │   │   │
│                  │  │  │ PG  │ │Mongo │ │Redis│       │   │   │
│                  │  │  │ RDS │ │      │ │集群  │       │   │   │
│                  │  │  └─────┘ └──────┘ └─────┘       │   │   │
│                  │  │  ┌─────┐ ┌──────┐ ┌──────┐      │   │   │
│                  │  │  │ ES  │ │Milvus│ │Kafka │      │   │   │
│                  │  │  └─────┘ └──────┘ └──────┘      │   │   │
│                  │  └──────────────────────────────────┘   │   │
│                  │                                          │   │
│                  │  ┌──────────────────────────────────┐   │   │
│                  │  │        GPU子网                     │   │   │
│                  │  │  ┌──────────────────────────┐    │   │   │
│                  │  │  │  GPU节点池 (A100/A10)     │    │   │   │
│                  │  │  │  Triton推理服务            │    │   │   │
│                  │  │  │  模型仓库 (OSS挂载)        │    │   │   │
│                  │  │  └──────────────────────────┘    │   │   │
│                  │  └──────────────────────────────────┘   │   │
│                  └──────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │ OSS      │  │ SLS日志   │  │ 云监控    │                      │
│  │ 对象存储  │  │          │  │ ARMS     │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
└─────────────────────────────────────────────────────────────────┘
```

### 11.2 Kubernetes 集群规划

| 节点池 | 机型 | 数量 | 用途 |
|--------|------|------|------|
| 业务节点池 | ecs.g7.2xlarge (8C32G) | 6-20 | 业务微服务 |
| GPU推理池 | ecs.gn7i-c16g1.4xlarge (A10) | 4-16 | 模型推理（常规） |
| GPU训练池 | ecs.gn7-c12g1.3xlarge (A100) | 2-8 | 高精度生成/微调 |
| 中间件节点池 | ecs.r7.2xlarge (8C64G) | 3-6 | Kafka/ES/Milvus |

### 11.3 容器编排与服务部署

```yaml
# 示例：gen-svc Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gen-svc
  namespace: tsangwu
spec:
  replicas: 3
  strategy:
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    spec:
      containers:
      - name: gen-svc
        image: registry.tsangwu.com/gen-svc:v1.0.0
        resources:
          requests: { cpu: "500m", memory: "1Gi" }
          limits:   { cpu: "2000m", memory: "4Gi" }
        livenessProbe:
          httpGet: { path: /healthz, port: 8080 }
        readinessProbe:
          httpGet: { path: /readyz, port: 8080 }
```

### 11.4 弹性伸缩策略

| 服务 | 伸缩指标 | 最小副本 | 最大副本 | 触发阈值 |
|------|----------|----------|----------|----------|
| 业务服务 (user/proj/asset) | CPU利用率 | 2 | 10 | CPU > 70% |
| gen-svc | Kafka消费延迟 | 3 | 20 | lag > 100 |
| agent-svc | Temporal活跃工作流数 | 3 | 15 | workflows > 50 |
| GPU推理 (Triton) | GPU利用率 + 队列深度 | 2 | 16 | GPU > 80% 或 queue > 20 |

GPU 弹性伸缩特殊策略：
- 预热池：保持 2 个 GPU 节点常驻，避免冷启动延迟
- 定时扩容：工作日 9:00-22:00 自动扩容至基准的 1.5 倍
- 突发扩容：队列深度超阈值时 3 分钟内扩容新节点
- 缩容冷却：缩容前等待 15 分钟，避免频繁伸缩

### 11.5 CI/CD 流水线

```
代码提交 → GitLab CI → 单元测试 → 代码扫描 → 构建镜像 → 推送Registry
                                                              │
                                                              ▼
                                              ArgoCD 监听 → 自动同步
                                                              │
                                              ┌───────────────┼───────────┐
                                              ▼               ▼           ▼
                                           dev环境         staging环境   prod环境
                                          (自动部署)      (自动部署)    (审批后部署)
```

环境策略：
- dev：每次 MR 合并自动部署，用于开发联调
- staging：release 分支自动部署，用于集成测试与验收
- prod：打 tag 触发，需人工审批后灰度发布（金丝雀 → 全量）

---

## 12. 安全与合规

### 12.1 安全架构总览

```
┌─────────────────────────────────────────────────────────┐
│                     安全防护体系                          │
├──────────────┬──────────────┬──────────────┬────────────┤
│   网络安全    │   数据安全    │   应用安全    │  内容安全   │
├──────────────┼──────────────┼──────────────┼────────────┤
│ · WAF防护    │ · 传输加密   │ · 身份认证   │ · 内容审核  │
│ · DDoS防御   │ · 存储加密   │ · 权限控制   │ · 版权校验  │
│ · VPC隔离    │ · 数据脱敏   │ · API限流    │ · 敏感词过滤│
│ · 安全组     │ · 备份恢复   │ · 注入防护   │ · 人工复审  │
└──────────────┴──────────────┴──────────────┴────────────┘
```

### 12.2 数据安全

#### 传输安全

| 场景 | 方案 |
|------|------|
| 客户端 ↔ 服务端 | TLS 1.3，强制 HTTPS |
| 服务间通信 | mTLS（双向证书认证），通过 Istio Service Mesh 管理 |
| 数据库连接 | SSL 加密连接 |

#### 存储安全

| 数据类型 | 加密方案 |
|----------|----------|
| 用户密码 | bcrypt (cost=12) |
| 敏感字段（手机号/邮箱） | AES-256-GCM 应用层加密，密钥由 KMS 托管 |
| OSS 对象 | 服务端加密 (SSE-KMS) |
| 数据库 | TDE 透明数据加密 |

#### 数据备份

| 数据源 | 备份策略 | 保留周期 |
|--------|----------|----------|
| PostgreSQL | 每日全量 + 实时WAL归档 | 全量30天，WAL 7天 |
| MongoDB | 每日全量 + Oplog持续备份 | 30天 |
| Redis | RDB每日 + AOF实时 | 7天 |
| OSS | 跨区域复制 (CRR) | 永久 |

#### 数据隔离

- 用户上传素材：按用户ID隔离存储路径 `oss://user-data/{user_id}/`
- 企业数据：按组织ID隔离 `oss://org-data/{org_id}/`
- 用户素材不用于模型训练（除非用户明确授权）
- 删除账号时，30天内可恢复，30天后永久删除

### 12.3 内容审核

```
生成内容 → 机器审核 → 判定
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
            通过     需人工复审    拒绝
              │         │         │
              ▼         ▼         ▼
           直接输出   人工审核队列  拦截+通知用户
                        │
                   ┌────┼────┐
                   ▼         ▼
                 通过       拒绝
```

审核维度：

| 维度 | 检测内容 | 处理方式 |
|------|----------|----------|
| 涉政 | 敏感政治人物/事件/标语 | 拦截 |
| 涉黄 | 色情/低俗内容 | 拦截 |
| 暴力 | 血腥/恐怖/极端暴力 | 拦截 |
| 违禁 | 毒品/武器/赌博相关 | 拦截 |
| 侵权 | 明星肖像/品牌商标 | 人工复审 |
| 文化敏感 | 宗教/民族/历史敏感内容 | 人工复审 |

### 12.4 版权保护体系

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

- 数字水印：生成内容嵌入不可见水印（DWT域水印），含任务ID/用户ID/时间戳
- 版权证明：商用版自动生成版权证明文件（PDF），含生成参数、素材来源、授权信息
- 侵权检测：基于感知哈希 + 深度特征的相似度检测，对接第三方版权库

### 12.5 合规要求

| 法规 | 合规措施 |
|------|----------|
| 《数据安全法》 | 数据分级分类管理，重要数据境内存储 |
| 《个人信息保护法》 | 最小必要原则采集，用户授权管理，数据可携带/可删除 |
| 《生成式AI管理办法》 | 内容审核前置，训练数据合规审查，生成内容标识 |
| 《网络安全法》 | 等保三级，安全事件应急响应，日志留存≥6个月 |

---

## 13. 监控与可观测性

### 13.1 可观测性三支柱

```
┌─────────────────────────────────────────────────────────┐
│                    可观测性平台                           │
├──────────────────┬──────────────────┬───────────────────┤
│      日志         │      指标         │     链路追踪      │
│    (Logging)     │    (Metrics)     │    (Tracing)     │
├──────────────────┼──────────────────┼───────────────────┤
│ ELK Stack        │ Prometheus       │ OpenTelemetry     │
│ · Filebeat采集   │ · 服务指标       │ · 自动埋点        │
│ · Logstash处理   │ · 业务指标       │ · 跨服务传播      │
│ · ES存储检索     │ · 基础设施指标   │ · Jaeger存储      │
│ · Kibana可视化   │ · Grafana面板    │ · 拓扑分析        │
└──────────────────┴──────────────────┴───────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   告警中心    │
                    │ Alertmanager │
                    │ + 飞书/钉钉   │
                    └──────────────┘
```

### 13.2 日志体系

#### 日志分级

| 级别 | 用途 | 示例 |
|------|------|------|
| ERROR | 需要立即关注的错误 | 生成任务失败、数据库连接断开 |
| WARN | 潜在问题 | GPU队列积压、API限流触发 |
| INFO | 关键业务事件 | 用户注册、任务创建、生成完成 |
| DEBUG | 调试信息（仅dev/staging） | Agent中间状态、模型推理参数 |

#### 日志格式（结构化JSON）

```json
{
    "timestamp": "2026-02-17T10:30:00.123Z",
    "level": "INFO",
    "service": "gen-svc",
    "trace_id": "abc123def456",
    "span_id": "span789",
    "user_id": "user_001",
    "message": "generation task completed",
    "fields": {
        "task_id": "task_uuid",
        "scene_type": "short_drama",
        "duration_ms": 185000,
        "gpu_node": "gpu-node-03"
    }
}
```

#### 日志保留策略

| 环境 | 保留周期 | 存储 |
|------|----------|------|
| prod | 180天（合规要求≥6个月） | ES热节点30天 → 温节点150天 → OSS归档 |
| staging | 30天 | ES |
| dev | 7天 | ES |

### 13.3 指标体系

#### 基础设施指标

| 指标 | 采集方式 | 告警阈值 |
|------|----------|----------|
| CPU利用率 | node_exporter | > 85% 持续5min |
| 内存利用率 | node_exporter | > 90% |
| 磁盘使用率 | node_exporter | > 85% |
| GPU利用率 | dcgm_exporter | > 95% 持续10min |
| GPU显存使用 | dcgm_exporter | > 90% |
| Pod重启次数 | kube-state-metrics | > 3次/小时 |

#### 业务指标

| 指标 | 说明 | 告警阈值 |
|------|------|----------|
| `gen_task_total` | 生成任务总数（按类型/状态） | - |
| `gen_task_duration_seconds` | 生成任务耗时分布 | P99 > SLA 2倍 |
| `gen_task_failure_rate` | 生成任务失败率 | > 5% |
| `gen_queue_depth` | 生成队列深度 | > 200 |
| `api_request_total` | API请求总数（按端点/状态码） | - |
| `api_latency_seconds` | API响应延迟 | P99 > 3s |
| `api_error_rate` | API错误率（5xx） | > 1% |
| `active_users` | 在线活跃用户数 | - |
| `asset_search_latency` | 素材检索延迟 | P99 > 500ms |
| `culture_check_accuracy` | 文化校验准确率 | < 85% |
| `copyright_risk_count` | 版权风险拦截数 | 日环比 > 200% |

#### Grafana 核心面板

| 面板 | 包含内容 |
|------|----------|
| 系统总览 | QPS、错误率、延迟P50/P99、活跃用户、任务队列 |
| 生成管线 | 各管线任务数/耗时/成功率、GPU利用率、队列深度 |
| Agent监控 | 各Agent执行耗时/成功率、工作流状态分布、重试次数 |
| 业务指标 | 注册量、生成量、付费转化、素材使用TOP |
| 基础设施 | 节点资源、Pod状态、中间件健康度 |

### 13.4 链路追踪

基于 OpenTelemetry SDK 自动埋点，覆盖全链路：

```
用户请求 → API网关 → 业务服务 → Agent编排 → 模型推理 → 后处理 → 响应
   │          │          │          │           │          │
   span1     span2      span3     span4       span5     span6
   └──────────────────── trace_id ─────────────────────────┘
```

关键追踪点：
- HTTP/gRPC 入口出口自动埋点
- Kafka 生产消费 trace 传播
- Temporal Workflow/Activity trace 关联
- GPU 推理耗时标记
- 外部模型 API 调用耗时与状态

### 13.5 告警策略

| 告警级别 | 响应时间 | 通知方式 | 示例场景 |
|----------|----------|----------|----------|
| P0 - 致命 | 5分钟内 | 电话 + 飞书 + 短信 | 服务不可用、数据库宕机、全量生成失败 |
| P1 - 严重 | 15分钟内 | 飞书 + 短信 | 错误率飙升、GPU节点故障、队列严重积压 |
| P2 - 警告 | 1小时内 | 飞书群通知 | 延迟升高、磁盘空间不足、证书即将过期 |
| P3 - 提示 | 下个工作日 | 飞书群通知 | 非核心服务异常、日志量异常增长 |

告警收敛规则：
- 相同告警 5 分钟内去重
- 同一服务多个实例告警合并
- 告警恢复自动发送恢复通知
- 夜间（23:00-8:00）P2/P3 告警静默，P0/P1 正常通知

---

> 本文档为苍梧系统详细设计方案 v1.0，将随技术验证与实施进展持续迭代更新。
