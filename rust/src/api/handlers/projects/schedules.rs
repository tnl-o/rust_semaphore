//! Projects API - Schedules Handler
//!
//! Обработчики для расписаний в проектах

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::ScheduleManager;
use crate::error::{Error, Result};
use crate::models::Schedule;
use crate::services::scheduler::SchedulePool;
use chrono::DateTime;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn invalid_schedule(msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(
            ErrorResponse::new(msg.into())
                .with_code("INVALID_SCHEDULE"),
        ),
    )
}

/// Проверка run_at / cron до записи в БД (SEC-05).
fn validate_schedule_before_save(schedule: &Schedule) -> std::result::Result<(), String> {
    if schedule.cron_format.as_deref() == Some("run_at") {
        let run_at = schedule
            .run_at
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                "Для одноразового запуска укажите дату и время (run_at, RFC3339)".to_string()
            })?;
        DateTime::parse_from_rfc3339(run_at)
            .map_err(|e| format!("Некорректная дата run_at: {}", e))?;
        return Ok(());
    }

    let cron = schedule.cron.trim();
    if cron.is_empty() {
        return Err(
            "Укажите cron (5 полей, как в веб-интерфейсе) или включите одноразовый запуск".to_string(),
        );
    }

    SchedulePool::validate_cron_for_storage(cron).map_err(|e| e.to_string())
}

/// Получает расписания проекта
pub async fn get_project_schedules(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<Schedule>>, (StatusCode, Json<ErrorResponse>)> {
    let schedules = state.store.get_schedules(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(schedules))
}

/// Получает расписание по ID
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Schedule>, (StatusCode, Json<ErrorResponse>)> {
    let schedule = state
        .store
        .get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    Ok(Json(schedule))
}

/// Создаёт новое расписание
pub async fn add_schedule(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Schedule>,
) -> std::result::Result<(StatusCode, Json<Schedule>), (StatusCode, Json<ErrorResponse>)> {
    let mut schedule = payload;
    schedule.project_id = project_id;

    if let Err(msg) = validate_schedule_before_save(&schedule) {
        return Err(invalid_schedule(msg));
    }

    let created = state.store.create_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет расписание
pub async fn update_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
    Json(payload): Json<Schedule>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut schedule = payload;
    schedule.id = schedule_id;
    schedule.project_id = project_id;

    if let Err(msg) = validate_schedule_before_save(&schedule) {
        return Err(invalid_schedule(msg));
    }

    state.store.update_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удаляет расписание
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_schedule(project_id, schedule_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Переключает активность расписания
///
/// PUT /api/project/{project_id}/schedules/{id}/active
pub async fn toggle_schedule_active(
    State(state): State<Arc<AppState>>,
    Path((project_id, schedule_id)): Path<(i32, i32)>,
    Json(payload): Json<serde_json::Value>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let active = payload
        .get("active")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut schedule = state
        .store
        .get_schedule(project_id, schedule_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Schedule not found".to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    schedule.active = active;
    state.store.update_schedule(schedule).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Валидирует cron-выражение
///
/// POST /api/projects/{project_id}/schedules/validate
pub async fn validate_schedule_cron_format(
    Path(_project_id): Path<i32>,
    Json(payload): Json<ValidateCronPayload>,
) -> std::result::Result<Json<ValidateCronResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cron = payload.cron.trim();
    let result = if cron.is_empty() {
        Err("Пустое cron выражение".to_string())
    } else {
        SchedulePool::validate_cron_for_storage(cron).map_err(|e| e.to_string())
    };

    let response = ValidateCronResponse {
        valid: result.is_ok(),
        error: result.err(),
    };

 Ok(Json(response))
}

// ============================================================================
// Types
// ============================================================================

/// Payload для валидации cron
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateCronPayload {
    pub cron: String,
}

/// Response валидации cron
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateCronResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_garbage_cron() {
        let s = Schedule {
            cron: "not a cron".to_string(),
            cron_format: None,
            run_at: None,
            ..minimal_sample_schedule()
        };
        assert!(validate_schedule_before_save(&s).is_err());
    }

    #[test]
    fn validate_accepts_five_field_cron() {
        let s = Schedule {
            cron: "0 9 * * *".to_string(),
            cron_format: None,
            run_at: None,
            ..minimal_sample_schedule()
        };
        assert!(validate_schedule_before_save(&s).is_ok());
    }

    #[test]
    fn validate_run_at_mode_requires_run_at() {
        let s = Schedule {
            cron: String::new(),
            cron_format: Some("run_at".to_string()),
            run_at: None,
            ..minimal_sample_schedule()
        };
        assert!(validate_schedule_before_save(&s).is_err());
    }

    #[test]
    fn validate_run_at_mode_accepts_rfc3339() {
        let s = Schedule {
            cron: String::new(),
            cron_format: Some("run_at".to_string()),
            run_at: Some("2030-01-02T15:00:00Z".to_string()),
            ..minimal_sample_schedule()
        };
        assert!(validate_schedule_before_save(&s).is_ok());
    }

    fn minimal_sample_schedule() -> Schedule {
        Schedule {
            id: 0,
            template_id: 1,
            project_id: 1,
            cron: String::new(),
            cron_format: None,
            name: "t".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        }
    }
}
