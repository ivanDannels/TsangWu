-- 苍梧系统 PostgreSQL 初始化迁移
-- 用户与权限

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
    PRIMARY KEY (user_id, role_id, COALESCE(org_id, 0))
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

-- 项目与创作

CREATE TABLE projects (
    id              BIGSERIAL PRIMARY KEY,
    uid             UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    owner_id        BIGINT NOT NULL REFERENCES users(id),
    org_id          BIGINT REFERENCES organizations(id),
    title           VARCHAR(256) NOT NULL,
    scene_type      SMALLINT NOT NULL,
    user_mode       SMALLINT NOT NULL DEFAULT 0,
    status          SMALLINT NOT NULL DEFAULT 0,
    config          JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE project_versions (
    id              BIGSERIAL PRIMARY KEY,
    project_id      BIGINT NOT NULL REFERENCES projects(id),
    version         INT NOT NULL,
    snapshot        JSONB NOT NULL,
    created_by      BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, version)
);
