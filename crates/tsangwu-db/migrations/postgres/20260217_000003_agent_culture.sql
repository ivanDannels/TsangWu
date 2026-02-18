-- Agent服务表
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    agent_type TEXT NOT NULL,
    status TEXT NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_project_id ON agents(project_id);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);

-- Agent任务表
CREATE TABLE IF NOT EXISTS agent_tasks (
    id UUID PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL,
    input JSONB NOT NULL,
    output JSONB,
    status TEXT NOT NULL,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_agent_tasks_agent_id ON agent_tasks(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_tasks_status ON agent_tasks(status);

-- 朝代表
CREATE TABLE IF NOT EXISTS dynasties (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    period TEXT NOT NULL,
    start_year INTEGER NOT NULL,
    end_year INTEGER NOT NULL,
    description TEXT NOT NULL,
    cultural_features TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dynasties_name ON dynasties(name);
CREATE INDEX IF NOT EXISTS idx_dynasties_period ON dynasties(start_year, end_year);

-- 文化元素表
CREATE TABLE IF NOT EXISTS culture_elements (
    id UUID PRIMARY KEY,
    dynasty_id UUID NOT NULL REFERENCES dynasties(id) ON DELETE CASCADE,
    element_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    tags TEXT[] NOT NULL,
    reference_images TEXT[] NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_culture_elements_dynasty_id ON culture_elements(dynasty_id);
CREATE INDEX IF NOT EXISTS idx_culture_elements_type ON culture_elements(element_type);
CREATE INDEX IF NOT EXISTS idx_culture_elements_tags ON culture_elements USING GIN(tags);

-- 文化验证规则表
CREATE TABLE IF NOT EXISTS culture_validation_rules (
    id UUID PRIMARY KEY,
    dynasty_id UUID NOT NULL REFERENCES dynasties(id) ON DELETE CASCADE,
    rule_type TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL,
    pattern TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_culture_rules_dynasty_id ON culture_validation_rules(dynasty_id);
CREATE INDEX IF NOT EXISTS idx_culture_rules_type ON culture_validation_rules(rule_type);
