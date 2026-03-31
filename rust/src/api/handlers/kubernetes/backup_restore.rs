//! Kubernetes backup/restore runbook (v1) + optional Velero read-only status.

use axum::{
    extract::{Query, State},
    Json,
};
use kube::{
    api::{Api, DynamicObject, ListParams},
    core::{ApiResource, GroupVersionKind},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct BackupRunbook {
    pub title: String,
    pub db_steps: Vec<String>,
    pub config_steps: Vec<String>,
    pub restore_steps: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VeleroStatus {
    pub installed: bool,
    pub backups_api: bool,
    pub restores_api: bool,
}

#[derive(Debug, Deserialize)]
pub struct VeleroQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&GroupVersionKind::gvk(group, version, kind), plural)
}

fn dyn_api(
    raw: kube::Client,
    namespace: Option<&str>,
    api_res: &ApiResource,
) -> Api<DynamicObject> {
    if let Some(ns) = namespace {
        Api::namespaced_with(raw, ns, api_res)
    } else {
        Api::all_with(raw, api_res)
    }
}

pub async fn get_backup_restore_runbook() -> Result<Json<BackupRunbook>> {
    Ok(Json(BackupRunbook {
        title: "Velum backup/restore runbook (v1)".to_string(),
        db_steps: vec![
            "Остановить write-heavy операции (или включить maintenance window).".to_string(),
            "Сделать дамп БД (PostgreSQL: pg_dump; SQLite: snapshot файла + WAL).".to_string(),
            "Проверить целостность дампа и размер артефактов.".to_string(),
            "Сохранить в защищённое хранилище с ротацией и retention.".to_string(),
        ],
        config_steps: vec![
            "Сохранить env/секреты и конфиги приложения (без утечки plaintext в логи).".to_string(),
            "Сохранить версии образов, миграций и commit SHA текущего деплоя.".to_string(),
            "Проверить, что backup включает критичные каталоги и external integrations.".to_string(),
        ],
        restore_steps: vec![
            "Поднять чистое окружение с теми же версиями приложения/схемы.".to_string(),
            "Восстановить БД из дампа и применить миграции при необходимости.".to_string(),
            "Восстановить конфиги/секреты и перезапустить сервисы.".to_string(),
            "Провести smoke-check: login, projects, templates, task run, audit entries.".to_string(),
        ],
        notes: vec![
            "Velero поддерживается только как optional read-only детект в v1.".to_string(),
            "Полный UI оркестрации backup/restore вне scope v1.".to_string(),
        ],
    }))
}

pub async fn get_velero_status(State(state): State<Arc<AppState>>) -> Result<Json<VeleroStatus>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw().clone();
    let lp = ListParams::default().limit(1);

    let backups: Api<DynamicObject> =
        Api::all_with(raw.clone(), &ar("velero.io", "v1", "Backup", "backups"));
    let restores: Api<DynamicObject> =
        Api::all_with(raw, &ar("velero.io", "v1", "Restore", "restores"));

    let backups_api = backups.list(&lp).await.is_ok();
    let restores_api = restores.list(&lp).await.is_ok();
    Ok(Json(VeleroStatus {
        installed: backups_api || restores_api,
        backups_api,
        restores_api,
    }))
}

pub async fn list_velero_backups(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VeleroQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("velero.io", "v1", "Backup", "backups");
    let api = dyn_api(client.raw().clone(), query.namespace.as_deref(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Velero Backup API not available: {e}")))?;
    Ok(Json(items.items.iter().map(|x| serde_json::json!(x)).collect()))
}

