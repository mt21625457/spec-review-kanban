pub mod config_service;
pub mod encryption;
pub mod gitea_client;
pub mod vibe_client;
pub mod webhook_handler;
pub mod review_service;

// 多实例管理服务
pub mod user_manager;
pub mod instance_manager;
pub mod agent_config_manager;

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::Database;

/// 服务集合
pub struct Services {
    pub config: Arc<config_service::ConfigService>,
    pub encryption: Arc<encryption::EncryptionService>,
    pub gitea: Arc<gitea_client::GiteaClient>,
    pub vibe: Arc<vibe_client::VibeClient>,
    pub webhook: Arc<webhook_handler::WebhookHandler>,
    pub review: Arc<review_service::ReviewService>,
    // 多实例管理
    pub user_manager: Arc<user_manager::UserManager>,
    pub instance_manager: Arc<instance_manager::InstanceManager>,
    pub agent_config_manager: Arc<agent_config_manager::AgentConfigManager>,
}

impl Services {
    pub fn new(db: Arc<Database>, config: Arc<AppConfig>) -> Self {
        let encryption = Arc::new(encryption::EncryptionService::new(
            config.config_encryption_key.clone(),
        ));

        let config_service = Arc::new(config_service::ConfigService::new(
            db.clone(),
            encryption.clone(),
        ));

        let gitea = Arc::new(gitea_client::GiteaClient::new(config_service.clone()));

        let vibe = Arc::new(vibe_client::VibeClient::new(config.vibe_kanban_url.clone()));

        let webhook = Arc::new(webhook_handler::WebhookHandler::new(
            db.clone(),
            config.gitea_webhook_secret.clone(),
        ));

        let review = Arc::new(review_service::ReviewService::new(
            db.clone(),
            config_service.clone(),
            gitea.clone(),
            vibe.clone(),
        ));

        // 多实例管理服务
        let jwt_secret = config.jwt_secret.clone().unwrap_or_else(|| {
            tracing::warn!("未配置 JWT_SECRET，使用默认值（仅用于开发）");
            "dev-jwt-secret-please-change-in-production".to_string()
        });

        let user_manager = Arc::new(user_manager::UserManager::with_defaults(
            db.clone(),
            jwt_secret,
        ));

        let instance_config = instance_manager::InstanceConfig {
            vibe_kanban_bin: config.vibe_kanban_bin.clone().map(PathBuf::from).unwrap_or_else(|| {
                PathBuf::from("/Users/yangjianbo/code/mt-ai/spec-review-kanban/target/release/server")
            }),
            data_root: config.vibe_instances_data_root.clone().map(PathBuf::from).unwrap_or_else(|| {
                PathBuf::from("/data/vibe-instances")
            }),
            port_base: config.vibe_instances_port_base.unwrap_or(18100),
            port_max: config.vibe_instances_port_max.unwrap_or(18199),
            ..Default::default()
        };

        let instance_manager = Arc::new(instance_manager::InstanceManager::new(
            db.clone(),
            instance_config,
        ));

        let agent_config_manager = Arc::new(agent_config_manager::AgentConfigManager::new(
            db.clone(),
            encryption.clone(),
        ));

        Self {
            config: config_service,
            encryption,
            gitea,
            vibe,
            webhook,
            review,
            user_manager,
            instance_manager,
            agent_config_manager,
        }
    }
}
