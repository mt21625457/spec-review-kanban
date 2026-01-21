use std::sync::Arc;

use uuid::Uuid;

use crate::db::models::repo_mapping::RepoMapping;
use crate::db::models::review_event::ReviewEvent;
use crate::db::models::review_run::{CreateReviewRun, ReviewRun, ReviewStatus};
use crate::db::Database;
use crate::services::config_service::ConfigService;
use crate::services::gitea_client::GiteaClient;
use crate::services::vibe_client::{CreateAndStartTaskRequest, CreateTask, VibeClient, WorkspaceRepoInput};
use crate::services::webhook_handler::PullRequestPayload;

/// 审核服务
pub struct ReviewService {
    db: Arc<Database>,
    config: Arc<ConfigService>,
    gitea: Arc<GiteaClient>,
    vibe: Arc<VibeClient>,
}

impl ReviewService {
    pub fn new(
        db: Arc<Database>,
        config: Arc<ConfigService>,
        gitea: Arc<GiteaClient>,
        vibe: Arc<VibeClient>,
    ) -> Self {
        Self {
            db,
            config,
            gitea,
            vibe,
        }
    }

    /// 处理 PR 审核请求
    pub async fn handle_pr_review(&self, payload: &PullRequestPayload) -> anyhow::Result<ReviewRun> {
        let repo_name = &payload.repository.full_name;

        // 查找仓库映射
        let mapping = RepoMapping::get_by_gitea_repo(&self.db.pool, repo_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("仓库 {} 未配置映射", repo_name))?;

        if !mapping.is_enabled {
            anyhow::bail!("仓库 {} 的映射已禁用", repo_name);
        }

        // 创建审核运行记录
        let review_run = ReviewRun::create(
            &self.db.pool,
            CreateReviewRun {
                repo_mapping_id: mapping.id.clone(),
                gitea_pr_number: payload.number,
                gitea_pr_url: Some(payload.pull_request.html_url.clone()),
            },
        )
        .await?;

        // 记录事件
        ReviewEvent::create(
            &self.db.pool,
            &review_run.id,
            "created",
            Some(&serde_json::to_string(&serde_json::json!({
                "pr_number": payload.number,
                "pr_title": payload.pull_request.title,
            }))?),
        )
        .await?;

        // 获取审核配置
        let review_settings = self.config.get_review_settings().await?;

        if review_settings.auto_start {
            // 自动启动审核
            self.start_review(&review_run, &mapping, payload).await?;
        }

        Ok(review_run)
    }

    /// 启动审核
    pub async fn start_review(
        &self,
        review_run: &ReviewRun,
        mapping: &RepoMapping,
        payload: &PullRequestPayload,
    ) -> anyhow::Result<ReviewRun> {
        // 检查 Vibe Kanban 项目是否配置
        let project_id = mapping
            .vibe_project_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| anyhow::anyhow!("仓库映射未配置 Vibe Kanban 项目"))?;

        // 检查 executor_profile_id
        let executor_profile_id = mapping
            .executor_profile_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("仓库映射未配置执行器配置"))?;

        // 构建任务标题
        let task_title = format!(
            "PR #{}: {} 代码审核",
            payload.number, payload.pull_request.title
        );

        // 构建任务描述
        let description = self.build_review_prompt(mapping, payload).await?;

        // 创建任务请求
        let request = CreateAndStartTaskRequest {
            task: CreateTask {
                project_id,
                title: task_title,
                column: "backlog".to_string(),
                description: Some(description),
            },
            executor_profile_id: executor_profile_id.clone(),
            repos: vec![WorkspaceRepoInput {
                url: mapping.local_path.clone(),
                branch: Some(payload.pull_request.head.ref_name.clone()),
            }],
        };

        // 调用 Vibe Kanban API
        let task_response = self.vibe.create_and_start_task(request).await?;

        // 更新审核运行状态
        let updated = review_run
            .start(
                &self.db.pool,
                &task_response.id.to_string(),
                task_response.workspace_id.as_ref().map(|id| id.to_string()).as_deref(),
            )
            .await?;

        // 记录事件
        ReviewEvent::create(
            &self.db.pool,
            &review_run.id,
            "started",
            Some(&serde_json::to_string(&serde_json::json!({
                "vibe_task_id": task_response.id,
                "vibe_workspace_id": task_response.workspace_id,
            }))?),
        )
        .await?;

        // 更新 Gitea commit status
        let (owner, repo) = self.parse_repo_name(&payload.repository.full_name)?;
        let _ = self
            .gitea
            .create_commit_status(
                &owner,
                &repo,
                &payload.pull_request.head.sha,
                "pending",
                "aicodex/review",
                Some("代码审核进行中"),
                None,
            )
            .await;

        Ok(updated)
    }

    /// 构建审核 Prompt
    async fn build_review_prompt(
        &self,
        mapping: &RepoMapping,
        payload: &PullRequestPayload,
    ) -> anyhow::Result<String> {
        let custom_prompt = mapping.custom_prompt.as_deref().unwrap_or("");

        let prompt = format!(
            r#"请对以下 Pull Request 进行代码审核：

## PR 信息
- **标题**: {}
- **分支**: {} -> {}
- **URL**: {}

## 审核要求
1. 检查代码风格和一致性
2. 检查潜在的 bug 和逻辑错误
3. 检查安全漏洞
4. 检查性能问题
5. 提供改进建议

{}

请使用 git diff 命令查看具体的代码变更，然后提供详细的审核意见。"#,
            payload.pull_request.title,
            payload.pull_request.head.ref_name,
            payload.pull_request.base.ref_name,
            payload.pull_request.html_url,
            if custom_prompt.is_empty() {
                "".to_string()
            } else {
                format!("## 额外要求\n{}", custom_prompt)
            }
        );

        Ok(prompt)
    }

    /// 解析仓库名 (owner/repo)
    fn parse_repo_name(&self, full_name: &str) -> anyhow::Result<(String, String)> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("无效的仓库名格式: {}", full_name);
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 列出审核运行
    pub async fn list_reviews(&self, limit: i64) -> anyhow::Result<Vec<ReviewRun>> {
        ReviewRun::list(&self.db.pool, limit).await.map_err(Into::into)
    }

    /// 获取审核详情
    pub async fn get_review(&self, id: &str) -> anyhow::Result<Option<ReviewRun>> {
        ReviewRun::get_by_id(&self.db.pool, id).await.map_err(Into::into)
    }

    /// 获取审核事件
    pub async fn get_review_events(&self, review_run_id: &str) -> anyhow::Result<Vec<ReviewEvent>> {
        ReviewEvent::list_for_review_run(&self.db.pool, review_run_id)
            .await
            .map_err(Into::into)
    }

    /// 重新运行审核
    pub async fn rerun_review(&self, id: &str) -> anyhow::Result<ReviewRun> {
        let review_run = ReviewRun::get_by_id(&self.db.pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("审核运行不存在"))?;

        // 获取仓库映射
        let mapping = RepoMapping::get_by_id(&self.db.pool, &review_run.repo_mapping_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("仓库映射不存在"))?;

        // 获取 PR 信息
        let (owner, repo) = self.parse_repo_name(&mapping.gitea_repo)?;
        let pr = self
            .gitea
            .get_pull_request(&owner, &repo, review_run.gitea_pr_number)
            .await?;

        // 构建 payload
        let payload = PullRequestPayload {
            action: "synchronize".to_string(),
            number: pr.number,
            pull_request: crate::services::webhook_handler::PullRequestInfo {
                number: pr.number,
                title: pr.title,
                html_url: pr.html_url,
                head: crate::services::webhook_handler::BranchInfo {
                    ref_name: pr.head.ref_name,
                    sha: pr.head.sha,
                },
                base: crate::services::webhook_handler::BranchInfo {
                    ref_name: pr.base.ref_name,
                    sha: pr.base.sha,
                },
            },
            repository: crate::services::webhook_handler::RepositoryInfo {
                full_name: mapping.gitea_repo.clone(),
                clone_url: String::new(),
            },
        };

        // 创建新的审核运行
        self.handle_pr_review(&payload).await
    }

    /// 取消审核
    pub async fn cancel_review(&self, id: &str) -> anyhow::Result<ReviewRun> {
        let review_run = ReviewRun::get_by_id(&self.db.pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("审核运行不存在"))?;

        let status: ReviewStatus = review_run.status.parse().unwrap_or(ReviewStatus::Pending);
        if !matches!(status, ReviewStatus::Pending | ReviewStatus::Running) {
            anyhow::bail!("只能取消待处理或运行中的审核");
        }

        let updated = review_run.cancel(&self.db.pool).await?;

        ReviewEvent::create(
            &self.db.pool,
            &review_run.id,
            "cancelled",
            None,
        )
        .await?;

        Ok(updated)
    }
}
