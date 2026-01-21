-- 多实例管理系统数据库迁移
-- 为 aicodex 添加用户管理和多 vibe-kanban 实例管理能力

-- ==============================================
-- 1. vibe_instances 表 (先创建，因为被其他表引用)
-- ==============================================
CREATE TABLE IF NOT EXISTS vibe_instances (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    port INTEGER NOT NULL UNIQUE,
    data_dir TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'stopped',
    auto_start INTEGER NOT NULL DEFAULT 1,
    max_users INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_health_check TEXT,
    health_status TEXT DEFAULT 'unknown'
);

-- ==============================================
-- 2. users 表
-- ==============================================
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    display_name TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    current_instance_id TEXT REFERENCES vibe_instances(id) ON DELETE SET NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login_at TEXT
);

-- ==============================================
-- 3. user_sessions 表
-- ==============================================
CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_address TEXT,
    user_agent TEXT
);

-- ==============================================
-- 4. user_instance_assignments 表 (多对多关系)
-- ==============================================
CREATE TABLE IF NOT EXISTS user_instance_assignments (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    assigned_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, instance_id)
);

-- ==============================================
-- 5. instance_ai_agents 表
-- ==============================================
CREATE TABLE IF NOT EXISTS instance_ai_agents (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    agent_type TEXT NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    api_key_encrypted TEXT,
    config_json TEXT,
    rate_limit_rpm INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(instance_id, agent_type)
);

-- ==============================================
-- 6. instance_usage_stats 表
-- ==============================================
CREATE TABLE IF NOT EXISTS instance_usage_stats (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL REFERENCES vibe_instances(id) ON DELETE CASCADE,
    agent_type TEXT NOT NULL,
    date TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    token_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(instance_id, agent_type, date)
);

-- ==============================================
-- 索引
-- ==============================================
CREATE INDEX IF NOT EXISTS idx_users_current_instance ON users(current_instance_id);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON user_sessions(token_hash);

CREATE INDEX IF NOT EXISTS idx_assignments_user_id ON user_instance_assignments(user_id);
CREATE INDEX IF NOT EXISTS idx_assignments_instance_id ON user_instance_assignments(instance_id);

CREATE INDEX IF NOT EXISTS idx_vibe_instances_status ON vibe_instances(status);
CREATE INDEX IF NOT EXISTS idx_vibe_instances_port ON vibe_instances(port);

CREATE INDEX IF NOT EXISTS idx_instance_ai_agents_instance ON instance_ai_agents(instance_id);
CREATE INDEX IF NOT EXISTS idx_instance_ai_agents_type ON instance_ai_agents(agent_type);

CREATE INDEX IF NOT EXISTS idx_usage_stats_instance ON instance_usage_stats(instance_id);
CREATE INDEX IF NOT EXISTS idx_usage_stats_date ON instance_usage_stats(date);
