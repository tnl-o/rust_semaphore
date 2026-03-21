//! NotificationPolicyManager - управление политиками уведомлений

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::NotificationPolicyManager;
use crate::error::{Error, Result};
use crate::models::notification::{
    NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};
use async_trait::async_trait;

#[async_trait]
impl NotificationPolicyManager for SqlStore {
    async fn get_notification_policies(&self, project_id: i32) -> Result<Vec<NotificationPolicy>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy WHERE project_id = $1 ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM `notification_policy` WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn get_notification_policy(&self, id: i32, project_id: i32) -> Result<NotificationPolicy> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy WHERE id = $1 AND project_id = $2"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM `notification_policy` WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn create_notification_policy(&self, project_id: i32, payload: NotificationPolicyCreate) -> Result<NotificationPolicy> {
        let enabled = payload.enabled.unwrap_or(true);
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "INSERT INTO notification_policy (project_id, name, channel_type, webhook_url, trigger, template_id, enabled, created)
                     VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now')) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(enabled)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "INSERT INTO notification_policy (project_id, name, channel_type, webhook_url, trigger, template_id, enabled, created)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(enabled)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `notification_policy` (project_id, name, channel_type, webhook_url, `trigger`, template_id, enabled, created)
                     VALUES (?, ?, ?, ?, ?, ?, ?, NOW())"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(enabled)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM `notification_policy` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn update_notification_policy(&self, id: i32, project_id: i32, payload: NotificationPolicyUpdate) -> Result<NotificationPolicy> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "UPDATE notification_policy SET name = ?, channel_type = ?, webhook_url = ?, trigger = ?, template_id = ?, enabled = ?
                     WHERE id = ? AND project_id = ? RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(payload.enabled)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "UPDATE notification_policy SET name = $1, channel_type = $2, webhook_url = $3, trigger = $4, template_id = $5, enabled = $6
                     WHERE id = $7 AND project_id = $8 RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(payload.enabled)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                sqlx::query(
                    "UPDATE `notification_policy` SET name = ?, channel_type = ?, webhook_url = ?, `trigger` = ?, template_id = ?, enabled = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&payload.name)
                .bind(&payload.channel_type)
                .bind(&payload.webhook_url)
                .bind(&payload.trigger)
                .bind(payload.template_id)
                .bind(payload.enabled)
                .bind(id)
                .bind(project_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let row = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM `notification_policy` WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn delete_notification_policy(&self, id: i32, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM notification_policy WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM notification_policy WHERE id = $1 AND project_id = $2")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `notification_policy` WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn get_matching_policies(&self, project_id: i32, trigger: &str, template_id: Option<i32>) -> Result<Vec<NotificationPolicy>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy
                     WHERE project_id = ? AND enabled = 1
                       AND (trigger = ? OR trigger = 'always')
                       AND (template_id IS NULL OR template_id = ?)"
                )
                .bind(project_id)
                .bind(trigger)
                .bind(template_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM notification_policy
                     WHERE project_id = $1 AND enabled = TRUE
                       AND (trigger = $2 OR trigger = 'always')
                       AND (template_id IS NULL OR template_id = $3)"
                )
                .bind(project_id)
                .bind(trigger)
                .bind(template_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, NotificationPolicy>(
                    "SELECT * FROM `notification_policy`
                     WHERE project_id = ? AND enabled = 1
                       AND (`trigger` = ? OR `trigger` = 'always')
                       AND (template_id IS NULL OR template_id = ?)"
                )
                .bind(project_id)
                .bind(trigger)
                .bind(template_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }
}
