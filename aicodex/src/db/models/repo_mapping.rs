use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// Agent 类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    Codex,
    Gemini,
    OpenCode,
    Copilot,
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Codex
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::ClaudeCode => write!(f, "claude_code"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Gemini => write!(f, "gemini"),
            AgentType::OpenCode => write!(f, "open_code"),
            AgentType::Copilot => write!(f, "copilot"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude_code" => Ok(AgentType::ClaudeCode),
            "codex" => Ok(AgentType::Codex),
            "gemini" => Ok(AgentType::Gemini),
            "open_code" => Ok(AgentType::OpenCode),
            "copilot" => Ok(AgentType::Copilot),
            _ => Err(format!("未知的 Agent 类型: {}", s)),
        }
    }
}

/// 仓库映射
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RepoMapping {
    pub id: String,
    pub gitea_repo: String,
    pub local_path: String,
    pub vibe_project_id: Option<String>,
    pub agent_type: String,
    pub executor_profile_id: Option<String>,
    pub custom_prompt: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建仓库映射请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateRepoMapping {
    pub gitea_repo: String,
    pub local_path: String,
    pub vibe_project_id: Option<String>,
    pub agent_type: Option<AgentType>,
    pub executor_profile_id: Option<String>,
    pub custom_prompt: Option<String>,
    pub is_enabled: Option<bool>,
}

/// 更新仓库映射请求
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRepoMapping {
    pub local_path: Option<String>,
    pub vibe_project_id: Option<String>,
    pub agent_type: Option<AgentType>,
    pub executor_profile_id: Option<String>,
    pub custom_prompt: Option<String>,
    pub is_enabled: Option<bool>,
}

impl RepoMapping {
    /// 通过 ID 获取
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM repo_mappings WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 通过 Gitea 仓库获取
    pub async fn get_by_gitea_repo(pool: &Pool<Sqlite>, gitea_repo: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM repo_mappings WHERE gitea_repo = ?")
            .bind(gitea_repo)
            .fetch_optional(pool)
            .await
    }

    /// 创建仓库映射
    pub async fn create(pool: &Pool<Sqlite>, input: CreateRepoMapping) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let agent_type = input.agent_type.unwrap_or_default().to_string();
        let is_enabled = input.is_enabled.unwrap_or(true);

        sqlx::query(
            r#"
            INSERT INTO repo_mappings (id, gitea_repo, local_path, vibe_project_id, agent_type, executor_profile_id, custom_prompt, is_enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.gitea_repo)
        .bind(&input.local_path)
        .bind(&input.vibe_project_id)
        .bind(&agent_type)
        .bind(&input.executor_profile_id)
        .bind(&input.custom_prompt)
        .bind(is_enabled)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新仓库映射
    pub async fn update(pool: &Pool<Sqlite>, id: &str, input: UpdateRepoMapping) -> Result<Self, sqlx::Error> {
        let current = Self::get_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)?;
        let now = Utc::now();

        let local_path = input.local_path.unwrap_or(current.local_path);
        let vibe_project_id = input.vibe_project_id.or(current.vibe_project_id);
        let agent_type = input.agent_type.map(|a| a.to_string()).unwrap_or(current.agent_type);
        let executor_profile_id = input.executor_profile_id.or(current.executor_profile_id);
        let custom_prompt = input.custom_prompt.or(current.custom_prompt);
        let is_enabled = input.is_enabled.unwrap_or(current.is_enabled);

        sqlx::query(
            r#"
            UPDATE repo_mappings SET
                local_path = ?,
                vibe_project_id = ?,
                agent_type = ?,
                executor_profile_id = ?,
                custom_prompt = ?,
                is_enabled = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&local_path)
        .bind(&vibe_project_id)
        .bind(&agent_type)
        .bind(&executor_profile_id)
        .bind(&custom_prompt)
        .bind(is_enabled)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 删除仓库映射
    pub async fn delete(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM repo_mappings WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 列出所有仓库映射
    pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM repo_mappings ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    /// 列出启用的仓库映射
    pub async fn list_enabled(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM repo_mappings WHERE is_enabled = 1 ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }
}
