//! Webhook SQL реализация

use crate::error::{Error, Result};
use crate::models::webhook::{Webhook, UpdateWebhook, WebhookLog, WebhookType};
use super::SqlDb;
use chrono::Utc;
use serde_json::json;

fn webhook_type_to_string(t: &WebhookType) -> String {
    match t {
        WebhookType::Generic => "generic".to_string(),
        WebhookType::Slack => "slack".to_string(),
        WebhookType::Teams => "teams".to_string(),
        WebhookType::Discord => "discord".to_string(),
        WebhookType::Telegram => "telegram".to_string(),
        WebhookType::Custom => "custom".to_string(),
    }
}

fn string_to_webhook_type(s: &str) -> WebhookType {
    match s {
        "slack" => WebhookType::Slack,
        "teams" => WebhookType::Teams,
        "discord" => WebhookType::Discord,
        "telegram" => WebhookType::Telegram,
        "custom" => WebhookType::Custom,
        _ => WebhookType::Generic,
    }
}

impl SqlDb {
    /// Получает webhook по ID
    pub async fn get_webhook(&self, webhook_id: i64) -> Result<Webhook> {
        match self.get_dialect() {
            super::SqlDialect::SQLite => {
                let row = sqlx::query("SELECT * FROM webhook WHERE id = ?")
                    .bind(webhook_id)
                    .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                row.map(|r| self.row_to_webhook(r)).ok_or_else(|| Error::NotFound(format!("Webhook {} not found", webhook_id)))
            }
            super::SqlDialect::PostgreSQL => {
                let row = sqlx::query("SELECT * FROM webhook WHERE id = $1")
                    .bind(webhook_id)
                    .fetch_optional(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                row.map(|r| self.row_to_webhook(r)).ok_or_else(|| Error::NotFound(format!("Webhook {} not found", webhook_id)))
            }
            super::SqlDialect::MySQL => {
                let row = sqlx::query("SELECT * FROM webhook WHERE id = ?")
                    .bind(webhook_id)
                    .fetch_optional(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                row.map(|r| self.row_to_webhook(r)).ok_or_else(|| Error::NotFound(format!("Webhook {} not found", webhook_id)))
            }
        }
    }

    /// Получает webhook проекта
    pub async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<Webhook>> {
        let rows = match self.get_dialect() {
            super::SqlDialect::SQLite => {
                sqlx::query("SELECT * FROM webhook WHERE project_id = ? ORDER BY created DESC")
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
            super::SqlDialect::PostgreSQL => {
                sqlx::query("SELECT * FROM webhook WHERE project_id = $1 ORDER BY created DESC")
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
            super::SqlDialect::MySQL => {
                sqlx::query("SELECT * FROM webhook WHERE project_id = ? ORDER BY created DESC")
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
        };
        Ok(rows.into_iter().map(|r| self.row_to_webhook(r)).collect())
    }

    /// Создаёт webhook
    pub async fn create_webhook(&self, mut webhook: Webhook) -> Result<Webhook> {
        let now = Utc::now();
        let type_str = webhook_type_to_string(&webhook.r#type);
        
        match self.get_dialect() {
            super::SqlDialect::SQLite => {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook (project_id, name, type, url, secret, headers, active, events, retry_count, timeout_secs, created, updated)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id"
                )
                .bind(webhook.project_id)
                .bind(&webhook.name)
                .bind(&type_str)
                .bind(&webhook.url)
                .bind(&webhook.secret)
                .bind(serde_json::to_value(&webhook.headers).unwrap_or_default())
                .bind(webhook.active)
                .bind(&webhook.events)
                .bind(webhook.retry_count)
                .bind(webhook.timeout_secs)
                .bind(now)
                .bind(now)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                webhook.id = id;
                webhook.created = now;
                webhook.updated = now;
                Ok(webhook)
            }
            super::SqlDialect::PostgreSQL => {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook (project_id, name, type, url, secret, headers, active, events, retry_count, timeout_secs, created, updated)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id"
                )
                .bind(webhook.project_id)
                .bind(&webhook.name)
                .bind(&type_str)
                .bind(&webhook.url)
                .bind(&webhook.secret)
                .bind(serde_json::to_value(&webhook.headers).unwrap_or_default())
                .bind(webhook.active)
                .bind(&webhook.events)
                .bind(webhook.retry_count)
                .bind(webhook.timeout_secs)
                .bind(now)
                .bind(now)
                .fetch_one(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                webhook.id = id;
                webhook.created = now;
                webhook.updated = now;
                Ok(webhook)
            }
            super::SqlDialect::MySQL => {
                let result = sqlx::query(
                    "INSERT INTO webhook (project_id, name, type, url, secret, headers, active, events, retry_count, timeout_secs, created, updated)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(webhook.project_id)
                .bind(&webhook.name)
                .bind(&type_str)
                .bind(&webhook.url)
                .bind(&webhook.secret)
                .bind(serde_json::to_value(&webhook.headers).unwrap_or_default())
                .bind(webhook.active)
                .bind(&webhook.events)
                .bind(webhook.retry_count)
                .bind(webhook.timeout_secs)
                .bind(now)
                .bind(now)
                .execute(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                webhook.id = result.last_insert_id() as i64;
                webhook.created = now;
                webhook.updated = now;
                Ok(webhook)
            }
        }
    }

    /// Обновляет webhook
    pub async fn update_webhook(&self, webhook_id: i64, webhook: UpdateWebhook) -> Result<Webhook> {
        let now = Utc::now();
        let mut current = self.get_webhook(webhook_id).await?;

        if let Some(name) = webhook.name { current.name = name; }
        if let Some(r#type) = webhook.r#type { current.r#type = r#type; }
        if let Some(url) = webhook.url { current.url = url; }
        if let Some(secret) = webhook.secret { current.secret = Some(secret); }
        if let Some(headers) = webhook.headers { current.headers = Some(headers); }
        if let Some(active) = webhook.active { current.active = active; }
        if let Some(events) = webhook.events {
            current.events = serde_json::to_value(&events).unwrap_or_default();
        }
        if let Some(retry_count) = webhook.retry_count { current.retry_count = retry_count; }
        if let Some(timeout_secs) = webhook.timeout_secs { current.timeout_secs = timeout_secs; }
        current.updated = now;

        let type_str = webhook_type_to_string(&current.r#type);

        match self.get_dialect() {
            super::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE webhook SET name=?, type=?, url=?, secret=?, headers=?, active=?, events=?, retry_count=?, timeout_secs=?, updated=? WHERE id=?"
                )
                .bind(&current.name).bind(&type_str).bind(&current.url)
                .bind(&current.secret).bind(serde_json::to_value(&current.headers).unwrap_or_default())
                .bind(current.active).bind(&current.events)
                .bind(current.retry_count).bind(current.timeout_secs)
                .bind(now).bind(webhook_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            super::SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE webhook SET name=$1, type=$2, url=$3, secret=$4, headers=$5, active=$6, events=$7, retry_count=$8, timeout_secs=$9, updated=$10 WHERE id=$11"
                )
                .bind(&current.name).bind(&type_str).bind(&current.url)
                .bind(&current.secret).bind(serde_json::to_value(&current.headers).unwrap_or_default())
                .bind(current.active).bind(&current.events)
                .bind(current.retry_count).bind(current.timeout_secs)
                .bind(now).bind(webhook_id)
                .execute(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            super::SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE webhook SET name=?, type=?, url=?, secret=?, headers=?, active=?, events=?, retry_count=?, timeout_secs=?, updated=? WHERE id=?"
                )
                .bind(&current.name).bind(&type_str).bind(&current.url)
                .bind(&current.secret).bind(serde_json::to_value(&current.headers).unwrap_or_default())
                .bind(current.active).bind(&current.events)
                .bind(current.retry_count).bind(current.timeout_secs)
                .bind(now).bind(webhook_id)
                .execute(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(current)
    }

    /// Удаляет webhook
    pub async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        match self.get_dialect() {
            super::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM webhook WHERE id = ?")
                    .bind(webhook_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            super::SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM webhook WHERE id = $1")
                    .bind(webhook_id)
                    .execute(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            super::SqlDialect::MySQL => {
                sqlx::query("DELETE FROM webhook WHERE id = ?")
                    .bind(webhook_id)
                    .execute(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    /// Получает логи webhook
    pub async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<WebhookLog>> {
        let rows = match self.get_dialect() {
            super::SqlDialect::SQLite => {
                sqlx::query("SELECT * FROM webhook_log WHERE webhook_id = ? ORDER BY created DESC")
                    .bind(webhook_id)
                    .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
            super::SqlDialect::PostgreSQL => {
                sqlx::query("SELECT * FROM webhook_log WHERE webhook_id = $1 ORDER BY created DESC")
                    .bind(webhook_id)
                    .fetch_all(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
            super::SqlDialect::MySQL => {
                sqlx::query("SELECT * FROM webhook_log WHERE webhook_id = ? ORDER BY created DESC")
                    .bind(webhook_id)
                    .fetch_all(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?
            }
        };
        Ok(rows.into_iter().map(|r| self.row_to_webhook_log(r)).collect())
    }

    /// Создаёт лог webhook
    pub async fn create_webhook_log(&self, mut log: WebhookLog) -> Result<WebhookLog> {
        let now = Utc::now();
        
        match self.get_dialect() {
            super::SqlDialect::SQLite => {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook_log (webhook_id, event_type, status_code, success, error, attempts, payload, response, created)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id"
                )
                .bind(log.webhook_id)
                .bind(&log.event_type)
                .bind(&log.status_code)
                .bind(log.success)
                .bind(&log.error)
                .bind(log.attempts)
                .bind(serde_json::to_value(&log.payload).unwrap_or_default())
                .bind(serde_json::to_value(&log.response).unwrap_or_default())
                .bind(now)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                log.id = id;
                log.created = now;
                Ok(log)
            }
            super::SqlDialect::PostgreSQL => {
                let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook_log (webhook_id, event_type, status_code, success, error, attempts, payload, response, created)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
                )
                .bind(log.webhook_id)
                .bind(&log.event_type)
                .bind(&log.status_code)
                .bind(log.success)
                .bind(&log.error)
                .bind(log.attempts)
                .bind(serde_json::to_value(&log.payload).unwrap_or_default())
                .bind(serde_json::to_value(&log.response).unwrap_or_default())
                .bind(now)
                .fetch_one(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                log.id = id;
                log.created = now;
                Ok(log)
            }
            super::SqlDialect::MySQL => {
                let result = sqlx::query(
                    "INSERT INTO webhook_log (webhook_id, event_type, status_code, success, error, attempts, payload, response, created)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(log.webhook_id)
                .bind(&log.event_type)
                .bind(&log.status_code)
                .bind(log.success)
                .bind(&log.error)
                .bind(log.attempts)
                .bind(serde_json::to_value(&log.payload).unwrap_or_default())
                .bind(serde_json::to_value(&log.response).unwrap_or_default())
                .bind(now)
                .execute(self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                log.id = result.last_insert_id() as i64;
                log.created = now;
                Ok(log)
            }
        }
    }

    /// Вспомогательный метод для преобразования строки в Webhook
    fn row_to_webhook(&self, row: sqlx::Row) -> Webhook {
        use sqlx::{Row, Column};
        
        let columns: Vec<_> = row.columns().iter().map(|c| c.name()).collect();
        
        let get_opt_str = |name: &str| -> Option<String> {
            row.try_get::<String, _>(name).ok()
        };
        
        let get_str = |name: &str| -> String {
            get_opt_str(name).unwrap_or_default()
        };
        
        let get_opt_json = |name: &str| -> Option<serde_json::Value> {
            row.try_get::<serde_json::Value, _>(name).ok()
        };
        
        let get_json = |name: &str| -> serde_json::Value {
            get_opt_json(name).unwrap_or_default()
        };

        let get_opt_i64 = |name: &str| -> Option<i64> {
            row.try_get::<i64, _>(name).ok()
        };

        let get_opt_i32 = |name: &str| -> Option<i32> {
            row.try_get::<i32, _>(name).ok()
        };

        let get_opt_bool = |name: &str| -> Option<bool> {
            row.try_get::<bool, _>(name).ok()
        };

        let get_i64 = |name: &str| -> i64 {
            get_opt_i64(name).unwrap_or(0)
        };

        let get_i32 = |name: &str| -> i32 {
            get_opt_i32(name).unwrap_or(0)
        };

        let get_bool = |name: &str| -> bool {
            get_opt_bool(name).unwrap_or(false)
        };

        let type_str = get_str("type");
        let r#type = string_to_webhook_type(&type_str);

        Webhook {
            id: get_i64("id"),
            project_id: get_opt_i64("project_id"),
            name: get_str("name"),
            r#type,
            url: get_str("url"),
            secret: get_opt_str("secret"),
            headers: get_opt_json("headers"),
            active: get_bool("active"),
            events: get_json("events"),
            retry_count: get_i32("retry_count"),
            timeout_secs: get_i64("timeout_secs"),
            created: chrono::Utc::now(), // Упрощённо
            updated: chrono::Utc::now(),
        }
    }

    /// Вспомогательный метод для преобразования строки в WebhookLog
    fn row_to_webhook_log(&self, row: sqlx::Row) -> WebhookLog {
        let get_opt_str = |name: &str| -> Option<String> {
            row.try_get::<String, _>(name).ok()
        };
        
        let get_str = |name: &str| -> String {
            get_opt_str(name).unwrap_or_default()
        };
        
        let get_opt_json = |name: &str| -> Option<serde_json::Value> {
            row.try_get::<serde_json::Value, _>(name).ok()
        };

        let get_opt_i64 = |name: &str| -> Option<i64> {
            row.try_get::<i64, _>(name).ok()
        };

        let get_opt_i32 = |name: &str| -> Option<i32> {
            row.try_get::<i32, _>(name).ok()
        };

        let get_opt_bool = |name: &str| -> Option<bool> {
            row.try_get::<bool, _>(name).ok()
        };

        let get_i64 = |name: &str| -> i64 {
            get_opt_i64(name).unwrap_or(0)
        };

        let get_i32 = |name: &str| -> i32 {
            get_opt_i32(name).unwrap_or(0)
        };

        let get_bool = |name: &str| -> bool {
            get_opt_bool(name).unwrap_or(false)
        };

        let get_opt_i32_status = |name: &str| -> Option<i32> {
            row.try_get::<i32, _>(name).ok()
        };

        WebhookLog {
            id: get_i64("id"),
            webhook_id: get_i64("webhook_id"),
            event_type: get_str("event_type"),
            status_code: get_opt_i32_status("status_code"),
            success: get_bool("success"),
            error: get_opt_str("error"),
            attempts: get_i32("attempts"),
            payload: get_opt_json("payload"),
            response: get_opt_json("response"),
            created: chrono::Utc::now(),
        }
    }
}
