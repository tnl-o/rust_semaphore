//! Модель SecretStorage - хранилище секретов

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип хранилища секретов
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecretStorageType {
    Local,
    Vault,
    Dvls,
}

impl std::fmt::Display for SecretStorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretStorageType::Local => write!(f, "local"),
            SecretStorageType::Vault => write!(f, "vault"),
            SecretStorageType::Dvls => write!(f, "dvls"),
        }
    }
}

impl std::str::FromStr for SecretStorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(SecretStorageType::Local),
            "vault" => Ok(SecretStorageType::Vault),
            "dvls" => Ok(SecretStorageType::Dvls),
            _ => Ok(SecretStorageType::Local),
        }
    }
}

/// Хранилище секретов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecretStorage {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название хранилища
    pub name: String,

    /// Тип хранилища
    pub r#type: SecretStorageType,

    /// Параметры (JSON)
    pub params: String,

    /// Только для чтения
    pub read_only: bool,
}

impl SecretStorage {
    /// Создаёт новое хранилище
    pub fn new(project_id: i32, name: String, storage_type: SecretStorageType, params: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            r#type: storage_type,
            params,
            read_only: false,
        }
    }
}
