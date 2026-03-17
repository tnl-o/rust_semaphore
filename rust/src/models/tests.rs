//! Тесты для моделей (Models)
//!
//! Тестирование структур данных и их методов

use crate::models::*;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // Access Key Tests
    // ============================================

    #[test]
    fn test_access_key_type_display() {
        // Проверка отображения типа ключа
        assert_eq!(AccessKeyType::Ssh.to_string(), "ssh");
        assert_eq!(AccessKeyType::LoginPassword.to_string(), "login_password");
        assert_eq!(AccessKeyType::None.to_string(), "none");
    }

    #[test]
    fn test_access_key_type_from_str() {
        // Парсинг типа ключа из строки
        assert_eq!("ssh".parse::<AccessKeyType>().unwrap(), AccessKeyType::Ssh);
        assert_eq!("login_password".parse::<AccessKeyType>().unwrap(), AccessKeyType::LoginPassword);
        assert_eq!("none".parse::<AccessKeyType>().unwrap(), AccessKeyType::None);
        assert!("invalid".parse::<AccessKeyType>().is_err());
    }

    #[test]
    fn test_access_key_new_ssh() {
        // Создание SSH ключа
        let key = AccessKey::new_ssh(
            "Test SSH Key".to_string(),
            "-----BEGIN RSA PRIVATE KEY-----...".to_string(),
            Some("passphrase".to_string()),
        );

        assert_eq!(key.name, "Test SSH Key");
        assert_eq!(key.key_type, AccessKeyType::Ssh);
        assert!(key.secret.is_some());
    }

    #[test]
    fn test_access_key_new_login_password() {
        // Создание ключа login/password
        let key = AccessKey::new_login_password(
            "Test Login".to_string(),
            "username".to_string(),
            "password123".to_string(),
        );

        assert_eq!(key.name, "Test Login");
        assert_eq!(key.key_type, AccessKeyType::LoginPassword);
        assert!(key.secret.is_some());
    }

    #[test]
    fn test_access_key_get_type() {
        // Получение типа ключа
        let ssh_key = AccessKey {
            id: 1,
            project_id: 1,
            name: "SSH Key".to_string(),
            key_type: AccessKeyType::Ssh,
            secret: None,
            ..Default::default()
        };

        assert_eq!(ssh_key.get_type(), AccessKeyType::Ssh);
    }

    // ============================================
    // Inventory Tests
    // ============================================

    #[test]
    fn test_inventory_type_serialization() {
        // Сериализация типа инвентаря
        let json = serde_json::to_string(&InventoryType::Static).unwrap();
        assert_eq!(json, "\"static\"");

        let json = serde_json::to_string(&InventoryType::File).unwrap();
        assert_eq!(json, "\"file\"");

        let json = serde_json::to_string(&InventoryType::StaticYaml).unwrap();
        assert_eq!(json, "\"static-yaml\"");
    }

    #[test]
    fn test_inventory_type_from_str() {
        // Парсинг типа инвентаря
        assert_eq!("static".parse::<InventoryType>().unwrap(), InventoryType::Static);
        assert_eq!("file".parse::<InventoryType>().unwrap(), InventoryType::File);
        assert_eq!("static-yaml".parse::<InventoryType>().unwrap(), InventoryType::StaticYaml);
        assert!("invalid".parse::<InventoryType>().is_err());
    }

    #[test]
    fn test_inventory_new() {
        // Создание инвентаря
        let inventory = Inventory::new(
            "Test Inventory".to_string(),
            InventoryType::Static,
            "[all]\nhost1\nhost2".to_string(),
        );

        assert_eq!(inventory.name, "Test Inventory");
        assert_eq!(inventory.inventory_type, InventoryType::Static);
        assert_eq!(inventory.inventory, "[all]\nhost1\nhost2");
    }

    // ============================================
    // Repository Tests
    // ============================================

    #[test]
    fn test_repository_type_display() {
        // Отображение типа репозитория
        assert_eq!(RepositoryType::Git.to_string(), "git");
        assert_eq!(RepositoryType::Github.to_string(), "github");
        assert_eq!(RepositoryType::Gitlab.to_string(), "gitlab");
        assert_eq!(RepositoryType::Bitbucket.to_string(), "bitbucket");
        assert_eq!(RepositoryType::Https.to_string(), "https");
        assert_eq!(RepositoryType::File.to_string(), "file");
    }

    #[test]
    fn test_repository_type_from_str() {
        // Парсинг типа репозитория
        assert_eq!("git".parse::<RepositoryType>().unwrap(), RepositoryType::Git);
        assert_eq!("github".parse::<RepositoryType>().unwrap(), RepositoryType::Github);
        assert_eq!("https".parse::<RepositoryType>().unwrap(), RepositoryType::Https);
        assert!("invalid".parse::<RepositoryType>().is_err());
    }

    #[test]
    fn test_repository_new() {
        // Создание репозитория
        let repo = Repository::new(
            "Test Repo".to_string(),
            "https://github.com/test/repo.git".to_string(),
            RepositoryType::Git,
        );

        assert_eq!(repo.name, "Test Repo");
        assert_eq!(repo.git_url, "https://github.com/test/repo.git");
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    // ============================================
    // Environment Tests
    // ============================================

    #[test]
    fn test_environment_new() {
        // Создание окружения
        let env = Environment::new(
            "Production".to_string(),
            r#"{"DB_HOST": "localhost", "DB_PORT": "5432"}"#.to_string(),
        );

        assert_eq!(env.name, "Production");
        assert_eq!(env.json, r#"{"DB_HOST": "localhost", "DB_PORT": "5432"}"#);
    }

    #[test]
    fn test_environment_json_parse() {
        // Парсинг JSON окружения
        let env = Environment {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            json: r#"{"KEY": "value"}"#.to_string(),
            ..Default::default()
        };

        let parsed: serde_json::Value = serde_json::from_str(&env.json).unwrap();
        assert_eq!(parsed["KEY"], "value");
    }

    // ============================================
    // Schedule Tests
    // ============================================

    #[test]
    fn test_schedule_new() {
        // Создание расписания
        let schedule = Schedule::new(
            "Daily Backup".to_string(),
            1, // template_id
            "0 0 * * *".to_string(), // cron
        );

        assert_eq!(schedule.name, "Daily Backup");
        assert_eq!(schedule.template_id, 1);
        assert_eq!(schedule.cron, "0 0 * * *");
        assert!(schedule.active);
    }

    #[test]
    fn test_schedule_cron_validation() {
        // Валидация cron выражения
        let valid_crons = [
            "0 * * * *",      // Ежечасно
            "0 0 * * *",      // Ежедневно
            "0 0 * * 0",      // Еженедельно
            "0 0 1 * *",      // Ежемесячно
            "*/5 * * * *",    // Каждые 5 минут
        ];

        for cron in valid_crons.iter() {
            // Простая проверка формата (5 полей)
            let parts: Vec<&str> = cron.split_whitespace().collect();
            assert_eq!(parts.len(), 5, "Invalid cron: {}", cron);
        }
    }

    // ============================================
    // Webhook Tests
    // ============================================

    #[test]
    fn test_webhook_type_display() {
        // Отображение типа webhook
        assert_eq!(WebhookType::Generic.to_string(), "generic");
        assert_eq!(WebhookType::Slack.to_string(), "slack");
        assert_eq!(WebhookType::Telegram.to_string(), "telegram");
        assert_eq!(WebhookType::Discord.to_string(), "discord");
        assert_eq!(WebhookType::Teams.to_string(), "teams");
    }

    #[test]
    fn test_webhook_type_from_str() {
        // Парсинг типа webhook
        assert_eq!("generic".parse::<WebhookType>().unwrap(), WebhookType::Generic);
        assert_eq!("slack".parse::<WebhookType>().unwrap(), WebhookType::Slack);
        assert_eq!("telegram".parse::<WebhookType>().unwrap(), WebhookType::Telegram);
        assert!("invalid".parse::<WebhookType>().is_err());
    }

    #[test]
    fn test_webhook_new() {
        // Создание webhook
        let webhook = Webhook::new(
            "Test Webhook".to_string(),
            WebhookType::Generic,
            "https://example.com/webhook".to_string(),
        );

        assert_eq!(webhook.name, "Test Webhook");
        assert_eq!(webhook.hook_type, WebhookType::Generic);
        assert_eq!(webhook.url, "https://example.com/webhook");
        assert!(webhook.active);
    }

    // ============================================
    // User Tests
    // ============================================

    #[test]
    fn test_user_validate_empty_username() {
        // Валидация: пустое имя пользователя
        let user = User {
            id: 0,
            name: "Test".to_string(),
            username: "".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            ..Default::default()
        };

        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validate_empty_email() {
        // Валидация: пустой email
        let user = User {
            id: 0,
            name: "Test".to_string(),
            username: "testuser".to_string(),
            email: "".to_string(),
            password: "password".to_string(),
            ..Default::default()
        };

        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validate_success() {
        // Валидация: успешная
        let user = User {
            id: 0,
            name: "Test User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            ..Default::default()
        };

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_role_display() {
        // Отображение роли пользователя
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::User.to_string(), "user");
    }
}
