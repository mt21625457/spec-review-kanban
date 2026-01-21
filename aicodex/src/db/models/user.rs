//! 用户数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// 用户角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    User,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "user" => Ok(UserRole::User),
            _ => Err(format!("未知的用户角色: {}", s)),
        }
    }
}

/// 用户
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub current_instance_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    /// 获取用户角色枚举
    pub fn role_enum(&self) -> UserRole {
        self.role.parse().unwrap_or(UserRole::User)
    }

    /// 是否为管理员
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// 根据 ID 获取用户
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 根据用户名获取用户
    pub async fn get_by_username(
        pool: &Pool<Sqlite>,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    /// 根据邮箱获取用户
    pub async fn get_by_email(
        pool: &Pool<Sqlite>,
        email: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /// 列出所有用户
    pub async fn list(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
    }

    /// 列出活跃用户
    pub async fn list_active(pool: &Pool<Sqlite>) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM users WHERE is_active = 1 ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    /// 创建用户
    pub async fn create(
        pool: &Pool<Sqlite>,
        id: &str,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        display_name: Option<&str>,
        role: UserRole,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, role, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .bind(role.to_string())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新用户信息
    pub async fn update(
        pool: &Pool<Sqlite>,
        id: &str,
        email: Option<&str>,
        display_name: Option<&str>,
        role: Option<UserRole>,
    ) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE users SET
                email = COALESCE(?, email),
                display_name = COALESCE(?, display_name),
                role = COALESCE(?, role),
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(email)
        .bind(display_name)
        .bind(role.map(|r| r.to_string()))
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 更新密码
    pub async fn update_password(
        pool: &Pool<Sqlite>,
        id: &str,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(password_hash)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 更新当前实例
    pub async fn update_current_instance(
        pool: &Pool<Sqlite>,
        id: &str,
        instance_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET current_instance_id = ?, updated_at = ? WHERE id = ?")
            .bind(instance_id)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 更新最后登录时间
    pub async fn update_last_login(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 激活/停用用户
    pub async fn set_active(
        pool: &Pool<Sqlite>,
        id: &str,
        is_active: bool,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query("UPDATE users SET is_active = ?, updated_at = ? WHERE id = ?")
            .bind(is_active)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// 删除用户
    pub async fn delete(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// 检查用户名是否已存在
    pub async fn exists_by_username(
        pool: &Pool<Sqlite>,
        username: &str,
    ) -> Result<bool, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = ?")
                .bind(username)
                .fetch_one(pool)
                .await?;
        Ok(result.0 > 0)
    }

    /// 检查邮箱是否已存在
    pub async fn exists_by_email(pool: &Pool<Sqlite>, email: &str) -> Result<bool, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE email = ?")
                .bind(email)
                .fetch_one(pool)
                .await?;
        Ok(result.0 > 0)
    }

    /// 统计用户数量
    pub async fn count(pool: &Pool<Sqlite>) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }
}

/// 用户公开信息（不含密码哈希）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub role: String,
    pub current_instance_id: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
            current_instance_id: user.current_instance_id,
            is_active: user.is_active,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}
