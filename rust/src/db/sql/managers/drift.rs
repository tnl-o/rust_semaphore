//! DriftManager - управление GitOps Drift Detection

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::DriftManager;
use crate::error::{Error, Result};
use crate::models::drift::{DriftConfig, DriftConfigCreate, DriftResult};
use async_trait::async_trait;

#[async_trait]
impl DriftManager for SqlStore {
    async fn get_drift_configs(&self, project_id: i32) -> Result<Vec<DriftConfig>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM drift_config WHERE project_id = ? ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM drift_config WHERE project_id = $1 ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM `drift_config` WHERE project_id = ? ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn get_drift_config(&self, id: i32, project_id: i32) -> Result<DriftConfig> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM drift_config WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM drift_config WHERE id = $1 AND project_id = $2"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let row = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM `drift_config` WHERE id = ? AND project_id = ?"
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

    async fn create_drift_config(&self, project_id: i32, payload: DriftConfigCreate) -> Result<DriftConfig> {
        let enabled = payload.enabled.unwrap_or(true);
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, DriftConfig>(
                    "INSERT INTO drift_config (project_id, template_id, enabled, schedule, created)
                     VALUES (?, ?, ?, ?, datetime('now')) RETURNING *"
                )
                .bind(project_id)
                .bind(payload.template_id)
                .bind(enabled)
                .bind(&payload.schedule)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, DriftConfig>(
                    "INSERT INTO drift_config (project_id, template_id, enabled, schedule, created)
                     VALUES ($1, $2, $3, $4, NOW()) RETURNING *"
                )
                .bind(project_id)
                .bind(payload.template_id)
                .bind(enabled)
                .bind(&payload.schedule)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `drift_config` (project_id, template_id, enabled, schedule, created)
                     VALUES (?, ?, ?, ?, NOW())"
                )
                .bind(project_id)
                .bind(payload.template_id)
                .bind(enabled)
                .bind(&payload.schedule)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, DriftConfig>(
                    "SELECT * FROM `drift_config` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn update_drift_config_enabled(&self, id: i32, project_id: i32, enabled: bool) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE drift_config SET enabled = ? WHERE id = ? AND project_id = ?"
                )
                .bind(enabled)
                .bind(id)
                .bind(project_id)
                .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE drift_config SET enabled = $1 WHERE id = $2 AND project_id = $3"
                )
                .bind(enabled)
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE `drift_config` SET enabled = ? WHERE id = ? AND project_id = ?"
                )
                .bind(enabled)
                .bind(id)
                .bind(project_id)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn delete_drift_config(&self, id: i32, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM drift_config WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM drift_config WHERE id = $1 AND project_id = $2")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `drift_config` WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn get_drift_results(&self, drift_config_id: i32, limit: i64) -> Result<Vec<DriftResult>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT * FROM drift_result WHERE drift_config_id = ? ORDER BY checked_at DESC LIMIT ?"
                )
                .bind(drift_config_id)
                .bind(limit)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT * FROM drift_result WHERE drift_config_id = $1 ORDER BY checked_at DESC LIMIT $2"
                )
                .bind(drift_config_id)
                .bind(limit)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT * FROM `drift_result` WHERE drift_config_id = ? ORDER BY checked_at DESC LIMIT ?"
                )
                .bind(drift_config_id)
                .bind(limit)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn create_drift_result(
        &self,
        project_id: i32,
        drift_config_id: i32,
        template_id: i32,
        status: &str,
        summary: Option<String>,
        task_id: Option<i32>,
    ) -> Result<DriftResult> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, DriftResult>(
                    "INSERT INTO drift_result (drift_config_id, project_id, template_id, status, summary, task_id, checked_at)
                     VALUES (?, ?, ?, ?, ?, ?, datetime('now')) RETURNING *"
                )
                .bind(drift_config_id)
                .bind(project_id)
                .bind(template_id)
                .bind(status)
                .bind(&summary)
                .bind(task_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, DriftResult>(
                    "INSERT INTO drift_result (drift_config_id, project_id, template_id, status, summary, task_id, checked_at)
                     VALUES ($1, $2, $3, $4, $5, $6, NOW()) RETURNING *"
                )
                .bind(drift_config_id)
                .bind(project_id)
                .bind(template_id)
                .bind(status)
                .bind(&summary)
                .bind(task_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `drift_result` (drift_config_id, project_id, template_id, status, summary, task_id, checked_at)
                     VALUES (?, ?, ?, ?, ?, ?, NOW())"
                )
                .bind(drift_config_id)
                .bind(project_id)
                .bind(template_id)
                .bind(status)
                .bind(&summary)
                .bind(task_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, DriftResult>(
                    "SELECT * FROM `drift_result` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn get_latest_drift_results(&self, project_id: i32) -> Result<Vec<DriftResult>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT dr.* FROM drift_result dr
                     INNER JOIN (
                         SELECT drift_config_id, MAX(checked_at) AS max_checked
                         FROM drift_result
                         WHERE project_id = ?
                         GROUP BY drift_config_id
                     ) latest ON dr.drift_config_id = latest.drift_config_id AND dr.checked_at = latest.max_checked
                     WHERE dr.project_id = ?
                     ORDER BY dr.checked_at DESC"
                )
                .bind(project_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT DISTINCT ON (drift_config_id) *
                     FROM drift_result
                     WHERE project_id = $1
                     ORDER BY drift_config_id, checked_at DESC"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, DriftResult>(
                    "SELECT dr.* FROM drift_result dr
                     INNER JOIN (
                         SELECT drift_config_id, MAX(checked_at) AS max_checked
                         FROM drift_result
                         WHERE project_id = ?
                         GROUP BY drift_config_id
                     ) latest ON dr.drift_config_id = latest.drift_config_id AND dr.checked_at = latest.max_checked
                     WHERE dr.project_id = ?
                     ORDER BY dr.checked_at DESC"
                )
                .bind(project_id)
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }
}
