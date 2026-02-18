-- 苍梧系统 SQLite 初始化迁移（本地模式）

CREATE TABLE IF NOT EXISTS users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    uid             TEXT NOT NULL UNIQUE,
    phone           TEXT UNIQUE,
    email           TEXT UNIQUE,
    nickname        TEXT NOT NULL,
    avatar_url      TEXT,
    account_type    INTEGER NOT NULL DEFAULT 0,
    status          INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS organizations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    license_no      TEXT,
    owner_id        INTEGER NOT NULL REFERENCES users(id),
    max_members     INTEGER NOT NULL DEFAULT 5,
    status          INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS roles (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    description     TEXT
);

CREATE TABLE IF NOT EXISTS permissions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT NOT NULL UNIQUE,
    resource        TEXT NOT NULL,
    action          TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    uid             TEXT NOT NULL UNIQUE,
    owner_id        INTEGER NOT NULL REFERENCES users(id),
    org_id          INTEGER REFERENCES organizations(id),
    title           TEXT NOT NULL,
    scene_type      INTEGER NOT NULL,
    user_mode       INTEGER NOT NULL DEFAULT 0,
    status          INTEGER NOT NULL DEFAULT 0,
    config          TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS generation_tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    uid             TEXT NOT NULL UNIQUE,
    project_id      INTEGER NOT NULL REFERENCES projects(id),
    user_id         INTEGER NOT NULL REFERENCES users(id),
    task_type       INTEGER NOT NULL,
    priority        INTEGER NOT NULL DEFAULT 1,
    status          INTEGER NOT NULL DEFAULT 0,
    input_params    TEXT NOT NULL,
    output_urls     TEXT,
    progress        INTEGER DEFAULT 0,
    error_msg       TEXT,
    started_at      TEXT,
    completed_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    plan_code       TEXT NOT NULL,
    status          INTEGER NOT NULL DEFAULT 1,
    started_at      TEXT NOT NULL,
    expires_at      TEXT NOT NULL,
    auto_renew      INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS documents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    collection      TEXT NOT NULL,
    data            TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_documents_collection ON documents(collection);
