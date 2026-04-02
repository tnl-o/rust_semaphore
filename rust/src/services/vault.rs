//! HashiCorp Vault Integration
//!
//! Клиент для HashiCorp Vault. Поддерживает:
//! - Аутентификация: Token и AppRole
//! - KV v2 secret engine: get/put/delete/list
//! - Dynamic secrets: database engine
//! - Lease management: renew / revoke / list

use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

// ============================================================================
// Config & Auth
// ============================================================================

/// Метод аутентификации в Vault
#[derive(Debug, Clone)]
pub enum VaultAuthMethod {
    /// Прямой токен (для dev/testing)
    Token(String),
    /// AppRole — для production (role_id + secret_id)
    AppRole {
        role_id: String,
        secret_id: String,
        mount_path: String,
    },
}

/// Конфигурация Vault клиента
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Адрес Vault (например https://vault.example.com:8200)
    pub address: String,
    /// Метод аутентификации
    pub auth: VaultAuthMethod,
    /// Namespace (Vault Enterprise)
    pub namespace: Option<String>,
    /// Таймаут HTTP запросов
    pub timeout_secs: u64,
    /// Mount path для KV v2 engine (по умолчанию "secret")
    pub kv_mount: String,
}

impl VaultConfig {
    /// Создаёт конфиг с Token-аутентификацией
    pub fn with_token(address: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            auth: VaultAuthMethod::Token(token.into()),
            namespace: None,
            timeout_secs: 10,
            kv_mount: "secret".to_string(),
        }
    }

    /// Создаёт конфиг с AppRole-аутентификацией
    pub fn with_approle(
        address: impl Into<String>,
        role_id: impl Into<String>,
        secret_id: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            auth: VaultAuthMethod::AppRole {
                role_id: role_id.into(),
                secret_id: secret_id.into(),
                mount_path: "approle".to_string(),
            },
            namespace: None,
            timeout_secs: 10,
            kv_mount: "secret".to_string(),
        }
    }
}

// ============================================================================
// Vault API response types (private)
// ============================================================================

#[derive(Deserialize)]
struct AppRoleLoginReq {
    role_id: String,
    secret_id: String,
}

#[derive(Deserialize)]
struct VaultAuthResponse {
    auth: VaultAuthInfo,
}

#[derive(Deserialize)]
struct VaultAuthInfo {
    client_token: String,
    lease_duration: u64,
    renewable: bool,
}

#[derive(Deserialize)]
struct KvReadResponse {
    data: KvSecretPayload,
}

#[derive(Deserialize)]
struct KvSecretPayload {
    data: HashMap<String, serde_json::Value>,
    metadata: KvSecretMetadata,
}

#[derive(Deserialize)]
struct KvSecretMetadata {
    created_time: String,
    destroyed: bool,
    version: u32,
}

#[derive(Deserialize)]
struct KvListResponse {
    data: KvListData,
}

#[derive(Deserialize)]
struct KvListData {
    keys: Vec<String>,
}

#[derive(Deserialize)]
struct DynamicSecretResponse {
    lease_id: String,
    lease_duration: u64,
    renewable: bool,
    data: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct LeaseLookupResponse {
    data: LeaseData,
}

#[derive(Deserialize)]
struct LeaseData {
    id: String,
    renewable: bool,
    ttl: u64,
}

#[derive(Deserialize)]
struct LeaseRenewResponse {
    lease_id: String,
    lease_duration: u64,
    renewable: bool,
}

// ============================================================================
// Public types
// ============================================================================

/// KV секрет (данные + метаданные)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSecret {
    /// Данные секрета
    pub data: HashMap<String, serde_json::Value>,
    /// Версия
    pub version: u32,
    /// Время создания (RFC3339)
    pub created_time: String,
    /// Секрет уничтожен
    pub destroyed: bool,
}

/// Динамический секрет (credentials с lease)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSecret {
    /// Vault lease ID для продления / отзыва
    pub lease_id: String,
    /// Время жизни lease в секундах
    pub lease_duration: u64,
    /// Можно ли продлить lease
    pub renewable: bool,
    /// Данные (например username/password для DB)
    pub data: HashMap<String, serde_json::Value>,
}

/// Информация о lease
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseInfo {
    pub lease_id: String,
    pub renewable: bool,
    pub ttl_secs: u64,
}

/// Результат продления lease
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRenewal {
    pub lease_id: String,
    pub lease_duration: u64,
    pub renewable: bool,
}

/// Ошибка Vault клиента
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Vault API error {status}: {message}")]
    Api { status: u16, message: String },
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Secret not found: {path}")]
    NotFound { path: String },
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type VaultResult<T> = Result<T, VaultError>;

// ============================================================================
// VaultClient
// ============================================================================

/// Клиент для работы с HashiCorp Vault
pub struct VaultClient {
    config: VaultConfig,
    http: Client,
    /// Кэшированный токен (обновляется при AppRole auth)
    token: RwLock<String>,
}

impl VaultClient {
    /// Создаёт и инициализирует клиент (выполняет аутентификацию)
    pub async fn new(config: VaultConfig) -> VaultResult<Self> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(false);

        if std::env::var("VAULT_SKIP_VERIFY").unwrap_or_default() == "true" {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let http = builder.build()?;

        let initial_token = match &config.auth {
            VaultAuthMethod::Token(t) => t.clone(),
            VaultAuthMethod::AppRole { .. } => String::new(), // will auth below
        };

        let client = Self {
            config,
            http,
            token: RwLock::new(initial_token),
        };

        // Authenticate if AppRole
        if matches!(client.config.auth, VaultAuthMethod::AppRole { .. }) {
            client.authenticate().await?;
        }

        Ok(client)
    }

    // -----------------------------------------------------------------------
    // Authentication
    // -----------------------------------------------------------------------

    /// Выполняет аутентификацию и сохраняет токен
    pub async fn authenticate(&self) -> VaultResult<()> {
        let token = match &self.config.auth {
            VaultAuthMethod::Token(t) => t.clone(),
            VaultAuthMethod::AppRole {
                role_id,
                secret_id,
                mount_path,
            } => {
                let url = format!(
                    "{}/v1/auth/{}/login",
                    self.config.address.trim_end_matches('/'),
                    mount_path
                );

                let body = serde_json::json!({
                    "role_id": role_id,
                    "secret_id": secret_id,
                });

                let resp = self
                    .vault_post_unauthenticated(&url, &body)
                    .await?;

                let auth: VaultAuthResponse = resp;
                info!(
                    "Vault AppRole auth OK, lease_duration={}s, renewable={}",
                    auth.auth.lease_duration, auth.auth.renewable
                );
                auth.auth.client_token
            }
        };

        let mut t = self.token.write().await;
        *t = token;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // KV v2 — get / put / delete / list
    // -----------------------------------------------------------------------

    /// Читает секрет из KV v2 engine
    ///
    /// `path` — путь относительно mount (например "myapp/db")
    pub async fn get_secret(&self, path: &str) -> VaultResult<VaultSecret> {
        let url = self.kv_data_url(path);
        debug!("Vault GET secret: {}", url);

        let resp: KvReadResponse = self.vault_get(&url).await?;

        Ok(VaultSecret {
            data: resp.data.data,
            version: resp.data.metadata.version,
            created_time: resp.data.metadata.created_time,
            destroyed: resp.data.metadata.destroyed,
        })
    }

    /// Записывает/обновляет секрет в KV v2
    ///
    /// Возвращает версию созданного секрета
    pub async fn put_secret(
        &self,
        path: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> VaultResult<u32> {
        let url = self.kv_data_url(path);
        debug!("Vault PUT secret: {}", url);

        let body = serde_json::json!({ "data": data });
        let resp: serde_json::Value = self.vault_post(&url, &body).await?;

        let version = resp["data"]["version"]
            .as_u64()
            .unwrap_or(1) as u32;
        info!("Vault secret written: path={} version={}", path, version);
        Ok(version)
    }

    /// Удаляет конкретные версии секрета из KV v2
    ///
    /// Передай пустой `versions` чтобы удалить последнюю версию.
    pub async fn delete_secret(&self, path: &str, versions: &[u32]) -> VaultResult<()> {
        if versions.is_empty() {
            // Soft-delete latest version via DELETE data endpoint
            let url = self.kv_data_url(path);
            debug!("Vault DELETE secret (latest): {}", url);
            self.vault_delete(&url).await?;
        } else {
            // Delete specific versions
            let url = self.kv_delete_url(path);
            debug!("Vault DELETE secret versions {:?}: {}", versions, url);
            let body = serde_json::json!({ "versions": versions });
            let _: serde_json::Value = self.vault_post(&url, &body).await?;
        }
        info!("Vault secret deleted: path={}", path);
        Ok(())
    }

    /// Перечисляет секреты по префиксу пути
    pub async fn list_secrets(&self, path: &str) -> VaultResult<Vec<String>> {
        let url = self.kv_metadata_url(path);
        debug!("Vault LIST secrets: {}", url);

        let resp: KvListResponse = self.vault_list(&url).await?;
        Ok(resp.data.keys)
    }

    // -----------------------------------------------------------------------
    // Dynamic secrets
    // -----------------------------------------------------------------------

    /// Генерирует динамические credentials через указанный engine
    ///
    /// Примеры `role_path`:
    /// - `"database/creds/my-role"` — DB credentials
    /// - `"aws/creds/my-role"` — AWS credentials
    pub async fn generate_dynamic_secret(&self, role_path: &str) -> VaultResult<DynamicSecret> {
        let url = format!(
            "{}/v1/{}",
            self.config.address.trim_end_matches('/'),
            role_path.trim_start_matches('/')
        );
        debug!("Vault generate dynamic secret: {}", url);

        let resp: DynamicSecretResponse = self.vault_get(&url).await?;

        info!(
            "Dynamic secret generated: lease_id={} ttl={}s",
            resp.lease_id, resp.lease_duration
        );
        Ok(DynamicSecret {
            lease_id: resp.lease_id,
            lease_duration: resp.lease_duration,
            renewable: resp.renewable,
            data: resp.data,
        })
    }

    // -----------------------------------------------------------------------
    // Lease management
    // -----------------------------------------------------------------------

    /// Возвращает информацию о lease
    pub async fn lookup_lease(&self, lease_id: &str) -> VaultResult<LeaseInfo> {
        let url = format!(
            "{}/v1/sys/leases/lookup",
            self.config.address.trim_end_matches('/')
        );
        let body = serde_json::json!({ "lease_id": lease_id });
        let resp: LeaseLookupResponse = self.vault_post(&url, &body).await?;

        Ok(LeaseInfo {
            lease_id: resp.data.id,
            renewable: resp.data.renewable,
            ttl_secs: resp.data.ttl,
        })
    }

    /// Продлевает lease
    ///
    /// `increment_secs` — запрошенное время продления (0 = по умолчанию)
    pub async fn renew_lease(
        &self,
        lease_id: &str,
        increment_secs: u64,
    ) -> VaultResult<LeaseRenewal> {
        let url = format!(
            "{}/v1/sys/leases/renew",
            self.config.address.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "lease_id": lease_id,
            "increment": increment_secs,
        });
        let resp: LeaseRenewResponse = self.vault_post(&url, &body).await?;

        info!(
            "Lease renewed: id={} duration={}s",
            resp.lease_id, resp.lease_duration
        );
        Ok(LeaseRenewal {
            lease_id: resp.lease_id,
            lease_duration: resp.lease_duration,
            renewable: resp.renewable,
        })
    }

    /// Отзывает (revoke) lease немедленно
    pub async fn revoke_lease(&self, lease_id: &str) -> VaultResult<()> {
        let url = format!(
            "{}/v1/sys/leases/revoke",
            self.config.address.trim_end_matches('/')
        );
        let body = serde_json::json!({ "lease_id": lease_id });
        let _: serde_json::Value = self.vault_post(&url, &body).await?;

        info!("Lease revoked: id={}", lease_id);
        Ok(())
    }

    /// Отзывает все leases по префиксу
    pub async fn revoke_lease_prefix(&self, prefix: &str) -> VaultResult<()> {
        let url = format!(
            "{}/v1/sys/leases/revoke-prefix/{}",
            self.config.address.trim_end_matches('/'),
            prefix.trim_start_matches('/')
        );
        self.vault_delete(&url).await?;
        info!("Leases revoked by prefix: {}", prefix);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Health check
    // -----------------------------------------------------------------------

    /// Проверяет доступность Vault
    pub async fn health_check(&self) -> VaultResult<VaultHealthStatus> {
        let url = format!(
            "{}/v1/sys/health",
            self.config.address.trim_end_matches('/')
        );

        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();

        // Vault health codes:
        // 200 = initialized, unsealed, active
        // 429 = unsealed, standby
        // 472 = DR secondary, active
        // 501 = not initialized
        // 503 = sealed
        Ok(match status {
            200 => VaultHealthStatus::Active,
            429 => VaultHealthStatus::Standby,
            472 => VaultHealthStatus::DrSecondary,
            501 => VaultHealthStatus::NotInitialized,
            503 => VaultHealthStatus::Sealed,
            _ => VaultHealthStatus::Unknown(status),
        })
    }

    // -----------------------------------------------------------------------
    // Private HTTP helpers
    // -----------------------------------------------------------------------

    async fn vault_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> VaultResult<T> {
        let token = self.token.read().await.clone();
        let mut req = self.http.get(url).header("X-Vault-Token", &token);

        if let Some(ns) = &self.config.namespace {
            req = req.header("X-Vault-Namespace", ns);
        }

        let resp = req.send().await?;
        Self::parse_response(resp, url).await
    }

    async fn vault_list<T: serde::de::DeserializeOwned>(&self, url: &str) -> VaultResult<T> {
        let token = self.token.read().await.clone();
        let mut req = self
            .http
            .request(reqwest::Method::from_bytes(b"LIST").unwrap(), url)
            .header("X-Vault-Token", &token);

        if let Some(ns) = &self.config.namespace {
            req = req.header("X-Vault-Namespace", ns);
        }

        let resp = req.send().await?;
        Self::parse_response(resp, url).await
    }

    async fn vault_post<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> VaultResult<T> {
        let token = self.token.read().await.clone();
        let mut req = self
            .http
            .post(url)
            .header("X-Vault-Token", &token)
            .json(body);

        if let Some(ns) = &self.config.namespace {
            req = req.header("X-Vault-Namespace", ns);
        }

        let resp = req.send().await?;
        Self::parse_response(resp, url).await
    }

    async fn vault_post_unauthenticated<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> VaultResult<T> {
        let resp = self.http.post(url).json(body).send().await?;
        Self::parse_response(resp, url).await
    }

    async fn vault_delete(&self, url: &str) -> VaultResult<()> {
        let token = self.token.read().await.clone();
        let mut req = self
            .http
            .delete(url)
            .header("X-Vault-Token", &token);

        if let Some(ns) = &self.config.namespace {
            req = req.header("X-Vault-Namespace", ns);
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        if status >= 400 {
            let text = resp.text().await.unwrap_or_default();
            return Err(VaultError::Api {
                status,
                message: text,
            });
        }
        Ok(())
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        url: &str,
    ) -> VaultResult<T> {
        let status = resp.status().as_u16();

        if status == 404 {
            let path = url.rsplit("/v1").last().unwrap_or(url).to_string();
            return Err(VaultError::NotFound { path });
        }

        if status >= 400 {
            let text = resp.text().await.unwrap_or_default();
            return Err(VaultError::Api {
                status,
                message: text,
            });
        }

        let text = resp.text().await?;
        serde_json::from_str::<T>(&text).map_err(|e| {
            error!("Vault JSON parse error for {}: {} — body: {}", url, e, &text[..text.len().min(200)]);
            VaultError::Json(e)
        })
    }

    // -----------------------------------------------------------------------
    // URL helpers
    // -----------------------------------------------------------------------

    fn kv_data_url(&self, path: &str) -> String {
        format!(
            "{}/v1/{}/data/{}",
            self.config.address.trim_end_matches('/'),
            self.config.kv_mount.trim_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn kv_metadata_url(&self, path: &str) -> String {
        format!(
            "{}/v1/{}/metadata/{}",
            self.config.address.trim_end_matches('/'),
            self.config.kv_mount.trim_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn kv_delete_url(&self, path: &str) -> String {
        format!(
            "{}/v1/{}/delete/{}",
            self.config.address.trim_end_matches('/'),
            self.config.kv_mount.trim_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

// ============================================================================
// VaultHealthStatus
// ============================================================================

/// Статус здоровья Vault
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultHealthStatus {
    /// Инициализирован, unsealed, active
    Active,
    /// Unsealed, standby
    Standby,
    /// DR secondary active
    DrSecondary,
    /// Не инициализирован
    NotInitialized,
    /// Sealed
    Sealed,
    /// Неизвестный статус
    Unknown(u16),
}

impl std::fmt::Display for VaultHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultHealthStatus::Active => write!(f, "active"),
            VaultHealthStatus::Standby => write!(f, "standby"),
            VaultHealthStatus::DrSecondary => write!(f, "dr_secondary"),
            VaultHealthStatus::NotInitialized => write!(f, "not_initialized"),
            VaultHealthStatus::Sealed => write!(f, "sealed"),
            VaultHealthStatus::Unknown(c) => write!(f, "unknown({})", c),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- VaultConfig ----

    #[test]
    fn test_config_with_token() {
        let cfg = VaultConfig::with_token("https://vault.example.com", "s.abc123");
        assert_eq!(cfg.address, "https://vault.example.com");
        assert_eq!(cfg.kv_mount, "secret");
        assert_eq!(cfg.timeout_secs, 10);
        assert!(cfg.namespace.is_none());
        match cfg.auth {
            VaultAuthMethod::Token(t) => assert_eq!(t, "s.abc123"),
            _ => panic!("expected Token auth"),
        }
    }

    #[test]
    fn test_config_with_approle() {
        let cfg =
            VaultConfig::with_approle("https://vault.example.com", "my-role-id", "my-secret-id");
        match cfg.auth {
            VaultAuthMethod::AppRole {
                role_id,
                secret_id,
                mount_path,
            } => {
                assert_eq!(role_id, "my-role-id");
                assert_eq!(secret_id, "my-secret-id");
                assert_eq!(mount_path, "approle");
            }
            _ => panic!("expected AppRole auth"),
        }
    }

    // ---- URL construction ----

    fn make_client_for_url_tests() -> VaultClient {
        let config = VaultConfig::with_token("https://vault.example.com:8200", "tok");
        VaultClient {
            config,
            http: Client::new(),
            token: RwLock::new("tok".to_string()),
        }
    }

    #[test]
    fn test_kv_data_url() {
        let c = make_client_for_url_tests();
        assert_eq!(
            c.kv_data_url("myapp/db"),
            "https://vault.example.com:8200/v1/secret/data/myapp/db"
        );
    }

    #[test]
    fn test_kv_metadata_url() {
        let c = make_client_for_url_tests();
        assert_eq!(
            c.kv_metadata_url("myapp/"),
            "https://vault.example.com:8200/v1/secret/metadata/myapp/"
        );
    }

    #[test]
    fn test_kv_delete_url() {
        let c = make_client_for_url_tests();
        assert_eq!(
            c.kv_delete_url("myapp/db"),
            "https://vault.example.com:8200/v1/secret/delete/myapp/db"
        );
    }

    #[test]
    fn test_kv_data_url_custom_mount() {
        let mut config = VaultConfig::with_token("https://vault.example.com", "tok");
        config.kv_mount = "kv".to_string();
        let c = VaultClient {
            config,
            http: Client::new(),
            token: RwLock::new("tok".to_string()),
        };
        assert_eq!(
            c.kv_data_url("prod/secrets"),
            "https://vault.example.com/v1/kv/data/prod/secrets"
        );
    }

    #[test]
    fn test_kv_data_url_trailing_slash_address() {
        let mut config = VaultConfig::with_token("https://vault.example.com/", "tok");
        config.kv_mount = "secret".to_string();
        let c = VaultClient {
            config,
            http: Client::new(),
            token: RwLock::new("tok".to_string()),
        };
        assert_eq!(
            c.kv_data_url("app/key"),
            "https://vault.example.com/v1/secret/data/app/key"
        );
    }

    // ---- Health status ----

    #[test]
    fn test_health_status_display() {
        assert_eq!(VaultHealthStatus::Active.to_string(), "active");
        assert_eq!(VaultHealthStatus::Standby.to_string(), "standby");
        assert_eq!(VaultHealthStatus::Sealed.to_string(), "sealed");
        assert_eq!(VaultHealthStatus::NotInitialized.to_string(), "not_initialized");
        assert_eq!(VaultHealthStatus::Unknown(418).to_string(), "unknown(418)");
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(VaultHealthStatus::Active, VaultHealthStatus::Active);
        assert_ne!(VaultHealthStatus::Active, VaultHealthStatus::Standby);
    }

    // ---- DynamicSecret / LeaseRenewal serialization ----

    #[test]
    fn test_dynamic_secret_serde() {
        let mut data = HashMap::new();
        data.insert("username".to_string(), serde_json::json!("v-approle-readonly-abc"));
        data.insert("password".to_string(), serde_json::json!("s3cr3t"));

        let ds = DynamicSecret {
            lease_id: "database/creds/readonly/abc123".to_string(),
            lease_duration: 3600,
            renewable: true,
            data,
        };

        let json = serde_json::to_string(&ds).unwrap();
        let restored: DynamicSecret = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.lease_id, "database/creds/readonly/abc123");
        assert_eq!(restored.lease_duration, 3600);
        assert!(restored.renewable);
        assert_eq!(
            restored.data["username"],
            serde_json::json!("v-approle-readonly-abc")
        );
    }

    #[test]
    fn test_lease_renewal_serde() {
        let lr = LeaseRenewal {
            lease_id: "database/creds/role/xyz".to_string(),
            lease_duration: 7200,
            renewable: true,
        };
        let json = serde_json::to_string(&lr).unwrap();
        let restored: LeaseRenewal = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.lease_id, "database/creds/role/xyz");
        assert_eq!(restored.lease_duration, 7200);
    }

    #[test]
    fn test_vault_secret_serde() {
        let mut data = HashMap::new();
        data.insert("api_key".to_string(), serde_json::json!("sk-abc123"));

        let vs = VaultSecret {
            data,
            version: 3,
            created_time: "2026-04-02T10:00:00Z".to_string(),
            destroyed: false,
        };

        let json = serde_json::to_string(&vs).unwrap();
        let restored: VaultSecret = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, 3);
        assert!(!restored.destroyed);
        assert_eq!(restored.data["api_key"], serde_json::json!("sk-abc123"));
    }

    // ---- Error types ----

    #[test]
    fn test_vault_error_not_found() {
        let err = VaultError::NotFound {
            path: "/secret/data/missing".to_string(),
        };
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn test_vault_error_api() {
        let err = VaultError::Api {
            status: 403,
            message: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("403"));
        assert!(err.to_string().contains("permission denied"));
    }
}
