use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::db::models::system_config::{keys, ConfigValueType, SystemConfig};
use crate::db::Database;
use crate::services::encryption::EncryptionService;

/// Gitea 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GiteaSettings {
    pub api_url: Option<String>,
    pub token: Option<String>,
    pub webhook_secret: Option<String>,
}

/// 审核配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSettings {
    pub default_agent: String,
    pub auto_start: bool,
    pub timeout_seconds: i64,
}

impl Default for ReviewSettings {
    fn default() -> Self {
        Self {
            default_agent: "codex".to_string(),
            auto_start: true,
            timeout_seconds: 3600,
        }
    }
}

/// 队列配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSettings {
    pub max_concurrent: i64,
    pub retry_count: i64,
    pub retry_delay_seconds: i64,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            retry_count: 3,
            retry_delay_seconds: 60,
        }
    }
}

/// 所有配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllSettings {
    pub gitea: GiteaSettings,
    pub review: ReviewSettings,
    pub queue: QueueSettings,
}

/// 配置服务
pub struct ConfigService {
    db: Arc<Database>,
    encryption: Arc<EncryptionService>,
}

impl ConfigService {
    pub fn new(db: Arc<Database>, encryption: Arc<EncryptionService>) -> Self {
        Self { db, encryption }
    }

    /// 获取所有配置
    pub async fn get_all_settings(&self) -> anyhow::Result<AllSettings> {
        Ok(AllSettings {
            gitea: self.get_gitea_settings().await?,
            review: self.get_review_settings().await?,
            queue: self.get_queue_settings().await?,
        })
    }

    /// 获取 Gitea 配置
    pub async fn get_gitea_settings(&self) -> anyhow::Result<GiteaSettings> {
        let configs = SystemConfig::list_by_prefix(&self.db.pool, "gitea.").await?;

        let mut settings = GiteaSettings::default();

        for config in configs {
            let value = if config.is_encrypted {
                self.encryption.decrypt(&config.config_value).ok()
            } else {
                Some(config.config_value)
            };

            match config.config_key.as_str() {
                keys::GITEA_API_URL => settings.api_url = value,
                keys::GITEA_TOKEN => settings.token = value,
                keys::GITEA_WEBHOOK_SECRET => settings.webhook_secret = value,
                _ => {}
            }
        }

        Ok(settings)
    }

    /// 更新 Gitea 配置
    pub async fn update_gitea_settings(&self, settings: &GiteaSettings) -> anyhow::Result<()> {
        if let Some(ref url) = settings.api_url {
            SystemConfig::set(
                &self.db.pool,
                keys::GITEA_API_URL,
                url,
                false,
                ConfigValueType::String,
                Some("Gitea API URL"),
            )
            .await?;
        }

        if let Some(ref token) = settings.token {
            let encrypted = self.encryption.encrypt(token)?;
            SystemConfig::set(
                &self.db.pool,
                keys::GITEA_TOKEN,
                &encrypted,
                true,
                ConfigValueType::String,
                Some("Gitea API Token"),
            )
            .await?;
        }

        if let Some(ref secret) = settings.webhook_secret {
            let encrypted = self.encryption.encrypt(secret)?;
            SystemConfig::set(
                &self.db.pool,
                keys::GITEA_WEBHOOK_SECRET,
                &encrypted,
                true,
                ConfigValueType::String,
                Some("Gitea Webhook Secret"),
            )
            .await?;
        }

        Ok(())
    }

    /// 获取审核配置
    pub async fn get_review_settings(&self) -> anyhow::Result<ReviewSettings> {
        let configs = SystemConfig::list_by_prefix(&self.db.pool, "review.").await?;

        let mut settings = ReviewSettings::default();

        for config in configs {
            match config.config_key.as_str() {
                keys::REVIEW_DEFAULT_AGENT => settings.default_agent = config.config_value,
                keys::REVIEW_AUTO_START => settings.auto_start = config.config_value == "true",
                keys::REVIEW_TIMEOUT_SECONDS => {
                    settings.timeout_seconds = config.config_value.parse().unwrap_or(3600)
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    /// 更新审核配置
    pub async fn update_review_settings(&self, settings: &ReviewSettings) -> anyhow::Result<()> {
        SystemConfig::set(
            &self.db.pool,
            keys::REVIEW_DEFAULT_AGENT,
            &settings.default_agent,
            false,
            ConfigValueType::String,
            Some("默认 Agent 类型"),
        )
        .await?;

        SystemConfig::set(
            &self.db.pool,
            keys::REVIEW_AUTO_START,
            if settings.auto_start { "true" } else { "false" },
            false,
            ConfigValueType::Boolean,
            Some("自动启动审核"),
        )
        .await?;

        SystemConfig::set(
            &self.db.pool,
            keys::REVIEW_TIMEOUT_SECONDS,
            &settings.timeout_seconds.to_string(),
            false,
            ConfigValueType::Integer,
            Some("审核超时时间（秒）"),
        )
        .await?;

        Ok(())
    }

    /// 获取队列配置
    pub async fn get_queue_settings(&self) -> anyhow::Result<QueueSettings> {
        let configs = SystemConfig::list_by_prefix(&self.db.pool, "queue.").await?;

        let mut settings = QueueSettings::default();

        for config in configs {
            match config.config_key.as_str() {
                keys::QUEUE_MAX_CONCURRENT => {
                    settings.max_concurrent = config.config_value.parse().unwrap_or(3)
                }
                keys::QUEUE_RETRY_COUNT => {
                    settings.retry_count = config.config_value.parse().unwrap_or(3)
                }
                keys::QUEUE_RETRY_DELAY_SECONDS => {
                    settings.retry_delay_seconds = config.config_value.parse().unwrap_or(60)
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    /// 更新队列配置
    pub async fn update_queue_settings(&self, settings: &QueueSettings) -> anyhow::Result<()> {
        SystemConfig::set(
            &self.db.pool,
            keys::QUEUE_MAX_CONCURRENT,
            &settings.max_concurrent.to_string(),
            false,
            ConfigValueType::Integer,
            Some("最大并发任务数"),
        )
        .await?;

        SystemConfig::set(
            &self.db.pool,
            keys::QUEUE_RETRY_COUNT,
            &settings.retry_count.to_string(),
            false,
            ConfigValueType::Integer,
            Some("重试次数"),
        )
        .await?;

        SystemConfig::set(
            &self.db.pool,
            keys::QUEUE_RETRY_DELAY_SECONDS,
            &settings.retry_delay_seconds.to_string(),
            false,
            ConfigValueType::Integer,
            Some("重试延迟（秒）"),
        )
        .await?;

        Ok(())
    }
}
