use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};
use uuid::Uuid;

/// 审核状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for ReviewStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewStatus::Pending => write!(f, "pending"),
            ReviewStatus::Running => write!(f, "running"),
            ReviewStatus::Completed => write!(f, "completed"),
            ReviewStatus::Failed => write!(f, "failed"),
            ReviewStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for ReviewStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(ReviewStatus::Pending),
            "running" => Ok(ReviewStatus::Running),
            "completed" => Ok(ReviewStatus::Completed),
            "failed" => Ok(ReviewStatus::Failed),
            "cancelled" => Ok(ReviewStatus::Cancelled),
            _ => Err(format!("未知的审核状态: {}", s)),
        }
    }
}

/// 审核运行记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewRun {
    pub id: String,
    pub repo_mapping_id: String,
    pub gitea_pr_number: i64,
    pub gitea_pr_url: Option<String>,
    pub vibe_task_id: Option<String>,
    pub vibe_workspace_id: Option<String>,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建审核运行请求
#[derive(Debug, Clone)]
pub struct CreateReviewRun {
    pub repo_mapping_id: String,
    pub gitea_pr_number: i64,
    pub gitea_pr_url: Option<String>,
}

impl ReviewRun {
    /// 通过 ID 获取
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM review_runs WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 通过仓库映射和 PR 号获取最新的审核
    pub async fn get_latest_for_pr(
        pool: &Pool<Sqlite>,
        repo_mapping_id: &str,
        pr_number: i64,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM review_runs
            WHERE repo_mapping_id = ? AND gitea_pr_number = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(repo_mapping_id)
        .bind(pr_number)
        .fetch_optional(pool)
        .await
    }

    /// 创建审核运行
    pub async fn create(pool: &Pool<Sqlite>, input: CreateReviewRun) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO review_runs (id, repo_mapping_id, gitea_pr_number, gitea_pr_url, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.repo_mapping_id)
        .bind(input.gitea_pr_number)
        .bind(&input.gitea_pr_url)
        .bind(ReviewStatus::Pending.to_string())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新状态为运行中
    pub async fn start(&self, pool: &Pool<Sqlite>, vibe_task_id: &str, vibe_workspace_id: Option<&str>) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE review_runs SET
                status = ?,
                vibe_task_id = ?,
                vibe_workspace_id = ?,
                started_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(ReviewStatus::Running.to_string())
        .bind(vibe_task_id)
        .bind(vibe_workspace_id)
        .bind(now)
        .bind(now)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &self.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新状态为完成
    pub async fn complete(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE review_runs SET
                status = ?,
                completed_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(ReviewStatus::Completed.to_string())
        .bind(now)
        .bind(now)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &self.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新状态为失败
    pub async fn fail(&self, pool: &Pool<Sqlite>, error_message: &str) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE review_runs SET
                status = ?,
                error_message = ?,
                completed_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(ReviewStatus::Failed.to_string())
        .bind(error_message)
        .bind(now)
        .bind(now)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &self.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新状态为取消
    pub async fn cancel(&self, pool: &Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE review_runs SET
                status = ?,
                completed_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(ReviewStatus::Cancelled.to_string())
        .bind(now)
        .bind(now)
        .bind(&self.id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &self.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    /// 列出审核运行
    pub async fn list(pool: &Pool<Sqlite>, limit: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM review_runs ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }

    /// 按状态列出
    pub async fn list_by_status(pool: &Pool<Sqlite>, status: ReviewStatus, limit: i64) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM review_runs WHERE status = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(status.to_string())
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}
