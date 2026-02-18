-- 生成任务、订阅、用量、版权、审核

CREATE TABLE generation_tasks (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    project_id      BIGINT NOT NULL REFERENCES projects(id),
    user_id         BIGINT NOT NULL REFERENCES users(id),
    task_type       SMALLINT NOT NULL,
    priority        SMALLINT NOT NULL DEFAULT 1,
    status          SMALLINT NOT NULL DEFAULT 0,
    input_params    JSONB NOT NULL,
    output_urls     JSONB,
    progress        SMALLINT DEFAULT 0,
    error_msg       TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_gen_tasks_user ON generation_tasks(user_id, status);
CREATE INDEX idx_gen_tasks_project ON generation_tasks(project_id);

CREATE TABLE subscriptions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    plan_code       VARCHAR(32) NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 1,
    started_at      TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    auto_renew      BOOLEAN DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE usage_records (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    task_id         BIGINT REFERENCES generation_tasks(id),
    usage_type      VARCHAR(32) NOT NULL,
    quantity        INT NOT NULL DEFAULT 1,
    credits_cost    INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_usage_user_date ON usage_records(user_id, created_at);

CREATE TABLE copyright_records (
    id              BIGSERIAL PRIMARY KEY,
    task_id         BIGINT NOT NULL REFERENCES generation_tasks(id),
    check_status    SMALLINT NOT NULL DEFAULT 0,
    risk_score      DECIMAL(5,2),
    risk_details    JSONB,
    certificate_url VARCHAR(512),
    checked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE content_reviews (
    id              BIGSERIAL PRIMARY KEY,
    task_id         BIGINT NOT NULL REFERENCES generation_tasks(id),
    review_type     SMALLINT NOT NULL,
    status          SMALLINT NOT NULL DEFAULT 0,
    risk_labels     JSONB,
    reviewer_id     BIGINT REFERENCES users(id),
    reviewed_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 文档存储表（PG JSONB 降级方案）
CREATE TABLE documents (
    id              BIGSERIAL PRIMARY KEY,
    collection      VARCHAR(128) NOT NULL,
    data            JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_documents_collection ON documents(collection);
CREATE INDEX idx_documents_data ON documents USING GIN(data);
