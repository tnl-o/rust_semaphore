//! Модель API-токена

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// API-токен для аутентификации
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct APIToken {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub token: String,
    pub created: DateTime<Utc>,
    pub expired: bool,
}
