//! Kubernetes Audit Logging helpers
//!
//! Утилиты для логирования Kubernetes операций в Audit Log

use crate::api::state::AppState;
use crate::db::store::AuditLogManager;
use crate::models::audit_log::{
    AuditAction, AuditLevel, AuditLogFilter, AuditObjectType,
};
use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Helper для создания записей audit log для Kubernetes операций
pub struct KubernetesAuditLogger;

impl KubernetesAuditLogger {
    /// Логирование создания Kubernetes ресурса
    pub async fn log_resource_creation(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceCreated,
            resource_kind,
            resource_name,
            namespace,
            format!("Создан ресурс {resource_kind}/{resource_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование обновления Kubernetes ресурса
    pub async fn log_resource_update(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
        changes: Option<&str>,
    ) {
        let description = if let Some(changes_desc) = changes {
            format!("Обновлен ресурс {resource_kind}/{resource_name} в namespace {namespace}: {changes_desc}")
        } else {
            format!("Обновлен ресурс {resource_kind}/{resource_name} в namespace {namespace}")
        };

        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceUpdated,
            resource_kind,
            resource_name,
            namespace,
            description,
        )
        .await;
    }

    /// Логирование удаления Kubernetes ресурса
    pub async fn log_resource_deletion(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesResourceDeleted,
            resource_kind,
            resource_name,
            namespace,
            format!("Удален ресурс {resource_kind}/{resource_name} из namespace {namespace}"),
        )
        .await;
    }

    /// Логирование установки Helm release
    pub async fn log_helm_install(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        chart_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseInstalled,
            "HelmRelease",
            release_name,
            namespace,
            format!("Установлен Helm chart {chart_name} как release {release_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование обновления Helm release
    pub async fn log_helm_upgrade(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        chart_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseUpgraded,
            "HelmRelease",
            release_name,
            namespace,
            format!("Обновлен Helm release {release_name} до chart {chart_name} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование отката Helm release
    pub async fn log_helm_rollback(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        revision: i32,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseRolledBack,
            "HelmRelease",
            release_name,
            namespace,
            format!("Выполнен откат Helm release {release_name} к revision {revision} в namespace {namespace}"),
        )
        .await;
    }

    /// Логирование удаления Helm release
    pub async fn log_helm_uninstall(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        release_name: &str,
        namespace: &str,
    ) {
        Self::log(
            state,
            user_id,
            username,
            AuditAction::KubernetesHelmReleaseUninstalled,
            "HelmRelease",
            release_name,
            namespace,
            format!("Удален Helm release {release_name} из namespace {namespace}"),
        )
        .await;
    }

    /// Базовый метод логирования
    #[allow(clippy::too_many_arguments)]
    async fn log(
        state: &Arc<AppState>,
        user_id: Option<i64>,
        username: Option<String>,
        action: AuditAction,
        resource_kind: &str,
        resource_name: &str,
        namespace: &str,
        description: String,
    ) {
        let object_name = format!("{}/{}", resource_kind, resource_name);

        let _ = state
            .store
            .create_audit_log(
                None, // project_id не применим для Kubernetes
                user_id,
                username,
                &action,
                &AuditObjectType::Kubernetes,
                None, // object_id
                Some(object_name),
                description,
                &AuditLevel::Info,
                None, // ip_address
                None, // user_agent
                Some(serde_json::json!({
                    "resource_kind": resource_kind,
                    "resource_name": resource_name,
                    "namespace": namespace
                })),
            )
            .await;
    }
}

#[derive(Debug, Deserialize)]
pub struct KubernetesAuditQuery {
    pub username: Option<String>,
    pub resource: Option<String>,
    pub verb: Option<String>,
    pub namespace: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct KubernetesAuditExportQuery {
    pub format: Option<String>, // json | csv
    pub username: Option<String>,
    pub resource: Option<String>,
    pub verb: Option<String>,
    pub namespace: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KubernetesAuditRow {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub username: Option<String>,
    pub cluster: Option<String>,
    pub namespace: Option<String>,
    pub resource: Option<String>,
    pub resource_name: Option<String>,
    pub verb: String,
    pub action: String,
    pub description: String,
    pub level: String,
}

fn action_to_verb(action: &AuditAction) -> String {
    match action {
        AuditAction::KubernetesResourceCreated | AuditAction::KubernetesHelmReleaseInstalled => {
            "create".to_string()
        }
        AuditAction::KubernetesResourceUpdated
        | AuditAction::KubernetesResourceScaled
        | AuditAction::KubernetesHelmReleaseUpgraded
        | AuditAction::KubernetesHelmReleaseRolledBack => "update".to_string(),
        AuditAction::KubernetesResourceDeleted | AuditAction::KubernetesHelmReleaseUninstalled => {
            "delete".to_string()
        }
        _ => "other".to_string(),
    }
}

fn level_to_str(level: &AuditLevel) -> String {
    match level {
        AuditLevel::Info => "info".to_string(),
        AuditLevel::Warning => "warning".to_string(),
        AuditLevel::Error => "error".to_string(),
        AuditLevel::Critical => "critical".to_string(),
    }
}

fn extract_meta(details: &Option<serde_json::Value>) -> (Option<String>, Option<String>, Option<String>) {
    let meta = details
        .as_ref()
        .and_then(|d| d.get("metadata"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let resource = meta
        .get("resource_kind")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let resource_name = meta
        .get("resource_name")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let namespace = meta
        .get("namespace")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    (resource, resource_name, namespace)
}

fn map_rows(
    logs: &[crate::models::audit_log::AuditLog],
    cluster: Option<String>,
) -> Vec<KubernetesAuditRow> {
    logs.iter()
        .map(|r| {
            let (resource, resource_name, namespace) = extract_meta(&r.details);
            KubernetesAuditRow {
                id: r.id,
                created: r.created,
                username: r.username.clone(),
                cluster: cluster.clone(),
                namespace,
                resource,
                resource_name,
                verb: action_to_verb(&r.action),
                action: r.action.to_string(),
                description: r.description.clone(),
                level: level_to_str(&r.level),
            }
        })
        .collect()
}

fn apply_filters(rows: Vec<KubernetesAuditRow>, q: &KubernetesAuditQuery) -> Vec<KubernetesAuditRow> {
    let mut out = rows;
    if let Some(resource) = &q.resource {
        let r = resource.to_lowercase();
        out.retain(|x| x.resource.clone().unwrap_or_default().to_lowercase().contains(&r));
    }
    if let Some(verb) = &q.verb {
        let v = verb.to_lowercase();
        out.retain(|x| x.verb.to_lowercase() == v);
    }
    if let Some(ns) = &q.namespace {
        let n = ns.to_lowercase();
        out.retain(|x| x.namespace.clone().unwrap_or_default().to_lowercase() == n);
    }
    if let Some(search) = &q.search {
        let s = search.to_lowercase();
        out.retain(|x| {
            x.description.to_lowercase().contains(&s)
                || x.resource.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.resource_name.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.namespace.clone().unwrap_or_default().to_lowercase().contains(&s)
                || x.username.clone().unwrap_or_default().to_lowercase().contains(&s)
        });
    }
    out
}

/// GET /api/kubernetes/audit
pub async fn list_kubernetes_audit(
    State(state): State<Arc<AppState>>,
    Query(query): Query<KubernetesAuditQuery>,
) -> crate::error::Result<Json<Vec<KubernetesAuditRow>>> {
    let filter = AuditLogFilter {
        username: query.username.clone(),
        object_type: Some(AuditObjectType::Kubernetes),
        search: query.search.clone(),
        limit: query.limit.unwrap_or(200),
        offset: query.offset.unwrap_or(0),
        sort: "created".to_string(),
        order: "desc".to_string(),
        ..Default::default()
    };

    let result = state
        .store
        .search_audit_logs(&filter)
        .await
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    let cluster = state
        .config
        .kubernetes
        .as_ref()
        .and_then(|k| k.context.clone());
    let rows = map_rows(&result.records, cluster);
    Ok(Json(apply_filters(rows, &query)))
}

/// GET /api/kubernetes/audit/export?format=csv|json
pub async fn export_kubernetes_audit(
    State(state): State<Arc<AppState>>,
    Query(query): Query<KubernetesAuditExportQuery>,
) -> crate::error::Result<impl IntoResponse> {
    let list_query = KubernetesAuditQuery {
        username: query.username.clone(),
        resource: query.resource.clone(),
        verb: query.verb.clone(),
        namespace: query.namespace.clone(),
        search: query.search.clone(),
        limit: query.limit,
        offset: query.offset,
    };
    let Json(rows) = list_kubernetes_audit(State(state), Query(list_query)).await?;

    let format = query.format.unwrap_or_else(|| "json".to_string()).to_lowercase();
    if format == "csv" {
        let mut csv = String::from("id,created,username,cluster,namespace,resource,resource_name,verb,action,level,description\n");
        for r in rows {
            let line = format!(
                "{},{},{},{},{},{},{},{},{},{},\"{}\"\n",
                r.id,
                r.created.to_rfc3339(),
                r.username.unwrap_or_default().replace(',', " "),
                r.cluster.unwrap_or_default().replace(',', " "),
                r.namespace.unwrap_or_default().replace(',', " "),
                r.resource.unwrap_or_default().replace(',', " "),
                r.resource_name.unwrap_or_default().replace(',', " "),
                r.verb,
                r.action,
                r.level,
                r.description.replace('"', "'").replace('\n', " "),
            );
            csv.push_str(&line);
        }
        Ok((
            [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            csv,
        )
            .into_response())
    } else {
        Ok(Json(rows).into_response())
    }
}
