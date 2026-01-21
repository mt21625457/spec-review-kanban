//! Vibe-Kanban 实例数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// 实例状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceStatus::Stopped => write!(f, "stopped"),
            InstanceStatus::Starting => write!(f, "starting"),
            InstanceStatus::Running => write!(f, "running"),
            InstanceStatus::Stopping => write!(f, "stopping"),
            InstanceStatus::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for InstanceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stopped" => Ok(InstanceStatus::Stopped),
            "starting" => Ok(InstanceStatus::Starting),
            "running" => Ok(InstanceStatus::Running),
            "stopping" => Ok(InstanceStatus::Stopping),
            "error" => Ok(InstanceStatus::Error),
            _ => Err(format!("未知的实例状态: {}", s)),
        }
    }
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Unknown => write!(f, "unknown"),
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

impl std::str::FromStr for HealthStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unknown" => Ok(HealthStatus::Unknown),
            "healthy" => Ok(HealthStatus::Healthy),
            "unhealthy" => Ok(HealthStatus::Unhealthy),
            _ => Err(format!("未知的健康状态: {}", s)),
        }
    }
}

/// Vibe-Kanban 实例
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VibeInstance {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub port: i32,
    pub data_dir: String,
    pub status: String,
    pub auto_start: bool,
    pub max_users: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: Option<String>,
    /// 最后一次错误信息
    pub last_error: Option<String>,
    /// 最后一次错误时间
    pub last_error_at: Option<DateTime<Utc>>,
}

impl VibeInstance {
    /// 获取状态枚举
    pub fn status_enum(&self) -> InstanceStatus {
        self.status.parse().unwrap_or(InstanceStatus::Stopped)
    }

    /// 获取健康状态枚举
    pub fn health_status_enum(&self) -> HealthStatus {
        self.health_status
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(HealthStatus::Unknown)
    }

    /// 根据 ID 获取实例
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM vibe_instances WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 根据端口获取实例
    pub async fn get_by_port(pool: &Pool<Sqlite>, port: i32) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM vibe_instances WHERE port = ?")
            .bind(port)
            .fetch_optional(pool)
            .await
    }

    /// 列出所有实例
    pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM vibe_instances ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    /// 列出运行中的实例
    pub async fn list_running(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM vibe_instances WHERE status = 'running' ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// 列出自动启动的实例
    pub async fn list_auto_start(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM vibe_instances WHERE auto_start = 1 ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// 创建实例
    pub async fn create(
        pool: &Pool<Sqlite>,
        id: &str,
        name: &str,
        description: Option<&str>,
        port: i32,
        data_dir: &str,
        auto_start: bool,
        max_users: Option<i32>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO vibe_instances (id, name, description, port, data_dir, status, auto_start, max_users, created_at, updated_at, health_status)
            VALUES (?, ?, ?, ?, ?, 'stopped', ?, ?, ?, ?, 'unknown')
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(port)
        .bind(data_dir)
        .bind(auto_start)
        .bind(max_users)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新实例信息
    pub async fn update(
        pool: &Pool<Sqlite>,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        auto_start: Option<bool>,
        max_users: Option<i32>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE vibe_instances SET
                name = COALESCE(?, name),
                description = COALESCE(?, description),
                auto_start = COALESCE(?, auto_start),
                max_users = COALESCE(?, max_users),
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(auto_start)
        .bind(max_users)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新实例状态
    pub async fn update_status(
        pool: &Pool<Sqlite>,
        id: &str,
        status: InstanceStatus,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE vibe_instances SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 更新健康状态
    pub async fn update_health_status(
        pool: &Pool<Sqlite>,
        id: &str,
        health_status: HealthStatus,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE vibe_instances SET health_status = ?, last_health_check = ?, updated_at = ? WHERE id = ?",
        )
        .bind(health_status.to_string())
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 更新错误信息
    pub async fn update_error(
        pool: &Pool<Sqlite>,
        id: &str,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        if let Some(err) = error {
            sqlx::query(
                "UPDATE vibe_instances SET last_error = ?, last_error_at = ?, updated_at = ? WHERE id = ?",
            )
            .bind(err)
            .bind(now)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            // 清除错误
            sqlx::query(
                "UPDATE vibe_instances SET last_error = NULL, last_error_at = NULL, updated_at = ? WHERE id = ?",
            )
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// 更新状态并设置错误
    pub async fn update_status_with_error(
        pool: &Pool<Sqlite>,
        id: &str,
        status: InstanceStatus,
        error: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        if let Some(err) = error {
            sqlx::query(
                "UPDATE vibe_instances SET status = ?, last_error = ?, last_error_at = ?, updated_at = ? WHERE id = ?",
            )
            .bind(status.to_string())
            .bind(err)
            .bind(now)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE vibe_instances SET status = ?, last_error = NULL, last_error_at = NULL, updated_at = ? WHERE id = ?",
            )
            .bind(status.to_string())
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// 删除实例
    pub async fn delete(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM vibe_instances WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 检查端口是否已使用
    pub async fn is_port_used(pool: &Pool<Sqlite>, port: i32) -> Result<bool, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM vibe_instances WHERE port = ?")
                .bind(port)
                .fetch_one(pool)
                .await?;
        Ok(result.0 > 0)
    }

    /// 获取下一个可用端口
    pub async fn next_available_port(
        pool: &Pool<Sqlite>,
        base_port: i32,
        max_port: i32,
    ) -> Result<Option<i32>, sqlx::Error> {
        // 获取所有已使用的端口
        let used_ports: Vec<(i32,)> = sqlx::query_as(
            "SELECT port FROM vibe_instances WHERE port >= ? AND port <= ? ORDER BY port",
        )
        .bind(base_port)
        .bind(max_port)
        .fetch_all(pool)
        .await?;

        let used_set: std::collections::HashSet<i32> =
            used_ports.into_iter().map(|(p,)| p).collect();

        for port in base_port..=max_port {
            if !used_set.contains(&port) {
                return Ok(Some(port));
            }
        }

        Ok(None)
    }

    /// 统计实例数量
    pub async fn count(pool: &Pool<Sqlite>) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vibe_instances")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }

    /// 统计实例用户数量
    pub async fn count_users(pool: &Pool<Sqlite>, id: &str) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_instance_assignments WHERE instance_id = ?")
                .bind(id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }
}

/// 实例公开信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub port: i32,
    pub status: String,
    pub health_status: String,
    pub auto_start: bool,
    pub max_users: Option<i32>,
    pub user_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub last_health_check: Option<DateTime<Utc>>,
    /// 最后一次错误信息
    pub last_error: Option<String>,
    /// 最后一次错误时间
    pub last_error_at: Option<DateTime<Utc>>,
}

impl From<VibeInstance> for InstanceInfo {
    fn from(instance: VibeInstance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            description: instance.description,
            port: instance.port,
            status: instance.status,
            health_status: instance.health_status.unwrap_or_else(|| "unknown".to_string()),
            auto_start: instance.auto_start,
            max_users: instance.max_users,
            user_count: None,
            created_at: instance.created_at,
            last_health_check: instance.last_health_check,
            last_error: instance.last_error,
            last_error_at: instance.last_error_at,
        }
    }
}
