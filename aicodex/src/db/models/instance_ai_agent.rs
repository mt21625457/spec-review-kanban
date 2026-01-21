//! 实例 AI 智能体配置数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// AI 智能体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    ClaudeCode,
    CodexCli,
    GeminiCli,
    OpenCode,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::ClaudeCode => write!(f, "claude-code"),
            AgentType::CodexCli => write!(f, "codex-cli"),
            AgentType::GeminiCli => write!(f, "gemini-cli"),
            AgentType::OpenCode => write!(f, "opencode"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude-code" => Ok(AgentType::ClaudeCode),
            "codex-cli" => Ok(AgentType::CodexCli),
            "gemini-cli" => Ok(AgentType::GeminiCli),
            "opencode" => Ok(AgentType::OpenCode),
            _ => Err(format!("未知的智能体类型: {}", s)),
        }
    }
}

/// 实例 AI 智能体配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InstanceAiAgent {
    pub id: String,
    pub instance_id: String,
    pub agent_type: String,
    pub is_enabled: bool,
    pub api_key_encrypted: Option<String>,
    pub config_json: Option<String>,
    pub rate_limit_rpm: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InstanceAiAgent {
    /// 获取智能体类型枚举
    pub fn agent_type_enum(&self) -> Result<AgentType, String> {
        self.agent_type.parse()
    }

    /// 根据 ID 获取配置
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM instance_ai_agents WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 根据实例和智能体类型获取配置
    pub async fn get_by_instance_and_type(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_ai_agents WHERE instance_id = ? AND agent_type = ?",
        )
        .bind(instance_id)
        .bind(agent_type)
        .fetch_optional(pool)
        .await
    }

    /// 获取实例的所有智能体配置
    pub async fn list_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_ai_agents WHERE instance_id = ? ORDER BY agent_type",
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }

    /// 获取实例的已启用智能体配置
    pub async fn list_enabled_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_ai_agents WHERE instance_id = ? AND is_enabled = 1 ORDER BY agent_type",
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }

    /// 创建或更新智能体配置
    pub async fn upsert(
        pool: &Pool<Sqlite>,
        id: &str,
        instance_id: &str,
        agent_type: &str,
        is_enabled: bool,
        api_key_encrypted: Option<&str>,
        config_json: Option<&str>,
        rate_limit_rpm: Option<i32>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO instance_ai_agents (id, instance_id, agent_type, is_enabled, api_key_encrypted, config_json, rate_limit_rpm, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(instance_id, agent_type) DO UPDATE SET
                is_enabled = excluded.is_enabled,
                api_key_encrypted = COALESCE(excluded.api_key_encrypted, instance_ai_agents.api_key_encrypted),
                config_json = COALESCE(excluded.config_json, instance_ai_agents.config_json),
                rate_limit_rpm = COALESCE(excluded.rate_limit_rpm, instance_ai_agents.rate_limit_rpm),
                updated_at = excluded.updated_at
            "#,
        )
        .bind(id)
        .bind(instance_id)
        .bind(agent_type)
        .bind(is_enabled)
        .bind(api_key_encrypted)
        .bind(config_json)
        .bind(rate_limit_rpm)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_instance_and_type(pool, instance_id, agent_type)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新启用状态
    pub async fn set_enabled(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        agent_type: &str,
        is_enabled: bool,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE instance_ai_agents SET is_enabled = ?, updated_at = ? WHERE instance_id = ? AND agent_type = ?",
        )
        .bind(is_enabled)
        .bind(now)
        .bind(instance_id)
        .bind(agent_type)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 更新 API Key
    pub async fn update_api_key(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        agent_type: &str,
        api_key_encrypted: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE instance_ai_agents SET api_key_encrypted = ?, updated_at = ? WHERE instance_id = ? AND agent_type = ?",
        )
        .bind(api_key_encrypted)
        .bind(now)
        .bind(instance_id)
        .bind(agent_type)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 删除智能体配置
    pub async fn delete(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        agent_type: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM instance_ai_agents WHERE instance_id = ? AND agent_type = ?",
        )
        .bind(instance_id)
        .bind(agent_type)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除实例的所有智能体配置
    pub async fn delete_all_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM instance_ai_agents WHERE instance_id = ?")
            .bind(instance_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}

/// 智能体配置信息（不含加密 key）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigInfo {
    pub agent_type: String,
    pub is_enabled: bool,
    pub has_api_key: bool,
    pub config: Option<serde_json::Value>,
    pub rate_limit_rpm: Option<i32>,
}

impl From<InstanceAiAgent> for AgentConfigInfo {
    fn from(agent: InstanceAiAgent) -> Self {
        Self {
            agent_type: agent.agent_type,
            is_enabled: agent.is_enabled,
            has_api_key: agent.api_key_encrypted.is_some(),
            config: agent.config_json.and_then(|s| serde_json::from_str(&s).ok()),
            rate_limit_rpm: agent.rate_limit_rpm,
        }
    }
}
