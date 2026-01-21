//! 用户管理服务
//!
//! 提供用户注册、认证、会话管理和实例分配功能

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::models::user::{User, UserInfo, UserRole};
use crate::db::models::user_instance_assignment::UserInstanceAssignment;
use crate::db::models::user_session::UserSession;
use crate::db::models::vibe_instance::{InstanceInfo, VibeInstance};
use crate::db::Database;
use crate::error::AppError;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// 用户 ID
    pub sub: String,
    /// 用户名
    pub username: String,
    /// 用户角色
    pub role: String,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
}

/// 登录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub instances: Vec<InstanceInfo>,
    pub current_instance_id: Option<String>,
}

/// 会话配置
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// 会话有效期（秒）
    pub session_ttl_secs: i64,
    /// 会话刷新阈值（秒）
    pub refresh_threshold_secs: i64,
    /// 最大并发会话数
    pub max_sessions_per_user: u32,
    /// JWT 密钥
    pub jwt_secret: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_ttl_secs: 86400,        // 24 小时
            refresh_threshold_secs: 3600,   // 1 小时
            max_sessions_per_user: 5,
            jwt_secret: "default-secret-key-please-change".to_string(),
        }
    }
}

/// 用户管理服务
pub struct UserManager {
    db: Arc<Database>,
    config: SessionConfig,
}

impl UserManager {
    /// 创建用户管理服务
    pub fn new(db: Arc<Database>, config: SessionConfig) -> Self {
        Self { db, config }
    }

    /// 创建默认配置的用户管理服务
    pub fn with_defaults(db: Arc<Database>, jwt_secret: String) -> Self {
        Self {
            db,
            config: SessionConfig {
                jwt_secret,
                ..Default::default()
            },
        }
    }

    // ==================== 密码操作 ====================

    /// 哈希密码
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))?;
        Ok(hash.to_string())
    }

    /// 验证密码
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("密码哈希解析失败: {}", e)))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    // ==================== 用户操作 ====================

    /// 注册用户
    pub async fn register(
        &self,
        username: &str,
        email: Option<&str>,
        password: &str,
        display_name: Option<&str>,
        role: Option<UserRole>,
    ) -> Result<User, AppError> {
        // 检查用户名是否已存在
        if User::exists_by_username(&self.db.pool, username).await? {
            return Err(AppError::Conflict("用户名已存在".to_string()));
        }

        // 检查邮箱是否已存在
        if let Some(email) = email {
            if User::exists_by_email(&self.db.pool, email).await? {
                return Err(AppError::Conflict("邮箱已被使用".to_string()));
            }
        }

        // 哈希密码
        let password_hash = Self::hash_password(password)?;

        // 创建用户
        let id = Uuid::new_v4().to_string();
        let user = User::create(
            &self.db.pool,
            &id,
            username,
            email,
            &password_hash,
            display_name,
            role.unwrap_or(UserRole::User),
        )
        .await?;

        tracing::info!(user_id = %id, username = %username, "用户注册成功");

        Ok(user)
    }

    /// 用户登录
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<LoginResponse, AppError> {
        // 获取用户
        let user = User::get_by_username(&self.db.pool, username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 检查用户是否激活
        if !user.is_active {
            return Err(AppError::Forbidden("用户已被停用".to_string()));
        }

        // 验证密码
        if !Self::verify_password(password, &user.password_hash)? {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        // 创建会话
        let (token, _session) = self.create_session(&user, ip_address, user_agent).await?;

        // 更新最后登录时间
        User::update_last_login(&self.db.pool, &user.id).await?;

        // 获取用户分配的实例
        let instances = self.get_user_instances(&user.id).await?;

        // 确定当前实例
        let current_instance_id = if user.current_instance_id.is_some() {
            user.current_instance_id.clone()
        } else if !instances.is_empty() {
            Some(instances[0].id.clone())
        } else {
            None
        };

        // 如果用户没有设置当前实例，设置为第一个分配的实例
        if user.current_instance_id.is_none() && current_instance_id.is_some() {
            User::update_current_instance(
                &self.db.pool,
                &user.id,
                current_instance_id.as_deref(),
            )
            .await?;
        }

        tracing::info!(user_id = %user.id, username = %username, "用户登录成功");

        Ok(LoginResponse {
            token,
            user: user.into(),
            instances,
            current_instance_id,
        })
    }

    /// 用户登出
    pub async fn logout(&self, token: &str) -> Result<(), AppError> {
        let token_hash = self.hash_token(token);
        UserSession::delete_by_token_hash(&self.db.pool, &token_hash).await?;
        Ok(())
    }

    /// 获取当前用户
    pub async fn get_current_user(&self, token: &str) -> Result<(User, Vec<InstanceInfo>), AppError> {
        let claims = self.verify_token(token)?;
        let user = User::get_by_id(&self.db.pool, &claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户不存在".to_string()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("用户已被停用".to_string()));
        }

        let instances = self.get_user_instances(&user.id).await?;

        Ok((user, instances))
    }

    /// 修改密码
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let user = User::get_by_id(&self.db.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        // 验证旧密码
        if !Self::verify_password(old_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("旧密码错误".to_string()));
        }

        // 哈希新密码
        let new_password_hash = Self::hash_password(new_password)?;

        // 更新密码
        User::update_password(&self.db.pool, user_id, &new_password_hash).await?;

        // 登出所有会话
        UserSession::delete_all_by_user(&self.db.pool, user_id).await?;

        tracing::info!(user_id = %user_id, "用户密码修改成功");

        Ok(())
    }

    // ==================== 会话操作 ====================

    /// 创建会话
    async fn create_session(
        &self,
        user: &User,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(String, UserSession), AppError> {
        // 生成 JWT
        let token = self.generate_token(user)?;
        let token_hash = self.hash_token(&token);

        let expires_at = Utc::now() + Duration::seconds(self.config.session_ttl_secs);

        // 限制会话数量
        UserSession::limit_user_sessions(
            &self.db.pool,
            &user.id,
            self.config.max_sessions_per_user,
        )
        .await?;

        // 创建会话
        let session_id = Uuid::new_v4().to_string();
        let session = UserSession::create(
            &self.db.pool,
            &session_id,
            &user.id,
            &token_hash,
            expires_at,
            ip_address,
            user_agent,
        )
        .await?;

        Ok((token, session))
    }

    /// 生成 JWT
    fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.session_ttl_secs);

        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Token 生成失败: {}", e)))?;

        Ok(token)
    }

    /// 验证 JWT
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Token 无效: {}", e)))?;

        Ok(token_data.claims)
    }

    /// 哈希 Token (用于数据库存储)
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 验证会话
    pub async fn verify_session(&self, token: &str) -> Result<User, AppError> {
        // 验证 JWT
        let claims = self.verify_token(token)?;

        // 检查会话是否存在
        let token_hash = self.hash_token(token);
        let session = UserSession::get_valid_by_token_hash(&self.db.pool, &token_hash)
            .await?
            .ok_or_else(|| AppError::Unauthorized("会话已过期".to_string()))?;

        // 获取用户
        let user = User::get_by_id(&self.db.pool, &claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户不存在".to_string()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("用户已被停用".to_string()));
        }

        // 检查是否需要刷新会话
        if session.needs_refresh(self.config.refresh_threshold_secs) {
            let new_expires_at = Utc::now() + Duration::seconds(self.config.session_ttl_secs);
            UserSession::extend(&self.db.pool, &session.id, new_expires_at).await?;
        }

        Ok(user)
    }

    // ==================== 实例操作 ====================

    /// 获取用户分配的实例
    pub async fn get_user_instances(&self, user_id: &str) -> Result<Vec<InstanceInfo>, AppError> {
        let assignments = UserInstanceAssignment::list_by_user(&self.db.pool, user_id).await?;

        let mut instances = Vec::new();
        for assignment in assignments {
            if let Some(instance) =
                VibeInstance::get_by_id(&self.db.pool, &assignment.instance_id).await?
            {
                let mut info: InstanceInfo = instance.into();
                info.user_count =
                    Some(VibeInstance::count_users(&self.db.pool, &assignment.instance_id).await?);
                instances.push(info);
            }
        }

        Ok(instances)
    }

    /// 检查用户是否被分配到指定实例
    pub async fn is_user_assigned_to_instance(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<bool, AppError> {
        let assignment = UserInstanceAssignment::get_by_user_and_instance(
            &self.db.pool,
            user_id,
            instance_id,
        )
        .await?;
        Ok(assignment.is_some())
    }

    /// 分配用户到实例
    pub async fn assign_user_to_instance(
        &self,
        admin: &User,
        user_id: &str,
        instance_id: &str,
    ) -> Result<(), AppError> {
        // 验证管理员权限
        if !admin.is_admin() {
            return Err(AppError::Forbidden("需要管理员权限".to_string()));
        }

        // 验证用户存在
        let user = User::get_by_id(&self.db.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        // 验证实例存在
        let instance = VibeInstance::get_by_id(&self.db.pool, instance_id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 检查实例用户数限制
        if let Some(max_users) = instance.max_users {
            if max_users > 0 {
                let current_users = VibeInstance::count_users(&self.db.pool, instance_id).await?;
                if current_users >= max_users as i64 {
                    return Err(AppError::Conflict("实例已达到用户数量上限".to_string()));
                }
            }
        }

        // 检查是否已分配
        if UserInstanceAssignment::is_assigned(&self.db.pool, user_id, instance_id).await? {
            return Ok(()); // 已分配，幂等操作
        }

        // 创建分配记录
        let assignment_id = Uuid::new_v4().to_string();
        UserInstanceAssignment::create(
            &self.db.pool,
            &assignment_id,
            user_id,
            instance_id,
            Some(&admin.id),
        )
        .await?;

        // 如果用户没有当前实例，设置为此实例
        if user.current_instance_id.is_none() {
            User::update_current_instance(&self.db.pool, user_id, Some(instance_id)).await?;
        }

        tracing::info!(
            user_id = %user_id,
            instance_id = %instance_id,
            admin_id = %admin.id,
            "用户分配到实例"
        );

        Ok(())
    }

    /// 取消用户的实例分配
    pub async fn unassign_user_from_instance(
        &self,
        admin: &User,
        user_id: &str,
        instance_id: &str,
    ) -> Result<(), AppError> {
        // 验证管理员权限
        if !admin.is_admin() {
            return Err(AppError::Forbidden("需要管理员权限".to_string()));
        }

        // 删除分配记录
        UserInstanceAssignment::delete_by_user_and_instance(&self.db.pool, user_id, instance_id)
            .await?;

        // 如果这是用户当前实例，切换到其他已分配实例
        let user = User::get_by_id(&self.db.pool, user_id).await?;
        if let Some(user) = user {
            if user.current_instance_id.as_deref() == Some(instance_id) {
                let other_instances = self.get_user_instances(user_id).await?;
                if let Some(first) = other_instances.first() {
                    User::update_current_instance(&self.db.pool, user_id, Some(&first.id)).await?;
                } else {
                    User::update_current_instance(&self.db.pool, user_id, None).await?;
                }
            }
        }

        tracing::info!(
            user_id = %user_id,
            instance_id = %instance_id,
            admin_id = %admin.id,
            "取消用户实例分配"
        );

        Ok(())
    }

    /// 用户切换当前实例
    pub async fn switch_instance(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<InstanceInfo, AppError> {
        // 验证用户有权访问该实例
        if !UserInstanceAssignment::is_assigned(&self.db.pool, user_id, instance_id).await? {
            return Err(AppError::Forbidden("您没有访问该实例的权限".to_string()));
        }

        // 获取实例
        let instance = VibeInstance::get_by_id(&self.db.pool, instance_id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 更新用户当前实例
        User::update_current_instance(&self.db.pool, user_id, Some(instance_id)).await?;

        tracing::info!(user_id = %user_id, instance_id = %instance_id, "用户切换实例");

        let mut info: InstanceInfo = instance.into();
        info.user_count = Some(VibeInstance::count_users(&self.db.pool, instance_id).await?);

        Ok(info)
    }

    // ==================== 用户管理 (管理员) ====================

    /// 列出所有用户
    pub async fn list_users(&self) -> Result<Vec<UserInfo>, AppError> {
        let users = User::list(&self.db.pool).await?;
        Ok(users.into_iter().map(UserInfo::from).collect())
    }

    /// 获取用户详情
    pub async fn get_user(&self, user_id: &str) -> Result<Option<UserInfo>, AppError> {
        let user = User::get_by_id(&self.db.pool, user_id).await?;
        Ok(user.map(UserInfo::from))
    }

    /// 更新用户信息
    pub async fn update_user(
        &self,
        user_id: &str,
        email: Option<&str>,
        display_name: Option<&str>,
        role: Option<UserRole>,
    ) -> Result<UserInfo, AppError> {
        let user = User::update(&self.db.pool, user_id, email, display_name, role).await?;
        Ok(user.into())
    }

    /// 激活/停用用户
    pub async fn set_user_active(&self, user_id: &str, is_active: bool) -> Result<(), AppError> {
        User::set_active(&self.db.pool, user_id, is_active).await?;

        if !is_active {
            // 停用用户时，删除所有会话
            UserSession::delete_all_by_user(&self.db.pool, user_id).await?;
        }

        tracing::info!(
            user_id = %user_id,
            is_active = %is_active,
            "用户状态变更"
        );

        Ok(())
    }

    /// 删除用户
    pub async fn delete_user(&self, user_id: &str) -> Result<(), AppError> {
        // 删除用户的所有会话
        UserSession::delete_all_by_user(&self.db.pool, user_id).await?;

        // 删除用户的所有实例分配
        UserInstanceAssignment::delete_all_by_user(&self.db.pool, user_id).await?;

        // 删除用户
        User::delete(&self.db.pool, user_id).await?;

        tracing::info!(user_id = %user_id, "用户已删除");

        Ok(())
    }

    /// 重置用户密码 (管理员)
    pub async fn reset_password(&self, user_id: &str, new_password: &str) -> Result<(), AppError> {
        let password_hash = Self::hash_password(new_password)?;
        User::update_password(&self.db.pool, user_id, &password_hash).await?;

        // 登出所有会话
        UserSession::delete_all_by_user(&self.db.pool, user_id).await?;

        tracing::info!(user_id = %user_id, "用户密码已重置");

        Ok(())
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, AppError> {
        let count = UserSession::cleanup_expired(&self.db.pool).await?;
        if count > 0 {
            tracing::info!(count = %count, "清理过期会话");
        }
        Ok(count)
    }
}
