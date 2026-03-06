//! Inventory CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::inventory, postgres::inventory, mysql::inventory

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Inventory;

impl SqlDb {
    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::inventory::get_inventories(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::get_inventories(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::inventory::get_inventories(pool, project_id).await
            }
        }
    }

    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::inventory::get_inventory(pool, project_id, inventory_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::get_inventory(pool, project_id, inventory_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::inventory::get_inventory(pool, project_id, inventory_id).await
            }
        }
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::inventory::create_inventory(pool, inventory).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::create_inventory(pool, inventory).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::inventory::create_inventory(pool, inventory).await
            }
        }
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::inventory::update_inventory(pool, inventory).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::update_inventory(pool, inventory).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::inventory::update_inventory(pool, inventory).await
            }
        }
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::inventory::delete_inventory(pool, project_id, inventory_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::inventory::delete_inventory(pool, project_id, inventory_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::inventory::delete_inventory(pool, project_id, inventory_id).await
            }
        }
    }
}
