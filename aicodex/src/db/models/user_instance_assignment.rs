//! 用户-实例分配数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// 用户-实例分配
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserInstanceAssignment {
    pub id: String,
    pub user_id: String,
    pub instance_id: String,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
}

impl UserInstanceAssignment {
    /// 根据 ID 获取分配记录
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM user_instance_assignments WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 获取特定用户和实例的分配记录
    pub async fn get_by_user_and_instance(
        pool: &Pool<Sqlite>,
        user_id: &str,
        instance_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM user_instance_assignments WHERE user_id = ? AND instance_id = ?",
        )
        .bind(user_id)
        .bind(instance_id)
        .fetch_optional(pool)
        .await
    }

    /// 获取用户的所有分配
    pub async fn list_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM user_instance_assignments WHERE user_id = ? ORDER BY assigned_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 获取实例的所有分配
    pub async fn list_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM user_instance_assignments WHERE instance_id = ? ORDER BY assigned_at DESC",
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }

    /// 创建分配
    pub async fn create(
        pool: &Pool<Sqlite>,
        id: &str,
        user_id: &str,
        instance_id: &str,
        assigned_by: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO user_instance_assignments (id, user_id, instance_id, assigned_by, assigned_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(instance_id)
        .bind(assigned_by)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 删除分配
    pub async fn delete(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_instance_assignments WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除特定用户和实例的分配
    pub async fn delete_by_user_and_instance(
        pool: &Pool<Sqlite>,
        user_id: &str,
        instance_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM user_instance_assignments WHERE user_id = ? AND instance_id = ?",
        )
        .bind(user_id)
        .bind(instance_id)
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 删除用户的所有分配
    pub async fn delete_all_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_instance_assignments WHERE user_id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 删除实例的所有分配
    pub async fn delete_all_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM user_instance_assignments WHERE instance_id = ?")
            .bind(instance_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 检查用户是否已分配到实例
    pub async fn is_assigned(
        pool: &Pool<Sqlite>,
        user_id: &str,
        instance_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM user_instance_assignments WHERE user_id = ? AND instance_id = ?",
        )
        .bind(user_id)
        .bind(instance_id)
        .fetch_one(pool)
        .await?;
        Ok(result.0 > 0)
    }

    /// 统计用户分配的实例数量
    pub async fn count_by_user(pool: &Pool<Sqlite>, user_id: &str) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_instance_assignments WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }

    /// 统计实例分配的用户数量
    pub async fn count_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_instance_assignments WHERE instance_id = ?")
                .bind(instance_id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }
}

/// 用户实例分配信息（含用户和实例名称）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserInstanceAssignmentInfo {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub user_display_name: Option<String>,
    pub instance_id: String,
    pub instance_name: String,
    pub assigned_at: DateTime<Utc>,
}

impl UserInstanceAssignmentInfo {
    /// 获取用户的所有分配（含实例信息）
    pub async fn list_by_user_with_instance(
        pool: &Pool<Sqlite>,
        user_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT
                a.id,
                a.user_id,
                u.username,
                u.display_name as user_display_name,
                a.instance_id,
                i.name as instance_name,
                a.assigned_at
            FROM user_instance_assignments a
            JOIN users u ON a.user_id = u.id
            JOIN vibe_instances i ON a.instance_id = i.id
            WHERE a.user_id = ?
            ORDER BY a.assigned_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// 获取实例的所有分配（含用户信息）
    pub async fn list_by_instance_with_user(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            SELECT
                a.id,
                a.user_id,
                u.username,
                u.display_name as user_display_name,
                a.instance_id,
                i.name as instance_name,
                a.assigned_at
            FROM user_instance_assignments a
            JOIN users u ON a.user_id = u.id
            JOIN vibe_instances i ON a.instance_id = i.id
            WHERE a.instance_id = ?
            ORDER BY a.assigned_at DESC
            "#,
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }
}
