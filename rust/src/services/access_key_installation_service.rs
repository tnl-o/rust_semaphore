//! AccessKey Installation Service
//!
//! Полная замена Go services/server/access_key_installation_svc.go
//! Сервис для установки ключей доступа

use crate::db_lib::{
    AccessKeyInstallerImpl, AccessKeyInstallerTrait, DbAccessKey, DbAccessKeyRole,
};
use crate::error::{Error, Result};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_logger::TaskLogger;

// ============================================================================
// Traits (интерфейсы)
// ============================================================================

/// Трейт сервиса установки ключей (аналог Go AccessKeyInstallationService)
pub trait AccessKeyInstallationServiceTrait: Send + Sync {
    /// Устанавливает ключ доступа
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation>;
}

// ============================================================================
/// AccessKeyEncryptionService - трейт для шифрования/дешифрования ключей
/// (аналог Go AccessKeyEncryptionService)
pub trait AccessKeyEncryptionService: Send + Sync {
    /// Шифрует секрет ключа
    fn encrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Дешифрует секрет ключа
    fn decrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Сериализует секрет (подготовка к хранению)
    fn serialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Десериализует секрет (чтение из хранилища)
    fn deserialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;
}

// ============================================================================
// AccessKeyInstallationServiceImpl
// ============================================================================

/// Реализация сервиса установки ключей
pub struct AccessKeyInstallationServiceImpl {
    encryption_service: Box<dyn AccessKeyEncryptionService>,
    key_installer: AccessKeyInstallerImpl,
}

impl AccessKeyInstallationServiceImpl {
    /// Создаёт новый сервис
    pub fn new(encryption_service: Box<dyn AccessKeyEncryptionService>) -> Self {
        Self {
            encryption_service,
            key_installer: AccessKeyInstallerImpl::new(),
        }
    }

    /// Создаёт сервис с кастомным установщиком
    pub fn with_installer(
        encryption_service: Box<dyn AccessKeyEncryptionService>,
        key_installer: AccessKeyInstallerImpl,
    ) -> Self {
        Self {
            encryption_service,
            key_installer,
        }
    }
}

impl AccessKeyInstallationServiceTrait for AccessKeyInstallationServiceImpl {
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation> {
        // Если тип ключа None - возвращаем пустую установку
        if key.key_type == crate::db_lib::DbAccessKeyType::None {
            return Ok(AccessKeyInstallation::new());
        }

        // Создаём копию ключа для десериализации
        let mut key_copy = key.clone();

        // Десериализуем секрет (расшифровываем)
        self.encryption_service.deserialize_secret(&mut key_copy)?;

        // Устанавливаем ключ через KeyInstaller
        self.key_installer.install(&key_copy, usage, logger)
    }
}

// ============================================================================
// AccessKeyEncryptionServiceImpl (заглушка)
// ============================================================================

/// Реализация шифрования секретов с AES-256-GCM
pub struct SimpleEncryptionService {
    /// 32-байтный ключ шифрования
    key: [u8; 32],
}

impl SimpleEncryptionService {
    /// Создаёт сервис с указанным ключом (UTF-8 строка, padded/truncated до 32 байт)
    pub fn new(secret: &str) -> Self {
        let mut key = [0u8; 32];
        let bytes = secret.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Self { key }
    }
}

impl Default for SimpleEncryptionService {
    fn default() -> Self {
        Self::new("semaphore-default-encryption-key")
    }
}

impl AccessKeyEncryptionService for SimpleEncryptionService {
    /// Шифрует поле `secret` ключа с помощью AES-256-GCM
    fn encrypt_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::utils::encryption::aes256_encrypt;
        if let Some(ref plaintext) = key.secret {
            let encrypted = aes256_encrypt(plaintext.as_bytes(), &self.key)
                .map_err(|e| Error::Other(e.to_string()))?;
            key.secret = Some(encrypted);
        }
        Ok(())
    }

    /// Дешифрует поле `secret` ключа
    fn decrypt_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::utils::encryption::aes256_decrypt;
        if let Some(ref encrypted) = key.secret {
            let plaintext_bytes =
                aes256_decrypt(encrypted, &self.key).map_err(|e| Error::Other(e.to_string()))?;
            key.secret =
                Some(String::from_utf8(plaintext_bytes).map_err(|e| Error::Other(e.to_string()))?);
        }
        Ok(())
    }

    /// Сериализует ssh_key / login_password → JSON → key.secret
    fn serialize_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::db_lib::DbAccessKeyType;
        match key.key_type {
            DbAccessKeyType::Ssh => {
                if let Some(ref ssh_key) = key.ssh_key {
                    key.secret = Some(
                        serde_json::to_string(ssh_key).map_err(|e| Error::Other(e.to_string()))?,
                    );
                }
            }
            DbAccessKeyType::LoginPassword => {
                if let Some(ref lp) = key.login_password {
                    key.secret =
                        Some(serde_json::to_string(lp).map_err(|e| Error::Other(e.to_string()))?);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Десериализует key.secret → ssh_key / login_password
    fn deserialize_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::db_lib::{DbAccessKeyType, DbLoginPassword, DbSshKey};
        if let Some(ref secret) = key.secret.clone() {
            match key.key_type {
                DbAccessKeyType::Ssh => {
                    let ssh_key: DbSshKey =
                        serde_json::from_str(secret).map_err(|e| Error::Other(e.to_string()))?;
                    key.ssh_key = Some(ssh_key);
                }
                DbAccessKeyType::LoginPassword => {
                    let lp: DbLoginPassword =
                        serde_json::from_str(secret).map_err(|e| Error::Other(e.to_string()))?;
                    key.login_password = Some(lp);
                }
                _ => {}
            }
        }
        Ok(())
    }
}


// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::{DbAccessKeyType, DbLoginPassword, DbSshKey};
    use crate::services::task_logger::BasicLogger;

    #[test]
    fn test_simple_encryption_service() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: Some(r#"{"login":"user","passphrase":"","private_key":"test"}"#.to_string()),
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        // Проверяем, что методы не паникуют
        assert!(encryption.encrypt_secret(&mut key).is_ok());
        assert!(encryption.decrypt_secret(&mut key).is_ok());
        assert!(encryption.serialize_secret(&mut key).is_ok());
        assert!(encryption.deserialize_secret(&mut key).is_ok());
    }

    #[test]
    fn test_access_key_installation_service_creation() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);

        // Проверяем, что сервис создан
        let _ = service;
    }

    #[test]
    fn test_access_key_installation_service_install_none() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "None Key".to_string(),
            key_type: DbAccessKeyType::None,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_none());
    }

    #[test]
    fn test_access_key_installation_service_install_ssh() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "SSH Key".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: Some(DbSshKey {
                login: "git".to_string(),
                passphrase: "".to_string(),
                private_key:
                    "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                        .to_string(),
            }),
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_access_key_service_creation() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyServiceImpl::new(encryption);

        // Проверяем, что сервис создан
        let _ = service;
    }

    #[test]
    fn test_access_key_service_create() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyServiceImpl::new(encryption);

        let key = DbAccessKey {
            id: 1,
            name: "Test Key".to_string(),
            key_type: DbAccessKeyType::LoginPassword,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: Some(DbLoginPassword {
                login: "user".to_string(),
                password: "pass".to_string(),
            }),
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.create(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_access_key_options() {
        let options = GetAccessKeyOptions {
            user_id: Some(1),
            environment_id: Some(2),
        };

        assert_eq!(options.user_id, Some(1));
        assert_eq!(options.environment_id, Some(2));
    }
}
