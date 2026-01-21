use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建任务请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateTask {
    pub project_id: Uuid,
    pub title: String,
    pub column: String,
    pub description: Option<String>,
}

/// 创建并启动任务请求
#[derive(Debug, Clone, Serialize)]
pub struct CreateAndStartTaskRequest {
    pub task: CreateTask,
    pub executor_profile_id: String,
    pub repos: Vec<WorkspaceRepoInput>,
}

/// 工作空间仓库输入
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRepoInput {
    pub url: String,
    pub branch: Option<String>,
}

/// 任务响应
#[derive(Debug, Clone, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub workspace_id: Option<Uuid>,
}

/// 工作空间响应
#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub status: String,
    pub execution_id: Option<Uuid>,
}

/// 健康检查响应
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

/// Vibe Kanban 客户端
pub struct VibeClient {
    base_url: String,
    client: reqwest::Client,
}

impl VibeClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> anyhow::Result<bool> {
        let resp = self
            .client
            .get(format!("{}/api/health", self.base_url))
            .send()
            .await?;

        Ok(resp.status().is_success())
    }

    /// 创建并启动任务
    pub async fn create_and_start_task(
        &self,
        request: CreateAndStartTaskRequest,
    ) -> anyhow::Result<TaskResponse> {
        let resp = self
            .client
            .post(format!("{}/api/tasks/create-and-start", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("创建任务失败 ({}): {}", status, body);
        }

        resp.json().await.map_err(Into::into)
    }

    /// 获取工作空间状态
    pub async fn get_workspace(&self, workspace_id: Uuid) -> anyhow::Result<WorkspaceResponse> {
        let resp = self
            .client
            .get(format!("{}/api/workspaces/{}", self.base_url, workspace_id))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("获取工作空间失败 ({}): {}", status, body);
        }

        resp.json().await.map_err(Into::into)
    }

    /// 获取任务状态
    pub async fn get_task(&self, task_id: Uuid) -> anyhow::Result<TaskResponse> {
        let resp = self
            .client
            .get(format!("{}/api/tasks/{}", self.base_url, task_id))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("获取任务失败 ({}): {}", status, body);
        }

        resp.json().await.map_err(Into::into)
    }

    /// 代理请求
    pub async fn proxy_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> anyhow::Result<(u16, String)> {
        let url = format!("{}{}", self.base_url, path);

        let mut req = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            "PATCH" => self.client.patch(&url),
            _ => anyhow::bail!("不支持的 HTTP 方法: {}", method),
        };

        if let Some(body) = body {
            req = req.header("Content-Type", "application/json").body(body.to_string());
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;

        Ok((status, body))
    }
}
