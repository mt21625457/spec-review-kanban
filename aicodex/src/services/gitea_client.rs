use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::services::config_service::ConfigService;

/// PR 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub head: PrBranch,
    pub base: PrBranch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrBranch {
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiteaUser {
    pub id: i64,
    pub login: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}

/// 连接测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub user: Option<GiteaUser>,
    pub error: Option<String>,
}

/// Gitea 客户端
pub struct GiteaClient {
    config: Arc<ConfigService>,
    client: reqwest::Client,
}

impl GiteaClient {
    pub fn new(config: Arc<ConfigService>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// 获取 API URL 和 Token
    async fn get_credentials(&self) -> anyhow::Result<(String, String)> {
        let settings = self.config.get_gitea_settings().await?;
        let api_url = settings
            .api_url
            .ok_or_else(|| anyhow::anyhow!("Gitea API URL 未配置"))?;
        let token = settings
            .token
            .ok_or_else(|| anyhow::anyhow!("Gitea Token 未配置"))?;
        Ok((api_url, token))
    }

    /// 测试连接
    pub async fn test_connection(&self) -> ConnectionTestResult {
        match self.get_current_user().await {
            Ok(user) => ConnectionTestResult {
                success: true,
                user: Some(user),
                error: None,
            },
            Err(e) => ConnectionTestResult {
                success: false,
                user: None,
                error: Some(e.to_string()),
            },
        }
    }

    /// 获取当前用户
    pub async fn get_current_user(&self) -> anyhow::Result<GiteaUser> {
        let (api_url, token) = self.get_credentials().await?;

        let resp = self
            .client
            .get(format!("{}/api/v1/user", api_url))
            .header("Authorization", format!("token {}", token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Gitea API 错误 ({}): {}", status, body);
        }

        resp.json().await.map_err(Into::into)
    }

    /// 获取 PR 信息
    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: i64,
    ) -> anyhow::Result<PullRequest> {
        let (api_url, token) = self.get_credentials().await?;

        let resp = self
            .client
            .get(format!(
                "{}/api/v1/repos/{}/{}/pulls/{}",
                api_url, owner, repo, pr_number
            ))
            .header("Authorization", format!("token {}", token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("获取 PR 失败 ({}): {}", status, body);
        }

        resp.json().await.map_err(Into::into)
    }

    /// 获取 PR diff
    pub async fn get_pull_request_diff(
        &self,
        owner: &str,
        repo: &str,
        pr_number: i64,
    ) -> anyhow::Result<String> {
        let (api_url, token) = self.get_credentials().await?;

        let resp = self
            .client
            .get(format!(
                "{}/api/v1/repos/{}/{}/pulls/{}.diff",
                api_url, owner, repo, pr_number
            ))
            .header("Authorization", format!("token {}", token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("获取 PR diff 失败 ({}): {}", status, body);
        }

        resp.text().await.map_err(Into::into)
    }

    /// 发布 PR 评论
    pub async fn create_pull_request_comment(
        &self,
        owner: &str,
        repo: &str,
        pr_number: i64,
        body: &str,
    ) -> anyhow::Result<()> {
        let (api_url, token) = self.get_credentials().await?;

        let resp = self
            .client
            .post(format!(
                "{}/api/v1/repos/{}/{}/issues/{}/comments",
                api_url, owner, repo, pr_number
            ))
            .header("Authorization", format!("token {}", token))
            .json(&serde_json::json!({ "body": body }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("发布评论失败 ({}): {}", status, body);
        }

        Ok(())
    }

    /// 更新 commit status
    pub async fn create_commit_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        state: &str,
        context: &str,
        description: Option<&str>,
        target_url: Option<&str>,
    ) -> anyhow::Result<()> {
        let (api_url, token) = self.get_credentials().await?;

        let mut payload = serde_json::json!({
            "state": state,
            "context": context,
        });

        if let Some(desc) = description {
            payload["description"] = serde_json::Value::String(desc.to_string());
        }
        if let Some(url) = target_url {
            payload["target_url"] = serde_json::Value::String(url.to_string());
        }

        let resp = self
            .client
            .post(format!(
                "{}/api/v1/repos/{}/{}/statuses/{}",
                api_url, owner, repo, sha
            ))
            .header("Authorization", format!("token {}", token))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("更新 commit status 失败 ({}): {}", status, body);
        }

        Ok(())
    }
}
