//! OIDC Authentication Handlers
//!
//! Обработчики для OIDC аутентификации

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::api::state::AppState;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::api::auth_local::LocalAuthService;

// ============================================================================
// API Handlers
// ============================================================================

/// GET /api/auth/oidc/{provider} - Redirect на OIDC провайдер
pub async fn oidc_login(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
) -> std::result::Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    // Находим провайдер в конфиге
    let _provider_config = state.config.auth.oidc_providers
        .iter()
        .find(|p| p.display_name.to_lowercase() == provider.to_lowercase())
        .ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(format!("OIDC provider '{}' not found", provider)))
        ))?;

    // TODO: Реализовать OIDC flow
    // 1. Создать OAuth2 client
    // 2. Сгенерировать state и PKCE challenge
    // 3. Получить authorization URL
    // 4. Редиректнуть пользователя

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse::new("OIDC login not fully implemented yet - see TODO in code".to_string()))
    ))
}

/// GET /api/auth/oidc/{provider}/callback - Callback от OIDC провайдера
pub async fn oidc_callback(
    State(_state): State<Arc<AppState>>,
    Path(_provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> std::result::Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем наличие code
    let _code = params.get("code")
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing code parameter".to_string()))
        ))?;

    // Проверяем наличие state (для защиты от CSRF)
    let _state_param = params.get("state")
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing state parameter".to_string()))
        ))?;

    // TODO: Реализовать обработку callback
    // 1. Обменять code на token
    // 2. Получить userinfo
    // 3. Найти или создать пользователя
    // 4. Сгенерировать JWT токен
    // 5. Редиректнуть на главную с токеном

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse::new("OIDC callback not fully implemented yet - see TODO in code".to_string()))
    ))
}

/// GET /api/auth/login - Metadata для login страницы
pub async fn get_login_metadata(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<LoginMetadataResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем OIDC провайдеры из конфига
    let oidc_providers: Vec<OidcProviderMetadata> = state.config.auth.oidc_providers
        .iter()
        .map(|p| OidcProviderMetadata {
            name: p.display_name.clone(),
            color: p.color.clone(),
            icon: p.icon.clone(),
            login_url: format!("/api/auth/oidc/{}", p.display_name.to_lowercase()),
        })
        .collect();

    Ok(Json(LoginMetadataResponse {
        oidc_providers,
        totp_enabled: state.config.auth.totp.enable,
        email_enabled: false, // TODO: добавить email в AuthConfig
        login_with_password: true, // Включаем форму username+password для локальной аутентификации
    }))
}

// ============================================================================
// Types
// ============================================================================

/// Metadata для OIDC провайдера
#[derive(Debug, Serialize, Deserialize)]
pub struct OidcProviderMetadata {
    pub name: String,
    pub color: String,
    pub icon: String,
    pub login_url: String,
}

/// Response для login metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginMetadataResponse {
    pub oidc_providers: Vec<OidcProviderMetadata>,
    pub totp_enabled: bool,
    pub email_enabled: bool,
    /// Когда true, Vue показывает форму username+password вместо magic link
    #[serde(rename = "login_with_password")]
    pub login_with_password: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_metadata_serialization() {
        let metadata = OidcProviderMetadata {
            name: "Google".to_string(),
            color: "#4285F4".to_string(),
            icon: "google".to_string(),
            login_url: "/api/auth/oidc/google".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("Google"));
        assert!(json.contains("#4285F4"));
    }

    #[test]
    fn test_login_metadata_response_serialization() {
        let response = LoginMetadataResponse {
            oidc_providers: vec![],
            totp_enabled: false,
            email_enabled: true,
            login_with_password: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("true"));
    }
}
