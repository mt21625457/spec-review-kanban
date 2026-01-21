use std::env;

/// 应用配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 数据库 URL
    pub database_url: String,
    /// 服务器监听地址
    pub host: String,
    /// 服务器监听端口
    pub port: u16,
    /// Vibe Kanban 服务地址
    pub vibe_kanban_url: String,
    /// Gitea Webhook Secret
    pub gitea_webhook_secret: Option<String>,
    /// 配置加密密钥
    pub config_encryption_key: Option<String>,

    // ==================== 多实例管理配置 ====================
    /// JWT 密钥
    pub jwt_secret: Option<String>,
    /// Vibe Kanban 可执行文件路径
    pub vibe_kanban_bin: Option<String>,
    /// 实例数据根目录
    pub vibe_instances_data_root: Option<String>,
    /// 实例端口范围起始
    pub vibe_instances_port_base: Option<i32>,
    /// 实例端口范围结束
    pub vibe_instances_port_max: Option<i32>,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./aicodex.db?mode=rwc".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8765".to_string())
                .parse()?,
            vibe_kanban_url: env::var("VIBE_KANBAN_URL")
                .unwrap_or_else(|_| "http://localhost:8766".to_string()),
            gitea_webhook_secret: env::var("GITEA_WEBHOOK_SECRET").ok(),
            config_encryption_key: env::var("CONFIG_ENCRYPTION_KEY").ok(),

            // 多实例管理配置
            jwt_secret: env::var("JWT_SECRET").ok(),
            vibe_kanban_bin: env::var("VIBE_KANBAN_BIN").ok(),
            vibe_instances_data_root: env::var("VIBE_INSTANCES_DATA_ROOT").ok(),
            vibe_instances_port_base: env::var("VIBE_INSTANCES_PORT_BASE")
                .ok()
                .and_then(|s| s.parse().ok()),
            vibe_instances_port_max: env::var("VIBE_INSTANCES_PORT_MAX")
                .ok()
                .and_then(|s| s.parse().ok()),
        })
    }
}
