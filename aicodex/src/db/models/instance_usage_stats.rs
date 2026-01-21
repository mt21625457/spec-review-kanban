//! 实例使用统计数据模型

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// 实例使用统计
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InstanceUsageStats {
    pub id: String,
    pub instance_id: String,
    pub agent_type: String,
    pub date: String,
    pub request_count: i32,
    pub token_count: i32,
    pub error_count: i32,
}

impl InstanceUsageStats {
    /// 根据 ID 获取统计
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM instance_usage_stats WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// 根据实例、智能体类型和日期获取统计
    pub async fn get_by_instance_agent_date(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        agent_type: &str,
        date: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_usage_stats WHERE instance_id = ? AND agent_type = ? AND date = ?",
        )
        .bind(instance_id)
        .bind(agent_type)
        .bind(date)
        .fetch_optional(pool)
        .await
    }

    /// 获取实例的统计列表
    pub async fn list_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_usage_stats WHERE instance_id = ? ORDER BY date DESC, agent_type",
        )
        .bind(instance_id)
        .fetch_all(pool)
        .await
    }

    /// 获取实例在日期范围内的统计
    pub async fn list_by_instance_date_range(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM instance_usage_stats WHERE instance_id = ? AND date >= ? AND date <= ? ORDER BY date DESC, agent_type",
        )
        .bind(instance_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await
    }

    /// 增加请求计数
    pub async fn increment_request(
        pool: &Pool<Sqlite>,
        id: &str,
        instance_id: &str,
        agent_type: &str,
        date: &str,
        token_count: i32,
        is_error: bool,
    ) -> Result<Self, sqlx::Error> {
        let error_increment = if is_error { 1 } else { 0 };

        sqlx::query(
            r#"
            INSERT INTO instance_usage_stats (id, instance_id, agent_type, date, request_count, token_count, error_count)
            VALUES (?, ?, ?, ?, 1, ?, ?)
            ON CONFLICT(instance_id, agent_type, date) DO UPDATE SET
                request_count = instance_usage_stats.request_count + 1,
                token_count = instance_usage_stats.token_count + excluded.token_count,
                error_count = instance_usage_stats.error_count + excluded.error_count
            "#,
        )
        .bind(id)
        .bind(instance_id)
        .bind(agent_type)
        .bind(date)
        .bind(token_count)
        .bind(error_increment)
        .execute(pool)
        .await?;

        Self::get_by_instance_agent_date(pool, instance_id, agent_type, date)
            .await?
            .ok_or(sqlx::Error::RowNotFound)
    }

    /// 删除实例的所有统计
    pub async fn delete_all_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM instance_usage_stats WHERE instance_id = ?")
            .bind(instance_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// 删除指定日期之前的统计
    pub async fn delete_before_date(
        pool: &Pool<Sqlite>,
        before_date: &str,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM instance_usage_stats WHERE date < ?")
            .bind(before_date)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }
}

/// 实例统计汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStatsSummary {
    pub instance_id: String,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_errors: i64,
    pub stats_by_agent: Vec<AgentStatsSummary>,
}

/// 按智能体汇总的统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatsSummary {
    pub agent_type: String,
    pub request_count: i64,
    pub token_count: i64,
    pub error_count: i64,
}

impl InstanceStatsSummary {
    /// 获取实例的统计汇总
    pub async fn get_by_instance(
        pool: &Pool<Sqlite>,
        instance_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Self, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT
                agent_type,
                SUM(request_count) as request_count,
                SUM(token_count) as token_count,
                SUM(error_count) as error_count
            FROM instance_usage_stats
            WHERE instance_id = ?
            "#,
        );

        if start_date.is_some() {
            query.push_str(" AND date >= ?");
        }
        if end_date.is_some() {
            query.push_str(" AND date <= ?");
        }
        query.push_str(" GROUP BY agent_type ORDER BY agent_type");

        #[derive(FromRow)]
        struct AgentStats {
            agent_type: String,
            request_count: i64,
            token_count: i64,
            error_count: i64,
        }

        let mut query_builder = sqlx::query_as::<_, AgentStats>(&query).bind(instance_id);

        if let Some(start) = start_date {
            query_builder = query_builder.bind(start);
        }
        if let Some(end) = end_date {
            query_builder = query_builder.bind(end);
        }

        let agent_stats: Vec<AgentStats> = query_builder.fetch_all(pool).await?;

        let stats_by_agent: Vec<AgentStatsSummary> = agent_stats
            .into_iter()
            .map(|s| AgentStatsSummary {
                agent_type: s.agent_type,
                request_count: s.request_count,
                token_count: s.token_count,
                error_count: s.error_count,
            })
            .collect();

        let total_requests: i64 = stats_by_agent.iter().map(|s| s.request_count).sum();
        let total_tokens: i64 = stats_by_agent.iter().map(|s| s.token_count).sum();
        let total_errors: i64 = stats_by_agent.iter().map(|s| s.error_count).sum();

        Ok(Self {
            instance_id: instance_id.to_string(),
            total_requests,
            total_tokens,
            total_errors,
            stats_by_agent,
        })
    }
}
