//! 实例管理服务
//!
//! 提供 vibe-kanban 实例的生命周期管理

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::models::instance_ai_agent::InstanceAiAgent;
use crate::db::models::user_instance_assignment::UserInstanceAssignment;
use crate::db::models::vibe_instance::{HealthStatus, InstanceInfo, InstanceStatus, VibeInstance};
use crate::db::Database;
use crate::error::AppError;

/// 实例配置
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    /// vibe-kanban 可执行文件路径
    pub vibe_kanban_bin: PathBuf,
    /// 数据根目录
    pub data_root: PathBuf,
    /// 端口范围起始
    pub port_base: i32,
    /// 端口范围结束
    pub port_max: i32,
    /// 健康检查间隔（秒）
    pub health_check_interval_secs: u64,
    /// 启动超时（秒）
    pub startup_timeout_secs: u64,
    /// 停止超时（秒）
    pub shutdown_timeout_secs: u64,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            vibe_kanban_bin: PathBuf::from("/usr/local/bin/vibe-kanban"),
            data_root: PathBuf::from("/data/vibe-instances"),
            port_base: 18100,
            port_max: 18199,
            health_check_interval_secs: 30,
            startup_timeout_secs: 30,
            shutdown_timeout_secs: 30,
        }
    }
}

/// 运行中的进程信息
struct RunningProcess {
    child: Child,
    #[allow(dead_code)]
    started_at: chrono::DateTime<chrono::Utc>,
}

/// 实例管理服务
pub struct InstanceManager {
    db: Arc<Database>,
    config: InstanceConfig,
    /// 运行中的进程
    processes: Arc<RwLock<HashMap<String, RunningProcess>>>,
}

impl InstanceManager {
    /// 创建实例管理服务
    pub fn new(db: Arc<Database>, config: InstanceConfig) -> Self {
        Self {
            db,
            config,
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认配置的实例管理服务
    pub fn with_defaults(db: Arc<Database>) -> Self {
        Self::new(db, InstanceConfig::default())
    }

    // ==================== 实例 CRUD ====================

    /// 创建实例
    pub async fn create(
        &self,
        name: &str,
        description: Option<&str>,
        auto_start: bool,
        max_users: Option<i32>,
    ) -> Result<VibeInstance, AppError> {
        // 分配端口
        let port = VibeInstance::next_available_port(
            &self.db.pool,
            self.config.port_base,
            self.config.port_max,
        )
        .await?
        .ok_or_else(|| AppError::Conflict("没有可用的端口".to_string()))?;

        // 生成 ID
        let id = Uuid::new_v4().to_string();

        // 创建数据目录
        let data_dir = self.config.data_root.join(&id);
        self.create_data_directories(&data_dir).await?;

        // 创建实例记录
        let instance = VibeInstance::create(
            &self.db.pool,
            &id,
            name,
            description,
            port,
            data_dir.to_string_lossy().as_ref(),
            auto_start,
            max_users,
        )
        .await?;

        tracing::info!(
            instance_id = %id,
            name = %name,
            port = %port,
            "实例已创建"
        );

        Ok(instance)
    }

    /// 获取实例
    pub async fn get(&self, id: &str) -> Result<Option<VibeInstance>, AppError> {
        Ok(VibeInstance::get_by_id(&self.db.pool, id).await?)
    }

    /// 列出所有实例
    pub async fn list(&self) -> Result<Vec<InstanceInfo>, AppError> {
        let instances = VibeInstance::list(&self.db.pool).await?;
        let mut infos = Vec::new();

        for instance in instances {
            let user_count = VibeInstance::count_users(&self.db.pool, &instance.id).await?;
            let mut info: InstanceInfo = instance.into();
            info.user_count = Some(user_count);
            infos.push(info);
        }

        Ok(infos)
    }

    /// 更新实例
    pub async fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        auto_start: Option<bool>,
        max_users: Option<i32>,
    ) -> Result<VibeInstance, AppError> {
        let instance =
            VibeInstance::update(&self.db.pool, id, name, description, auto_start, max_users)
                .await?;

        tracing::info!(instance_id = %id, "实例已更新");

        Ok(instance)
    }

    /// 删除实例
    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        // 获取实例
        let instance = self
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 检查实例是否已停止
        if instance.status_enum() != InstanceStatus::Stopped {
            return Err(AppError::Conflict("实例必须先停止才能删除".to_string()));
        }

        // 检查是否有用户分配
        let user_count = VibeInstance::count_users(&self.db.pool, id).await?;
        if user_count > 0 {
            return Err(AppError::Conflict(format!(
                "实例还有 {} 个用户分配，请先取消分配",
                user_count
            )));
        }

        // 删除数据目录
        let data_dir = PathBuf::from(&instance.data_dir);
        if data_dir.exists() {
            tokio::fs::remove_dir_all(&data_dir).await.map_err(|e| {
                AppError::Internal(format!("删除数据目录失败: {}", e))
            })?;
        }

        // 删除数据库记录
        VibeInstance::delete(&self.db.pool, id).await?;

        tracing::info!(instance_id = %id, "实例已删除");

        Ok(())
    }

    // ==================== 生命周期管理 ====================

    /// 启动实例
    pub async fn start(&self, id: &str) -> Result<(), AppError> {
        let instance = self
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 检查是否已在运行
        if instance.status_enum() == InstanceStatus::Running {
            return Ok(());
        }

        // 更新状态为启动中，清除之前的错误
        VibeInstance::update_status_with_error(&self.db.pool, id, InstanceStatus::Starting, None).await?;

        // 准备环境变量
        let env_vars = match self.prepare_environment(&instance).await {
            Ok(vars) => vars,
            Err(e) => {
                let error_msg = format!("准备环境变量失败: {}", e);
                VibeInstance::update_status_with_error(&self.db.pool, id, InstanceStatus::Error, Some(&error_msg)).await?;
                return Err(e);
            }
        };

        // 启动进程
        match self.spawn_process(&instance, env_vars).await {
            Ok(child) => {
                // 保存进程句柄
                let running_process = RunningProcess {
                    child,
                    started_at: chrono::Utc::now(),
                };
                self.processes.write().await.insert(id.to_string(), running_process);

                // 等待健康检查通过
                match self.wait_for_healthy(&instance).await {
                    Ok(_) => {
                        // 启动成功，清除错误
                        VibeInstance::update_status_with_error(&self.db.pool, id, InstanceStatus::Running, None)
                            .await?;
                        VibeInstance::update_health_status(&self.db.pool, id, HealthStatus::Healthy)
                            .await?;
                        tracing::info!(instance_id = %id, "实例已启动");
                    }
                    Err(e) => {
                        // 启动超时，停止进程
                        self.kill_process(id).await;
                        let error_msg = format!("健康检查失败: {}", e);
                        VibeInstance::update_status_with_error(&self.db.pool, id, InstanceStatus::Error, Some(&error_msg))
                            .await?;
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("启动进程失败: {}", e);
                VibeInstance::update_status_with_error(&self.db.pool, id, InstanceStatus::Error, Some(&error_msg)).await?;
                return Err(e);
            }
        }

        Ok(())
    }

    /// 停止实例
    pub async fn stop(&self, id: &str) -> Result<(), AppError> {
        let instance = self
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        // 检查是否已停止
        if instance.status_enum() == InstanceStatus::Stopped {
            return Ok(());
        }

        // 更新状态为停止中
        VibeInstance::update_status(&self.db.pool, id, InstanceStatus::Stopping).await?;

        // 停止进程
        self.stop_process(id).await;

        // 更新状态为已停止
        VibeInstance::update_status(&self.db.pool, id, InstanceStatus::Stopped).await?;
        VibeInstance::update_health_status(&self.db.pool, id, HealthStatus::Unknown).await?;

        tracing::info!(instance_id = %id, "实例已停止");

        Ok(())
    }

    /// 重启实例
    pub async fn restart(&self, id: &str) -> Result<(), AppError> {
        self.stop(id).await?;
        self.start(id).await?;
        Ok(())
    }

    // ==================== 健康检查 ====================

    /// 检查实例健康状态
    pub async fn health_check(&self, id: &str) -> Result<HealthStatus, AppError> {
        let instance = self
            .get(id)
            .await?
            .ok_or_else(|| AppError::NotFound("实例不存在".to_string()))?;

        if instance.status_enum() != InstanceStatus::Running {
            return Ok(HealthStatus::Unknown);
        }

        let health_url = format!("http://127.0.0.1:{}/api/health", instance.port);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP 客户端创建失败: {}", e)))?;

        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                VibeInstance::update_health_status(&self.db.pool, id, HealthStatus::Healthy)
                    .await?;
                Ok(HealthStatus::Healthy)
            }
            _ => {
                VibeInstance::update_health_status(&self.db.pool, id, HealthStatus::Unhealthy)
                    .await?;
                Ok(HealthStatus::Unhealthy)
            }
        }
    }

    /// 获取实例用户列表
    pub async fn get_instance_users(&self, instance_id: &str) -> Result<Vec<crate::db::models::user_instance_assignment::UserInstanceAssignmentInfo>, AppError> {
        use crate::db::models::user_instance_assignment::UserInstanceAssignmentInfo;
        let assignments = UserInstanceAssignmentInfo::list_by_instance_with_user(&self.db.pool, instance_id).await?;
        Ok(assignments)
    }

    // ==================== 内部方法 ====================

    /// 创建数据目录结构
    async fn create_data_directories(&self, data_dir: &Path) -> Result<(), AppError> {
        let dirs = [
            "db",
            "config",
            "worktrees",
            "logs",
            "ai-agents/claude-code",
            "ai-agents/codex-cli",
            "ai-agents/gemini-cli",
            "ai-agents/opencode",
        ];

        for dir in &dirs {
            let path = data_dir.join(dir);
            tokio::fs::create_dir_all(&path).await.map_err(|e| {
                AppError::Internal(format!("创建目录失败 {:?}: {}", path, e))
            })?;
        }

        Ok(())
    }

    /// 准备环境变量
    async fn prepare_environment(
        &self,
        instance: &VibeInstance,
    ) -> Result<Vec<(String, String)>, AppError> {
        let mut env = vec![
            // vibe-kanban 使用 BACKEND_PORT 或 PORT
            ("PORT".to_string(), instance.port.to_string()),
            ("BACKEND_PORT".to_string(), instance.port.to_string()),
            ("HOST".to_string(), "127.0.0.1".to_string()),
            // 数据目录相关（需要 vibe-kanban 支持）
            ("VIBE_DATA_DIR".to_string(), instance.data_dir.clone()),
            // 日志级别 - vibe-kanban 期望简单的级别名称（info/debug/warn）
            ("RUST_LOG".to_string(), "info".to_string()),
        ];

        // 加载 AI 智能体配置
        let agents = InstanceAiAgent::list_enabled_by_instance(&self.db.pool, &instance.id).await?;
        for agent in agents {
            env.extend(self.get_agent_env_vars(instance, &agent).await?);
        }

        Ok(env)
    }

    /// 获取智能体环境变量
    async fn get_agent_env_vars(
        &self,
        instance: &VibeInstance,
        agent: &InstanceAiAgent,
    ) -> Result<Vec<(String, String)>, AppError> {
        let mut env = Vec::new();

        match agent.agent_type.as_str() {
            "claude-code" => {
                if let Some(ref api_key) = agent.api_key_encrypted {
                    // TODO: 解密 API key
                    env.push(("ANTHROPIC_API_KEY".to_string(), api_key.clone()));
                }
                env.push((
                    "CLAUDE_CONFIG_DIR".to_string(),
                    format!("{}/ai-agents/claude-code", instance.data_dir),
                ));
            }
            "codex-cli" => {
                if let Some(ref api_key) = agent.api_key_encrypted {
                    env.push(("OPENAI_API_KEY".to_string(), api_key.clone()));
                }
                env.push((
                    "CODEX_CONFIG_HOME".to_string(),
                    format!("{}/ai-agents/codex-cli", instance.data_dir),
                ));
            }
            "gemini-cli" => {
                if let Some(ref api_key) = agent.api_key_encrypted {
                    env.push(("GOOGLE_API_KEY".to_string(), api_key.clone()));
                }
                env.push((
                    "GEMINI_CONFIG_DIR".to_string(),
                    format!("{}/ai-agents/gemini-cli", instance.data_dir),
                ));
            }
            "opencode" => {
                env.push((
                    "OPENCODE_CONFIG_DIR".to_string(),
                    format!("{}/ai-agents/opencode", instance.data_dir),
                ));
            }
            _ => {}
        }

        Ok(env)
    }

    /// 启动进程
    async fn spawn_process(
        &self,
        instance: &VibeInstance,
        env_vars: Vec<(String, String)>,
    ) -> Result<Child, AppError> {
        tracing::info!(
            instance_id = %instance.id,
            port = %instance.port,
            data_dir = %instance.data_dir,
            bin = ?self.config.vibe_kanban_bin,
            "准备启动进程"
        );

        // 检查可执行文件是否存在
        if !self.config.vibe_kanban_bin.exists() {
            return Err(AppError::Internal(format!(
                "vibe-kanban 可执行文件不存在: {:?}",
                self.config.vibe_kanban_bin
            )));
        }

        // 打印环境变量（用于调试）
        for (key, value) in &env_vars {
            tracing::debug!(key = %key, value = %value, "设置环境变量");
        }

        let mut child = Command::new(&self.config.vibe_kanban_bin)
            .envs(env_vars)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    bin = ?self.config.vibe_kanban_bin,
                    "启动进程失败"
                );
                AppError::Internal(format!(
                    "启动进程失败 {:?}: {}",
                    self.config.vibe_kanban_bin, e
                ))
            })?;

        // 异步收集子进程的日志
        let instance_id = instance.id.clone();
        if let Some(stdout) = child.stdout.take() {
            let id = instance_id.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    tracing::info!(instance_id = %id, source = "stdout", "{}", line);
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let id = instance_id.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    tracing::warn!(instance_id = %id, source = "stderr", "{}", line);
                }
            });
        }

        tracing::info!(
            instance_id = %instance.id,
            pid = ?child.id(),
            "进程已启动，等待健康检查..."
        );

        Ok(child)
    }

    /// 等待健康检查通过
    async fn wait_for_healthy(&self, instance: &VibeInstance) -> Result<(), AppError> {
        let health_url = format!("http://127.0.0.1:{}/api/health", instance.port);
        let timeout = Duration::from_secs(self.config.startup_timeout_secs);
        let interval = Duration::from_millis(500);
        let start = std::time::Instant::now();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::Internal(format!("HTTP 客户端创建失败: {}", e)))?;

        while start.elapsed() < timeout {
            match client.get(&health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(interval).await;
                }
            }
        }

        Err(AppError::Timeout("实例启动超时".to_string()))
    }

    /// 停止进程
    async fn stop_process(&self, id: &str) {
        let mut processes = self.processes.write().await;
        if let Some(mut running) = processes.remove(id) {
            // 发送 SIGTERM
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = running.child.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
            }

            // 等待优雅关闭
            let timeout = Duration::from_secs(self.config.shutdown_timeout_secs);
            match tokio::time::timeout(timeout, running.child.wait()).await {
                Ok(_) => {
                    tracing::debug!(instance_id = %id, "进程已优雅关闭");
                }
                Err(_) => {
                    // 超时，强制终止
                    let _ = running.child.kill().await;
                    tracing::warn!(instance_id = %id, "进程强制终止");
                }
            }
        }
    }

    /// 强制终止进程
    async fn kill_process(&self, id: &str) {
        let mut processes = self.processes.write().await;
        if let Some(mut running) = processes.remove(id) {
            let _ = running.child.kill().await;
            tracing::debug!(instance_id = %id, "进程已强制终止");
        }
    }

    /// 恢复运行中的实例（服务重启后）
    pub async fn recover_instances(&self) -> Result<(), AppError> {
        let instances = VibeInstance::list_running(&self.db.pool).await?;

        for instance in instances {
            tracing::info!(
                instance_id = %instance.id,
                "检查实例状态..."
            );

            // 检查进程是否还在运行
            match self.health_check(&instance.id).await {
                Ok(HealthStatus::Healthy) => {
                    tracing::info!(instance_id = %instance.id, "实例仍在运行");
                }
                _ => {
                    // 实例不健康，尝试重启
                    tracing::warn!(
                        instance_id = %instance.id,
                        "实例不健康，尝试重启"
                    );
                    VibeInstance::update_status(&self.db.pool, &instance.id, InstanceStatus::Stopped)
                        .await?;

                    if instance.auto_start {
                        if let Err(e) = self.start(&instance.id).await {
                            tracing::error!(
                                instance_id = %instance.id,
                                error = %e,
                                "实例重启失败"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
