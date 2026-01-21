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
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./aicodex.db?mode=rwc".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            vibe_kanban_url: env::var("VIBE_KANBAN_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            gitea_webhook_secret: env::var("GITEA_WEBHOOK_SECRET").ok(),
            config_encryption_key: env::var("CONFIG_ENCRYPTION_KEY").ok(),
        })
    }
}
