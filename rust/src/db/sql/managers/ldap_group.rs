//! LDAP Group Mapping SQL Manager

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::LdapGroupMappingManager;
use crate::error::{Error, Result};
use crate::models::ldap_group::{LdapGroupMapping, LdapGroupMappingCreate};
use async_trait::async_trait;

#[async_trait]
impl LdapGroupMappingManager for SqlStore {
    async fn get_ldap_group_mappings(&self) -> Result<Vec<LdapGroupMapping>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, LdapGroupMapping>(
                    r#"SELECT lgm.id, lgm.ldap_group_dn, lgm.project_id, lgm.role, lgm.created_at,
                              COALESCE(p.name,'') AS project_name
                       FROM ldap_group_mapping lgm
                       LEFT JOIN project p ON p.id = lgm.project_id
                       ORDER BY lgm.id"#
                )
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, LdapGroupMapping>(
                    r#"SELECT lgm.id, lgm.ldap_group_dn, lgm.project_id, lgm.role, lgm.created_at::text,
                              COALESCE(p.name,'') AS project_name
                       FROM ldap_group_mapping lgm
                       LEFT JOIN project p ON p.id = lgm.project_id
                       ORDER BY lgm.id"#
                )
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, LdapGroupMapping>(
                    r#"SELECT lgm.id, lgm.ldap_group_dn, lgm.project_id, lgm.role, lgm.created_at,
                              COALESCE(p.name,'') AS project_name
                       FROM ldap_group_mapping lgm
                       LEFT JOIN project p ON p.id = lgm.project_id
                       ORDER BY lgm.id"#
                )
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn create_ldap_group_mapping(&self, payload: LdapGroupMappingCreate) -> Result<LdapGroupMapping> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, LdapGroupMapping>(
                    r#"INSERT INTO ldap_group_mapping (ldap_group_dn, project_id, role)
                       VALUES (?, ?, ?)
                       RETURNING id, ldap_group_dn, project_id, role, created_at, '' AS project_name"#
                )
                .bind(&payload.ldap_group_dn)
                .bind(payload.project_id)
                .bind(&payload.role)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, LdapGroupMapping>(
                    r#"INSERT INTO ldap_group_mapping (ldap_group_dn, project_id, role)
                       VALUES ($1, $2, $3)
                       RETURNING id, ldap_group_dn, project_id, role, created_at::text, '' AS project_name"#
                )
                .bind(&payload.ldap_group_dn)
                .bind(payload.project_id)
                .bind(&payload.role)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "INSERT INTO ldap_group_mapping (ldap_group_dn, project_id, role) VALUES (?, ?, ?)"
                )
                .bind(&payload.ldap_group_dn)
                .bind(payload.project_id)
                .bind(&payload.role)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                let row = sqlx::query_as::<_, LdapGroupMapping>(
                    "SELECT id, ldap_group_dn, project_id, role, created_at, '' AS project_name FROM ldap_group_mapping WHERE id = LAST_INSERT_ID()"
                )
                .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn delete_ldap_group_mapping(&self, id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM ldap_group_mapping WHERE id = ?")
                    .bind(id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM ldap_group_mapping WHERE id = $1")
                    .bind(id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM ldap_group_mapping WHERE id = ?")
                    .bind(id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn get_mappings_for_groups(&self, group_dns: &[String]) -> Result<Vec<LdapGroupMapping>> {
        if group_dns.is_empty() {
            return Ok(vec![]);
        }
        let all = self.get_ldap_group_mappings().await?;
        Ok(all.into_iter().filter(|m| group_dns.contains(&m.ldap_group_dn)).collect())
    }
}
