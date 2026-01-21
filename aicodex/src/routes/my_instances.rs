//! 用户实例切换 API
//!
//! 提供用户自己的实例管理功能（查看分配的实例、切换当前实例）

use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::error::ApiError;
use crate::middleware::AuthenticatedUser;
use crate::AppState;

/// 切换实例请求
#[derive(Debug, Deserialize)]
pub struct SwitchInstanceRequest {
    pub instance_id: String,
}

/// 创建用户实例路由
pub fn my_instances_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_my_instances))
        .route("/current", get(get_current_instance).put(switch_instance))
        .route("/current/health", get(current_instance_health))
}

/// 获取用户分配的所有实例
async fn list_my_instances(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let instances = state
        .services
        .user_manager
        .get_user_instances(auth_user.id())
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instances": instances,
            "current_instance_id": auth_user.current_instance_id(),
        }
    })))
}

/// 获取当前实例
async fn get_current_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let current_id = auth_user
        .current_instance_id()
        .ok_or_else(|| ApiError::NotFound("未选择实例".to_string()))?;

    let instance = state
        .services
        .instance_manager
        .get(current_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("当前实例不存在".to_string()))?;

    let mut info = crate::db::models::vibe_instance::InstanceInfo::from(instance);
    info.user_count = Some(
        crate::db::models::vibe_instance::VibeInstance::count_users(
            &state.db.pool,
            current_id,
        )
        .await?,
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instance": info,
        }
    })))
}

/// 切换当前实例
async fn switch_instance(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(req): Json<SwitchInstanceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 切换实例
    let _instance = state
        .services
        .user_manager
        .switch_instance(auth_user.id(), &req.instance_id)
        .await?;

    // 如果实例未运行且配置了自动启动，则启动
    let instance_detail = state
        .services
        .instance_manager
        .get(&req.instance_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    if instance_detail.status_enum() != crate::db::models::vibe_instance::InstanceStatus::Running
        && instance_detail.auto_start
    {
        if let Err(e) = state.services.instance_manager.start(&req.instance_id).await {
            tracing::warn!(
                instance_id = %req.instance_id,
                error = %e,
                "自动启动实例失败"
            );
        }
    }

    // 重新获取实例信息
    let updated_instance = state
        .services
        .instance_manager
        .get(&req.instance_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("实例不存在".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已切换到实例",
        "data": {
            "instance": crate::db::models::vibe_instance::InstanceInfo::from(updated_instance),
        }
    })))
}

/// 当前实例健康检查
async fn current_instance_health(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let current_id = auth_user
        .current_instance_id()
        .ok_or_else(|| ApiError::NotFound("未选择实例".to_string()))?;

    let health = state
        .services
        .instance_manager
        .health_check(current_id)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "instance_id": current_id,
            "health_status": health.to_string(),
        }
    })))
}
