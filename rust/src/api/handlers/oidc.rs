//! OIDC Authentication Handlers
//!
//! Обработчики для OIDC аутентификации

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{AppendHeaders, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::auth_local::LocalAuthService;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::config::config_oidc::OidcProvider;
use crate::db::store::UserManager;
use crate::error::Error;
use oauth2::TokenResponse;

// ============================================================================
// API Handlers
// ============================================================================

/// Достаёт email из JSON userinfo: сначала провайдерский `email_claim`, затем стандартные поля.
pub(crate) fn extract_oidc_email_from_userinfo(
    userinfo: &serde_json::Value,
    provider: &OidcProvider,
) -> String {
    fn claim_str(v: &serde_json::Value, key: &str) -> Option<String> {
        v.get(key).and_then(|x| x.as_str().filter(|s| !s.is_empty()).map(str::to_string))
    }

    if !provider.email_claim.is_empty() {
        if let Some(s) = claim_str(userinfo, provider.email_claim.as_str()) {
            return s;
        }
    }
    for key in ["email", "preferred_username", "upn", "mail"] {
        if let Some(s) = claim_str(userinfo, key) {
            return s;
        }
    }
    String::new()
}

/// GET /api/auth/oidc/{provider} - Redirect на OIDC провайдер
pub async fn oidc_login(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
) -> std::result::Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    let provider_config = state
        .config
        .auth
        .oidc_providers
        .iter()
        .find(|p| p.display_name.to_lowercase() == provider.to_lowercase())
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "OIDC provider '{}' not found",
                    provider
                ))),
            )
        })?;

    if !provider_config.is_configured() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC provider not configured".to_string(),
            )),
        ));
    }

    let auth_url = if !provider_config.endpoint.auth_url.is_empty() {
        &provider_config.endpoint.auth_url
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC auth URL not configured".to_string(),
            )),
        ));
    };

    let token_url = if !provider_config.endpoint.token_url.is_empty() {
        &provider_config.endpoint.token_url
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC token URL not configured".to_string(),
            )),
        ));
    };

    let client =
        oauth2::basic::BasicClient::new(oauth2::ClientId::new(provider_config.client_id.clone()))
            .set_client_secret(oauth2::ClientSecret::new(
                provider_config.client_secret.clone(),
            ))
            .set_auth_uri(oauth2::AuthUrl::new(auth_url.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Invalid auth URL: {}", e))),
                )
            })?)
            .set_token_uri(oauth2::TokenUrl::new(token_url.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Invalid token URL: {}", e))),
                )
            })?)
            .set_redirect_uri(
                oauth2::RedirectUrl::new(provider_config.redirect_url.clone()).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid redirect URL: {}", e))),
                    )
                })?,
            );

    let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    let state_param = uuid::Uuid::new_v4().to_string();

    {
        let mut cache = state.oidc_state.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to lock OIDC state".to_string())),
            )
        })?;
        cache.insert(
            state_param.clone(),
            crate::api::state::OidcState {
                pkce_verifier: pkce_verifier.secret().to_string(),
                provider: provider.clone(),
            },
        );
    }

    let (auth_url, _) = client
        .authorize_url(|| oauth2::CsrfToken::new(state_param.clone()))
        .add_scopes(
            provider_config
                .scopes
                .iter()
                .map(|s| oauth2::Scope::new(s.clone())),
        )
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok(Redirect::temporary(auth_url.as_str()))
}

/// GET /api/auth/oidc/{provider}/callback - Callback от OIDC провайдера
pub async fn oidc_callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> std::result::Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Redirect,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    let code = params.get("code").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing code parameter".to_string())),
        )
    })?;

    let state_param = params.get("state").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing state parameter".to_string())),
        )
    })?;

    let oidc_state = {
        let mut cache = state.oidc_state.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to lock OIDC state".to_string())),
            )
        })?;
        cache.remove(state_param).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Invalid or expired state".to_string())),
            )
        })?
    };

    if oidc_state.provider != provider {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("State mismatch".to_string())),
        ));
    }

    let provider_config = state
        .config
        .auth
        .oidc_providers
        .iter()
        .find(|p| p.display_name.to_lowercase() == provider.to_lowercase())
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "OIDC provider '{}' not found",
                    provider
                ))),
            )
        })?;

    let client =
        oauth2::basic::BasicClient::new(oauth2::ClientId::new(provider_config.client_id.clone()))
            .set_client_secret(oauth2::ClientSecret::new(
                provider_config.client_secret.clone(),
            ))
            .set_auth_uri(oauth2::AuthUrl::new(provider_config.endpoint.auth_url.clone()).map_err(
                |e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid auth URL: {}", e))),
                    )
                },
            )?)
            .set_token_uri(oauth2::TokenUrl::new(provider_config.endpoint.token_url.clone()).map_err(
                |e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid token URL: {}", e))),
                    )
                },
            )?)
            .set_redirect_uri(
                oauth2::RedirectUrl::new(provider_config.redirect_url.clone()).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid redirect URL: {}", e))),
                    )
                })?,
            );

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "HTTP client build failed: {}",
                    e
                ))),
            )
        })?;

    let token_result = client
        .exchange_code(oauth2::AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(oauth2::PkceCodeVerifier::new(oidc_state.pkce_verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Token exchange failed: {}", e))),
            )
        })?;

    let access_token = token_result.access_token().secret();
    let userinfo_url = if !provider_config.endpoint.userinfo_url.is_empty() {
        provider_config.endpoint.userinfo_url.clone()
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC userinfo URL not configured".to_string(),
            )),
        ));
    };

    let client = reqwest::Client::new();
    let userinfo: serde_json::Value = client
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new(format!(
                    "Userinfo request failed: {}",
                    e
                ))),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new(format!("Userinfo parse failed: {}", e))),
            )
        })?;

    let email = extract_oidc_email_from_userinfo(&userinfo, provider_config);
    let username = userinfo
        .get("preferred_username")
        .or_else(|| userinfo.get("email"))
        .or_else(|| userinfo.get("sub"))
        .or_else(|| userinfo.get("name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| email.clone());
    let name = userinfo
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&username)
        .to_string();

    if email.is_empty() && username.is_empty() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::new(
                "OIDC userinfo missing email/username".to_string(),
            )),
        ));
    }

    let user = match state
        .store
        .get_user_by_login_or_email(&username, &email)
        .await
    {
        Ok(u) => u,
        Err(Error::NotFound(_)) => {
            let password_hash = crate::api::auth_local::hash_password(
                &uuid::Uuid::new_v4().to_string(),
            )
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!(
                        "Failed to hash password: {}",
                        e
                    ))),
                )
            })?;
            let new_user = crate::models::User {
                id: 0,
                created: chrono::Utc::now(),
                username: username.clone(),
                name: name.clone(),
                email: email.clone(),
                password: password_hash.clone(),
                admin: false,
                external: true,
                alert: false,
                pro: false,
                totp: None,
                email_otp: None,
            };
            state
                .store
                .create_user(new_user, &password_hash)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Failed to create user: {}", e))),
                    )
                })?
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ))
        }
    };

    let auth_service = LocalAuthService::new(state.store.clone());
    let token_info = auth_service.generate_token(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Token generation failed: {}",
                e
            ))),
        )
    })?;

    let base_url = std::env::var("SEMAPHORE_WEB_ROOT").unwrap_or_else(|_| "/".to_string());
    let redirect_url = format!(
        "{}?token={}",
        base_url.trim_end_matches('/'),
        token_info.token
    );

    let cookie_value = format!(
        "semaphore={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token_info.token, token_info.expires_in
    );

    let headers = AppendHeaders([(header::SET_COOKIE, cookie_value)]);

    Ok((headers, Redirect::temporary(&redirect_url)))
}

/// GET /api/auth/login - Metadata для login страницы
pub async fn get_login_metadata(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<LoginMetadataResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем OIDC провайдеры из конфига
    let oidc_providers: Vec<OidcProviderMetadata> = state
        .config
        .auth
        .oidc_providers
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
        email_enabled: state.config.auth.email_enabled,
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

    #[test]
    fn test_extract_oidc_email_default_chain() {
        let provider = OidcProvider::default();
        let v = serde_json::json!({
            "preferred_username": "u1",
            "sub": "oidc-sub"
        });
        assert_eq!(extract_oidc_email_from_userinfo(&v, &provider), "u1");

        let v2 = serde_json::json!({"upn": "user@contoso.com"});
        assert_eq!(
            extract_oidc_email_from_userinfo(&v2, &provider),
            "user@contoso.com"
        );
    }

    #[test]
    fn test_extract_oidc_email_custom_claim() {
        let provider = OidcProvider {
            email_claim: "unique_name".to_string(),
            ..Default::default()
        };
        let v = serde_json::json!({
            "unique_name": "azure@example.com",
            "email": "other@example.com"
        });
        assert_eq!(
            extract_oidc_email_from_userinfo(&v, &provider),
            "azure@example.com"
        );
    }
}
