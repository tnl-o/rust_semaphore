//! Handlers для Playbooks API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::{Playbook, PlaybookCreate, PlaybookUpdate};
use crate::error::Error;

/// GET /api/project/{project_id}/playbooks
pub async fn get_playbooks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Playbook>>, (StatusCode, Json<Error>)> {
    let playbooks = state.store.get_playbooks(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Error::Database(e.to_string()))
        ))?;
    Ok(Json(playbooks))
}

/// GET /api/project/{project_id}/playbooks/{id}
pub async fn get_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<Playbook>, (StatusCode, Json<Error>)> {
    let playbook = state.store.get_playbook(id, project_id)
        .await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(Error::Database(e.to_string()))
        ))?;
    Ok(Json(playbook))
}

/// POST /api/project/{project_id}/playbooks
pub async fn create_playbook(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<PlaybookCreate>,
) -> Result<Json<Playbook>, (StatusCode, Json<Error>)> {
    let playbook = state.store.create_playbook(project_id, payload)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Error::Database(e.to_string()))
        ))?;
    Ok(Json(playbook))
}

/// PUT /api/project/{project_id}/playbooks/{id}
pub async fn update_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<PlaybookUpdate>,
) -> Result<Json<Playbook>, (StatusCode, Json<Error>)> {
    let playbook = state.store.update_playbook(id, project_id, payload)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Error::Database(e.to_string()))
        ))?;
    Ok(Json(playbook))
}

/// DELETE /api/project/{project_id}/playbooks/{id}
pub async fn delete_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<Error>)> {
    state.store.delete_playbook(id, project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Error::Database(e.to_string()))
        ))?;
    Ok(StatusCode::NO_CONTENT)
}
