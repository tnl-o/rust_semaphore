//! Environment CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::environment, postgres::environment, mysql::environment

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Environment;

impl SqlDb {
    /// Получает окружения проекта
    pub async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::environment::get_environments(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::environment::get_environments(pool, project_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::environment::get_environments(pool, project_id).await
            }
        }
    }

    /// Получает окружение по ID
    pub async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::environment::get_environment(pool, project_id, environment_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::environment::get_environment(pool, project_id, environment_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::environment::get_environment(pool, project_id, environment_id).await
            }
        }
    }

    /// Создаёт окружение
    pub async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::environment::create_environment(pool, environment).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::environment::create_environment(pool, environment).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::environment::create_environment(pool, environment).await
            }
        }
    }

    /// Обновляет окружение
    pub async fn update_environment(&self, environment: Environment) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::environment::update_environment(pool, environment).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::environment::update_environment(pool, environment).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::environment::update_environment(pool, environment).await
            }
        }
    }

    /// Удаляет окружение
    pub async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?;
                crate::db::sql::sqlite::environment::delete_environment(pool, project_id, environment_id).await
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::environment::delete_environment(pool, project_id, environment_id).await
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?;
                crate::db::sql::mysql::environment::delete_environment(pool, project_id, environment_id).await
            }
        }
    }
}
