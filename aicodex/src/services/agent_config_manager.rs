//! AI 智能体配置管理服务
//!
//! 管理每个实例的 AI 智能体配置，包括 API Key 加密存储和配置文件生成

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::instance_ai_agent::{AgentConfigInfo, AgentType, InstanceAiAgent};
use crate::db::models::vibe_instance::VibeInstance;
use crate::db::Database;
use crate::error::AppError;
use crate::services::encryption::EncryptionService;

/// AI 智能体配置请求
#[derive(Debug, Deserialize)]
pub struct AgentConfigRequest {
    pub is_enabled: bool,
    pub api_key: Option<String>,
    pub config: Option<serde_json::Value>,
    pub rate_limit_rpm: Option<i32>,
}

/// Claude Code 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub model: Option<String>,
    pub max_tokens: Option<i32>,
    pub custom_instructions: Option<String>,
}

/// Codex CLI 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexCliConfig {
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

/// Gemini CLI 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiCliConfig {
    pub model: Option<String>,
    pub safety_settings: Option<serde_json::Value>,
}

/// OpenCode 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// AI 智能体配置管理服务
pub struct AgentConfigManager {
    db: Arc<Database>,
    encryption: Arc<EncryptionService>,
}

impl AgentConfigManager {
    /// 创建智能体配置管理服务
    pub fn new(db: Arc<Database>, encryption: Arc<EncryptionService>) -> Self {
        Self { db, encryption }
    }

    // ==================== 配置管理 ====================

    /// 获取实例的所有智能体配置
    pub async fn list_configs(&self, instance_id: &str) -> Result<Vec<AgentConfigInfo>, AppError> {
        let agents = InstanceAiAgent::list_by_instance(&self.db.pool, instance_id).await?;
        Ok(agents.into_iter().map(AgentConfigInfo::from).collect())
    }

    /// 获取特定智能体配置
    pub async fn get_config(
        &self,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<Option<AgentConfigInfo>, AppError> {
        let agent =
            InstanceAiAgent::get_by_instance_and_type(&self.db.pool, instance_id, agent_type)
                .await?;
        Ok(agent.map(AgentConfigInfo::from))
    }

    /// 设置智能体配置
    pub async fn set_config(
        &self,
        instance_id: &str,
        agent_type: &str,
        request: AgentConfigRequest,
    ) -> Result<AgentConfigInfo, AppError> {
        // 验证实例存在
        VibeInstance::get_by_id(&self.db.pool, instance_id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 验证智能体类型
        let _: AgentType = agent_type
            .parse()
            .map_err(|e: String| AppError::BadRequest(e))?;

        // 加密 API Key
        let api_key_encrypted = if let Some(ref key) = request.api_key {
            Some(self.encryption.encrypt(key)?)
        } else {
            None
        };

        // 序列化配置
        let config_json = request
            .config
            .map(|c| serde_json::to_string(&c))
            .transpose()
            .map_err(|e| AppError::Internal(format!("配置序列化失败: {}", e)))?;

        // 保存配置
        let id = Uuid::new_v4().to_string();
        let agent = InstanceAiAgent::upsert(
            &self.db.pool,
            &id,
            instance_id,
            agent_type,
            request.is_enabled,
            api_key_encrypted.as_deref(),
            config_json.as_deref(),
            request.rate_limit_rpm,
        )
        .await?;

        // 生成配置文件
        self.generate_config_file(instance_id, agent_type).await?;

        tracing::info!(
            instance_id = %instance_id,
            agent_type = %agent_type,
            is_enabled = %request.is_enabled,
            "智能体配置已更新"
        );

        Ok(agent.into())
    }

    /// 启用/禁用智能体
    pub async fn set_enabled(
        &self,
        instance_id: &str,
        agent_type: &str,
        is_enabled: bool,
    ) -> Result<(), AppError> {
        InstanceAiAgent::set_enabled(&self.db.pool, instance_id, agent_type, is_enabled).await?;

        tracing::info!(
            instance_id = %instance_id,
            agent_type = %agent_type,
            is_enabled = %is_enabled,
            "智能体启用状态已更新"
        );

        Ok(())
    }

    /// 更新 API Key
    pub async fn update_api_key(
        &self,
        instance_id: &str,
        agent_type: &str,
        api_key: &str,
    ) -> Result<(), AppError> {
        let api_key_encrypted = self.encryption.encrypt(api_key)?;
        InstanceAiAgent::update_api_key(&self.db.pool, instance_id, agent_type, &api_key_encrypted)
            .await?;

        // 重新生成配置文件
        self.generate_config_file(instance_id, agent_type).await?;

        tracing::info!(
            instance_id = %instance_id,
            agent_type = %agent_type,
            "智能体 API Key 已更新"
        );

        Ok(())
    }

    /// 删除智能体配置
    pub async fn delete_config(
        &self,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<(), AppError> {
        InstanceAiAgent::delete(&self.db.pool, instance_id, agent_type).await?;

        tracing::info!(
            instance_id = %instance_id,
            agent_type = %agent_type,
            "智能体配置已删除"
        );

        Ok(())
    }

    // ==================== 测试连接 ====================

    /// 测试智能体连接
    pub async fn test_connection(
        &self,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<bool, AppError> {
        let agent =
            InstanceAiAgent::get_by_instance_and_type(&self.db.pool, instance_id, agent_type)
                .await?
                .ok_or_else(|| AppError::NotFound("智能体配置不存在".to_string()))?;

        let api_key = if let Some(ref encrypted) = agent.api_key_encrypted {
            self.encryption.decrypt(encrypted)?
        } else {
            return Err(AppError::BadRequest("未配置 API Key".to_string()));
        };

        match agent_type {
            "claude-code" => self.test_anthropic_connection(&api_key).await,
            "codex-cli" => self.test_openai_connection(&api_key).await,
            "gemini-cli" => self.test_google_connection(&api_key).await,
            "opencode" => Ok(true), // OpenCode 不需要直接测试
            _ => Err(AppError::BadRequest("未知的智能体类型".to_string())),
        }
    }

    /// 测试 Anthropic API 连接
    async fn test_anthropic_connection(&self, api_key: &str) -> Result<bool, AppError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("连接测试失败: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// 测试 OpenAI API 连接
    async fn test_openai_connection(&self, api_key: &str) -> Result<bool, AppError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("连接测试失败: {}", e)))?;

        Ok(response.status().is_success())
    }

    /// 测试 Google API 连接
    async fn test_google_connection(&self, api_key: &str) -> Result<bool, AppError> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models?key={}",
            api_key
        );
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("连接测试失败: {}", e)))?;

        Ok(response.status().is_success())
    }

    // ==================== 配置文件生成 ====================

    /// 生成配置文件
    async fn generate_config_file(
        &self,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<(), AppError> {
        let instance = VibeInstance::get_by_id(&self.db.pool, instance_id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        let agent =
            InstanceAiAgent::get_by_instance_and_type(&self.db.pool, instance_id, agent_type)
                .await?;

        let Some(agent) = agent else {
            return Ok(());
        };

        let data_dir = PathBuf::from(&instance.data_dir);

        match agent_type {
            "claude-code" => {
                self.generate_claude_code_config(&data_dir, &agent).await?
            }
            "codex-cli" => self.generate_codex_cli_config(&data_dir, &agent).await?,
            "gemini-cli" => {
                self.generate_gemini_cli_config(&data_dir, &agent).await?
            }
            "opencode" => self.generate_opencode_config(&data_dir, &agent).await?,
            _ => {}
        }

        Ok(())
    }

    /// 生成 Claude Code 配置
    async fn generate_claude_code_config(
        &self,
        data_dir: &PathBuf,
        agent: &InstanceAiAgent,
    ) -> Result<(), AppError> {
        let config_dir = data_dir.join("ai-agents/claude-code");
        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;

        let config: ClaudeCodeConfig = agent
            .config_json
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(ClaudeCodeConfig {
                model: Some("claude-sonnet-4-20250514".to_string()),
                max_tokens: Some(8192),
                custom_instructions: None,
            });

        let settings = serde_json::json!({
            "model": config.model,
            "maxTokens": config.max_tokens,
            "customInstructions": config.custom_instructions
        });

        let settings_path = config_dir.join("settings.json");
        tokio::fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap())
            .await
            .map_err(|e| AppError::Internal(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 生成 Codex CLI 配置
    async fn generate_codex_cli_config(
        &self,
        data_dir: &PathBuf,
        agent: &InstanceAiAgent,
    ) -> Result<(), AppError> {
        let config_dir = data_dir.join("ai-agents/codex-cli");
        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;

        let config: CodexCliConfig = agent
            .config_json
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(CodexCliConfig {
                model: Some("gpt-4".to_string()),
                temperature: Some(0.7),
            });

        let yaml_content = format!(
            "model: {}\ntemperature: {}\n",
            config.model.unwrap_or_else(|| "gpt-4".to_string()),
            config.temperature.unwrap_or(0.7)
        );

        let config_path = config_dir.join("config.yaml");
        tokio::fs::write(&config_path, yaml_content)
            .await
            .map_err(|e| AppError::Internal(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 生成 Gemini CLI 配置
    async fn generate_gemini_cli_config(
        &self,
        data_dir: &PathBuf,
        agent: &InstanceAiAgent,
    ) -> Result<(), AppError> {
        let config_dir = data_dir.join("ai-agents/gemini-cli");
        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;

        let config: GeminiCliConfig = agent
            .config_json
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(GeminiCliConfig {
                model: Some("gemini-pro".to_string()),
                safety_settings: None,
            });

        let settings = serde_json::json!({
            "model": config.model,
            "safetySettings": config.safety_settings
        });

        let config_path = config_dir.join("config.json");
        tokio::fs::write(&config_path, serde_json::to_string_pretty(&settings).unwrap())
            .await
            .map_err(|e| AppError::Internal(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 生成 OpenCode 配置
    async fn generate_opencode_config(
        &self,
        data_dir: &PathBuf,
        agent: &InstanceAiAgent,
    ) -> Result<(), AppError> {
        let config_dir = data_dir.join("ai-agents/opencode");
        tokio::fs::create_dir_all(&config_dir)
            .await
            .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;

        let config: OpenCodeConfig = agent
            .config_json
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(OpenCodeConfig {
                provider: Some("openai".to_string()),
                model: Some("gpt-4".to_string()),
            });

        let toml_content = format!(
            "[provider]\nname = \"{}\"\n\n[model]\nname = \"{}\"\n",
            config.provider.unwrap_or_else(|| "openai".to_string()),
            config.model.unwrap_or_else(|| "gpt-4".to_string())
        );

        let config_path = config_dir.join("config.toml");
        tokio::fs::write(&config_path, toml_content)
            .await
            .map_err(|e| AppError::Internal(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    /// 为实例重新生成所有配置文件
    pub async fn regenerate_all_configs(&self, instance_id: &str) -> Result<(), AppError> {
        let agents = InstanceAiAgent::list_enabled_by_instance(&self.db.pool, instance_id).await?;

        for agent in agents {
            self.generate_config_file(instance_id, &agent.agent_type)
                .await?;
        }

        Ok(())
    }
}
