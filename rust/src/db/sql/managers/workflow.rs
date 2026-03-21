//! WorkflowManager - управление Workflow DAG

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::WorkflowManager;
use crate::error::{Error, Result};
use crate::models::workflow::{
    Workflow, WorkflowCreate, WorkflowUpdate,
    WorkflowNode, WorkflowNodeCreate, WorkflowNodeUpdate,
    WorkflowEdge, WorkflowEdgeCreate,
    WorkflowRun,
};
use async_trait::async_trait;

#[async_trait]
impl WorkflowManager for SqlStore {
    // =========================================================================
    // Workflows
    // =========================================================================

    async fn get_workflows(&self, project_id: i32) -> Result<Vec<Workflow>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM workflow WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM workflow WHERE project_id = $1 ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM `workflow` WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn get_workflow(&self, id: i32, project_id: i32) -> Result<Workflow> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM workflow WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM workflow WHERE id = $1 AND project_id = $2"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let row = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM `workflow` WHERE id = ? AND project_id = ?"
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

    async fn create_workflow(&self, project_id: i32, payload: WorkflowCreate) -> Result<Workflow> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, Workflow>(
                    "INSERT INTO workflow (project_id, name, description, created, updated)
                     VALUES (?, ?, ?, datetime('now'), datetime('now')) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.description)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, Workflow>(
                    "INSERT INTO workflow (project_id, name, description, created, updated)
                     VALUES ($1, $2, $3, NOW(), NOW()) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.description)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `workflow` (project_id, name, description, created, updated)
                     VALUES (?, ?, ?, NOW(), NOW())"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.description)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM `workflow` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn update_workflow(&self, id: i32, project_id: i32, payload: WorkflowUpdate) -> Result<Workflow> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, Workflow>(
                    "UPDATE workflow SET name = ?, description = ?, updated = datetime('now')
                     WHERE id = ? AND project_id = ? RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.description)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, Workflow>(
                    "UPDATE workflow SET name = $1, description = $2, updated = NOW()
                     WHERE id = $3 AND project_id = $4 RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.description)
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
                    "UPDATE `workflow` SET name = ?, description = ?, updated = NOW()
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&payload.name)
                .bind(&payload.description)
                .bind(id)
                .bind(project_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let row = sqlx::query_as::<_, Workflow>(
                    "SELECT * FROM `workflow` WHERE id = ? AND project_id = ?"
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

    async fn delete_workflow(&self, id: i32, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM workflow WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM workflow WHERE id = $1 AND project_id = $2")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `workflow` WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    // =========================================================================
    // Workflow Nodes
    // =========================================================================

    async fn get_workflow_nodes(&self, workflow_id: i32) -> Result<Vec<WorkflowNode>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, WorkflowNode>(
                    "SELECT * FROM workflow_node WHERE workflow_id = ? ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, WorkflowNode>(
                    "SELECT * FROM workflow_node WHERE workflow_id = $1 ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, WorkflowNode>(
                    "SELECT * FROM `workflow_node` WHERE workflow_id = ? ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn create_workflow_node(&self, workflow_id: i32, payload: WorkflowNodeCreate) -> Result<WorkflowNode> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "INSERT INTO workflow_node (workflow_id, template_id, name, pos_x, pos_y)
                     VALUES (?, ?, ?, ?, ?) RETURNING *"
                )
                .bind(workflow_id)
                .bind(payload.template_id)
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "INSERT INTO workflow_node (workflow_id, template_id, name, pos_x, pos_y)
                     VALUES ($1, $2, $3, $4, $5) RETURNING *"
                )
                .bind(workflow_id)
                .bind(payload.template_id)
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `workflow_node` (workflow_id, template_id, name, pos_x, pos_y)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(workflow_id)
                .bind(payload.template_id)
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "SELECT * FROM `workflow_node` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn update_workflow_node(&self, id: i32, workflow_id: i32, payload: WorkflowNodeUpdate) -> Result<WorkflowNode> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "UPDATE workflow_node SET name = ?, pos_x = ?, pos_y = ?
                     WHERE id = ? AND workflow_id = ? RETURNING *"
                )
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .bind(id)
                .bind(workflow_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "UPDATE workflow_node SET name = $1, pos_x = $2, pos_y = $3
                     WHERE id = $4 AND workflow_id = $5 RETURNING *"
                )
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .bind(id)
                .bind(workflow_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                sqlx::query(
                    "UPDATE `workflow_node` SET name = ?, pos_x = ?, pos_y = ?
                     WHERE id = ? AND workflow_id = ?"
                )
                .bind(&payload.name)
                .bind(payload.pos_x)
                .bind(payload.pos_y)
                .bind(id)
                .bind(workflow_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let row = sqlx::query_as::<_, WorkflowNode>(
                    "SELECT * FROM `workflow_node` WHERE id = ? AND workflow_id = ?"
                )
                .bind(id)
                .bind(workflow_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn delete_workflow_node(&self, id: i32, workflow_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM workflow_node WHERE id = ? AND workflow_id = ?")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM workflow_node WHERE id = $1 AND workflow_id = $2")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `workflow_node` WHERE id = ? AND workflow_id = ?")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    // =========================================================================
    // Workflow Edges
    // =========================================================================

    async fn get_workflow_edges(&self, workflow_id: i32) -> Result<Vec<WorkflowEdge>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, WorkflowEdge>(
                    "SELECT * FROM workflow_edge WHERE workflow_id = ? ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, WorkflowEdge>(
                    "SELECT * FROM workflow_edge WHERE workflow_id = $1 ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, WorkflowEdge>(
                    "SELECT * FROM `workflow_edge` WHERE workflow_id = ? ORDER BY id"
                )
                .bind(workflow_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn create_workflow_edge(&self, workflow_id: i32, payload: WorkflowEdgeCreate) -> Result<WorkflowEdge> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, WorkflowEdge>(
                    "INSERT INTO workflow_edge (workflow_id, from_node_id, to_node_id, condition)
                     VALUES (?, ?, ?, ?) RETURNING *"
                )
                .bind(workflow_id)
                .bind(payload.from_node_id)
                .bind(payload.to_node_id)
                .bind(&payload.condition)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, WorkflowEdge>(
                    "INSERT INTO workflow_edge (workflow_id, from_node_id, to_node_id, condition)
                     VALUES ($1, $2, $3, $4) RETURNING *"
                )
                .bind(workflow_id)
                .bind(payload.from_node_id)
                .bind(payload.to_node_id)
                .bind(&payload.condition)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `workflow_edge` (workflow_id, from_node_id, to_node_id, condition)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(workflow_id)
                .bind(payload.from_node_id)
                .bind(payload.to_node_id)
                .bind(&payload.condition)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, WorkflowEdge>(
                    "SELECT * FROM `workflow_edge` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn delete_workflow_edge(&self, id: i32, workflow_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM workflow_edge WHERE id = ? AND workflow_id = ?")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM workflow_edge WHERE id = $1 AND workflow_id = $2")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `workflow_edge` WHERE id = ? AND workflow_id = ?")
                    .bind(id)
                    .bind(workflow_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    // =========================================================================
    // Workflow Runs
    // =========================================================================

    async fn get_workflow_runs(&self, workflow_id: i32, project_id: i32) -> Result<Vec<WorkflowRun>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let rows = sqlx::query_as::<_, WorkflowRun>(
                    "SELECT * FROM workflow_run WHERE workflow_id = ? AND project_id = ? ORDER BY created DESC"
                )
                .bind(workflow_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::PostgreSQL => {
                let rows = sqlx::query_as::<_, WorkflowRun>(
                    "SELECT * FROM workflow_run WHERE workflow_id = $1 AND project_id = $2 ORDER BY created DESC"
                )
                .bind(workflow_id)
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
            SqlDialect::MySQL => {
                let rows = sqlx::query_as::<_, WorkflowRun>(
                    "SELECT * FROM `workflow_run` WHERE workflow_id = ? AND project_id = ? ORDER BY created DESC"
                )
                .bind(workflow_id)
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(rows)
            }
        }
    }

    async fn create_workflow_run(&self, workflow_id: i32, project_id: i32) -> Result<WorkflowRun> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let row = sqlx::query_as::<_, WorkflowRun>(
                    "INSERT INTO workflow_run (workflow_id, project_id, status, created)
                     VALUES (?, ?, 'pending', datetime('now')) RETURNING *"
                )
                .bind(workflow_id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::PostgreSQL => {
                let row = sqlx::query_as::<_, WorkflowRun>(
                    "INSERT INTO workflow_run (workflow_id, project_id, status, created)
                     VALUES ($1, $2, 'pending', NOW()) RETURNING *"
                )
                .bind(workflow_id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO `workflow_run` (workflow_id, project_id, status, created)
                     VALUES (?, ?, 'pending', NOW())"
                )
                .bind(workflow_id)
                .bind(project_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let inserted_id = result.last_insert_id() as i32;
                let row = sqlx::query_as::<_, WorkflowRun>(
                    "SELECT * FROM `workflow_run` WHERE id = ?"
                )
                .bind(inserted_id)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
        }
    }

    async fn update_workflow_run_status(&self, id: i32, status: &str, message: Option<String>) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE workflow_run SET status = ?, message = ? WHERE id = ?"
                )
                .bind(status)
                .bind(&message)
                .bind(id)
                .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE workflow_run SET status = $1, message = $2 WHERE id = $3"
                )
                .bind(status)
                .bind(&message)
                .bind(id)
                .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE `workflow_run` SET status = ?, message = ? WHERE id = ?"
                )
                .bind(status)
                .bind(&message)
                .bind(id)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
        }
        Ok(())
    }
}
