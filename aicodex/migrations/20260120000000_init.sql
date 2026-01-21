-- aicodex 数据库初始化迁移
-- 创建所有必需的表

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_configs (
    id              TEXT PRIMARY KEY,
    config_key      TEXT NOT NULL UNIQUE,
    config_value    TEXT NOT NULL,
    is_encrypted    INTEGER NOT NULL DEFAULT 0,
    value_type      TEXT NOT NULL DEFAULT 'string',
    description     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 仓库映射表
CREATE TABLE IF NOT EXISTS repo_mappings (
    id                  TEXT PRIMARY KEY,
    gitea_repo          TEXT NOT NULL UNIQUE,  -- owner/repo 格式
    local_path          TEXT NOT NULL,          -- 本地仓库路径
    vibe_project_id     TEXT,                   -- vibe-kanban 项目 ID
    agent_type          TEXT NOT NULL DEFAULT 'codex',
    executor_profile_id TEXT,
    custom_prompt       TEXT,
    is_enabled          INTEGER NOT NULL DEFAULT 1,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 审核运行记录
CREATE TABLE IF NOT EXISTS review_runs (
    id                  TEXT PRIMARY KEY,
    repo_mapping_id     TEXT NOT NULL,
    gitea_pr_number     INTEGER NOT NULL,
    gitea_pr_url        TEXT,
    vibe_task_id        TEXT,                   -- vibe-kanban 任务 ID
    vibe_workspace_id   TEXT,                   -- vibe-kanban 工作空间 ID
    status              TEXT NOT NULL DEFAULT 'pending',
    started_at          TEXT,
    completed_at        TEXT,
    error_message       TEXT,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (repo_mapping_id) REFERENCES repo_mappings(id)
);

-- 审核事件表
CREATE TABLE IF NOT EXISTS review_events (
    id              TEXT PRIMARY KEY,
    review_run_id   TEXT NOT NULL,
    event_type      TEXT NOT NULL,
    event_data      TEXT,  -- JSON
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (review_run_id) REFERENCES review_runs(id)
);

-- Webhook 审计日志
CREATE TABLE IF NOT EXISTS webhook_audits (
    id              TEXT PRIMARY KEY,
    gitea_repo      TEXT NOT NULL,
    event_type      TEXT NOT NULL,
    payload_hash    TEXT NOT NULL,
    status          TEXT NOT NULL,
    error_message   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_system_configs_key ON system_configs(config_key);
CREATE INDEX IF NOT EXISTS idx_repo_mappings_gitea_repo ON repo_mappings(gitea_repo);
CREATE INDEX IF NOT EXISTS idx_review_runs_status ON review_runs(status);
CREATE INDEX IF NOT EXISTS idx_review_runs_repo ON review_runs(repo_mapping_id);
CREATE INDEX IF NOT EXISTS idx_review_events_review_run ON review_events(review_run_id);
CREATE INDEX IF NOT EXISTS idx_webhook_audits_repo ON webhook_audits(gitea_repo);
CREATE INDEX IF NOT EXISTS idx_webhook_audits_hash ON webhook_audits(payload_hash);
