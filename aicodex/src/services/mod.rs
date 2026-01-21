pub mod config_service;
pub mod encryption;
pub mod gitea_client;
pub mod vibe_client;
pub mod webhook_handler;
pub mod review_service;

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

        Self {
            config: config_service,
            encryption,
            gitea,
            vibe,
            webhook,
            review,
        }
    }
}
