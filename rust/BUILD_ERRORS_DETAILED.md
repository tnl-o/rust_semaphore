# Детальный отчёт об ошибках компиляции Semaphore Rust
**Дата:** 2026-03-02 (обновлено: 2026-03-02, сессия 3)  
**Всего ошибок:** 625  
**Предупреждений:** ~200

---

## 📊 Статистика по категориям

| Категория | Количество ошибок | Приоритет | Статус |
|-----------|------------------|-----------|--------|
| StoreWrapper trait implementations | ~50 | 🔴 Критический | ✅ Исправлено |
| SQLx Type/Decode trait bounds | ~80 | 🔴 Критический | ⏳ В работе |
| BoltDB transaction methods | ~60 | 🔴 Критический | ⏳ В работе |
| Config methods vs fields | ~15 | 🟡 Высокий | ⏳ В работе |
| Missing struct fields | ~40 | 🟡 Высокий | ⏳ В работе |
| TemplateType Option handling | ~30 | 🟡 Высокий | ⏳ В работе |
| Exporter trait implementations | ~20 | 🟡 Высокий | ⏳ В работе |
| GitClient lifetime parameters | ~8 | 🟡 Высокий | ✅ Исправлено |
| LocalJob/Clone traits | ~15 | 🟡 Высокий | ✅ Исправлено |
| Missing crate dependencies | ~10 | 🟢 Средний | ⏳ В работе |
| Type mismatches (Option/String) | ~50 | 🟢 Средний | ⏳ В работе |
| Async/sync restore methods | ~30 | 🟢 Средний | ⏳ В работе |
| Unused imports/variables | ~200 | ⚪ Низкий | ⏳ В работе |

---

## ✅ ИСПРАВЛЕНО (242 ошибки)

### Этап 1: StoreWrapper и импорты трейтов

**Исправлено ошибок:** ~150

**Выполненные задачи:**
1. ✅ Исправлены сигнатуры методов StoreWrapper:
   - TaskManager: get_tasks, get_task_outputs
   - ScheduleManager: set_schedule_active, set_schedule_commit_hash
   - SessionManager: get_session, expire_session, verify_session, touch_session
   - TokenManager: get_api_token, expire_api_token, delete_api_token
   - EventManager: get_events
   - RunnerManager: get_runners
   - Добавлен Clone для StoreWrapper

2. ✅ Добавлен #[async_trait] для impl GitClient for GoGitClient

3. ✅ Добавлены импорты трейтов во все API handlers:
   - users.rs, user.rs, auth.rs, totp.rs: UserManager
   - events.rs: EventManager
   - access_key.rs: AccessKeyManager, ProjectStore
   - templates.rs: TemplateManager, ProjectStore
   - tasks.rs: TaskManager
   - schedules.rs: ScheduleManager
   - inventory.rs: InventoryManager
   - repository.rs: RepositoryManager
   - environment.rs: EnvironmentManager
   - integration.rs: IntegrationManager
   - views.rs: ViewManager
   - secret_storages.rs: SecretStorageManager
   - project.rs: ProjectStore

4. ✅ Исправлены mismatched types:
   - events.rs: get_events (None, limit) вместо (None, params)
   - tasks.rs: get_tasks (project_id, None) вместо (project_id, params)
   - users.rs: update_user (возвращает user_to_update)
   - user.rs: update_user (возвращает current_user)

5. ✅ Удалено дублирующееся определение RetrieveQueryParams в users.rs

**Коммиты:** 9
- `set_schedule_active` (добавить project_id)
- `set_schedule_commit_hash` (добавить project_id, изменить тип hash на &str)
- `get_session` (изменить параметры на user_id, session_id: i32)
- `expire_session` (изменить параметры)
- `verify_session` (изменить параметры, вернуть Session)
- `touch_session` (изменить параметры)
- `get_api_token` (token_id: &str вместо i32)
- `expire_api_token` (добавить user_id, token_id: &str)
- `delete_api_token` (добавить user_id, token_id: &str)
- `get_events` (project_id: Option<i32>, limit: usize)
- `get_runners` (project_id: Option<i32>)

---

### 2. SQLx Type/Decode Trait Bounds (~80 ошибок)

**Файлы:** `src/models/*.rs`, `src/db/sql/*.rs`

**Проблема:** Кастомные типы не реализуют трейты SQLx

**Примеры ошибок:**
```
error[E0277]: the trait bound `UserTotp: sqlx::Decode<'_, _>` is not satisfied
   --> src/db/sql/user_crud.rs:32:41

error[E0277]: the trait bound `HashMap<std::string::String, JsonValue>: sqlx::Decode<'_, _>` is not satisfied
   --> src/db/sql/task_crud.rs:52:48

error[E0277]: the trait bound `models::access_key::AccessKeyOwner: sqlx::Decode<'_, _>` is not satisfied
   --> src/db/sql/access_key.rs:14:48
```

**Затронутые типы:**
- `UserTotp` - убрать FromRow или реализовать Type/Decode
- `UserEmailOtp` - убрать FromRow или реализовать Type/Decode
- `Task.params` (HashMap<String, JsonValue>) - использовать serde_json::Value
- `AccessKeyOwner` - реализовать Display + FromStr + SQLx трейты
- `SecretStorageType` - реализовать Display + FromStr + SQLx трейты
- `TemplateType` - реализовать Display + FromStr
- `InventoryType` - реализовать Display + FromStr
- `ProjectInvite` - реализовать SQLx трейты

**Решение:** Для enum реализовать Display и FromStr, для struct убрать FromRow если используется Option

---

### 3. BoltDB Transaction Methods (~60 ошибок)

**Файлы:** `src/db/bolt/*.rs`

**Проблема:** Методы `update` и `view` не реализованы для sled::Db

**Примеры ошибок:**
```
error[E0599]: no method named `update` found for struct `Db` in the current scope
  --> src/db/bolt/event.rs:17:17

error[E0599]: no method named `view` found for struct `Db` in the current scope
  --> src/db/bolt/event.rs:37:17

error[E0277]: the size for values of type `[u8]` cannot be known at compilation time
   --> src/db/bolt/user.rs:77:20
```

**Решение:** Использовать методы из trait `BoltDbOperations` или исправить использование транзакций sled

**Затронутые файлы:**
- `src/db/bolt/event.rs`
- `src/db/bolt/user.rs`
- `src/db/bolt/task.rs`
- `src/db/bolt/template.rs`
- `src/db/bolt/project.rs`
- `src/db/bolt/schedule.rs`
- `src/db/bolt/session.rs`

---

## 🟡 ВЫСОКИЙ ПРИОРИТЕТ

### 4. Config Methods vs Fields (~15 ошибок)

**Файлы:** `src/config/types.rs`, `src/cli/*.rs`, `src/api/user.rs`

**Проблема:** Поля конфига доступны только как методы

**Примеры ошибок:**
```
error[E0615]: attempted to take value of method `db_dialect` on type `config::types::Config`
  --> src/cli/cmd_server.rs:54:22

error[E0615]: attempted to take value of method `non_admin_can_create_project` on type `config::types::Config`
  --> src/api/user.rs:40:55
```

**Решение:** Использовать методы вместо полей:
- `config.db_dialect` → `config.db_dialect()`
- `config.db_path` → `config.db_path()`
- `config.non_admin_can_create_project` → `config.non_admin_can_create_project()`

---

### 5. Missing Struct Fields (~40 ошибок)

**Файлы:** `src/models/*.rs`, `src/services/*.rs`, `src/api/handlers/*.rs`

**Проблема:** Отсутствуют поля в структурах

**Примеры:**
```
error[E0063]: missing field `created` in initializer of `token::APIToken`
  --> src/api/user.rs:76:50

error[E0063]: missing fields `environment_id` and `repository_id` in initializer of `models::task::Task`
   --> src/services/scheduler.rs:158:20
```

**Решение:** Добавить поля в структуры или исправить инициализацию

**Затронутые структуры:**
- `APIToken` - добавить `created`
- `Task` - добавить `repository_id`, `environment_id`
- `Schedule` - добавить `cron`, `name`
- `AccessKey` - исправить поля
- `BackupProject` - добавить `r#type`, `default_secret_storage_id`

---

### 6. TemplateType Option Handling (~30 ошибок)

**Файлы:** `src/models/template.rs`, `src/services/local_job/run.rs`, `src/db/sql/template_utils.rs`

**Проблема:** `template_type` теперь `Option<TemplateType>`

**Примеры ошибок:**
```
error[E0308]: mismatched types
  --> src/services/local_job/run.rs:71:13
    | expected `Option<TemplateType>`, found `TemplateType`

error[E0599]: `std::option::Option<models::template::TemplateType>` doesn't implement `std::fmt::Display`
   --> src/db/sql/template_crud.rs:64:46
```

**Решение:**
1. Обернуть в `Some()` при сравнении
2. Использовать `.map(|t| t.to_string())` или `.unwrap_or_default()`
3. Реализовать Display для TemplateType

---

### 7. Exporter Trait Implementations (~20 ошибок)

**Файлы:** `src/services/exporter.rs`, `src/services/exporter_main.rs`

**Проблема:** Traits не реализованы для типов

**Примеры ошибок:**
```
error[E0277]: the trait bound `exporter::ExporterChain: exporter::DataExporter` is not satisfied
   --> src/services/exporter.rs:268:38

error[E0277]: the trait bound `exporter::ValueMap<models::user::User>: exporter::TypeExporter` is not satisfied
   --> src/services/exporter.rs:323:30
```

**Решение:** Реализовать трейты `DataExporter` для `ExporterChain` и `TypeExporter` для `ValueMap<T>`

---

### 8. GitClient Lifetime Parameters (~8 ошибок)

**Файлы:** `src/db_lib/go_git_client.rs`, `src/db_lib/cmd_git_client.rs`

**Проблема:** Lifetime параметры не совпадают с трейтом

**Примеры ошибок:**
```
error[E0195]: lifetime parameters or bounds on method `clone` do not match the trait declaration
   --> src/db_lib/go_git_client.rs:71:19
```

**Решение:** Добавить `#[async_trait]` и исправить сигнатуры методов

---

### 9. LocalJob/Clone Traits (~15 ошибок)

**Файлы:** `src/services/local_job/types.rs`, `src/services/task_pool_runner.rs`, `src/services/task_logger.rs`

**Проблема:** Отсутствует реализация Clone

**Примеры ошибок:**
```
error[E0277]: the trait bound `task_pool_types::RunningTask: Clone` is not satisfied
    --> src/services/task_pool_runner.rs:102:31

error[E0599]: no method named `clone` found for struct `AccessKeyInstallerImpl` in the current scope
   --> src/services/task_runner/lifecycle.rs:42:32
```

**Решение:**
- Добавить `#[derive(Clone)]` для `RunningTask`
- Реализовать Clone для `AccessKeyInstallerImpl`
- Реализовать Clone для `TaskLogger` или использовать Arc

---

## 🟢 СРЕДНИЙ ПРИОРИТЕТ

### 10. Missing Crate Dependencies (~10 ошибок)

**Файлы:** `Cargo.toml`, `src/config/config_helpers.rs`, `src/config/config_sysproc.rs`

**Проблема:** Отсутствуют зависимости в Cargo.toml

**Примеры ошибок:**
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `which`
  --> src/config/config_helpers.rs:10:5

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `libc`
  --> src/config/config_sysproc.rs:31:17
```

**Решение:** Добавить в Cargo.toml:
```toml
[dependencies]
which = "4"
libc = "0.2"
```

---

### 11. Type Mismatches Option/String (~50 ошибок)

**Файлы:** `src/services/**/*.rs`, `src/db/bolt/*.rs`

**Проблема:** Несоответствие типов Option<T> и T

**Примеры ошибок:**
```
error[E0308]: mismatched types
   --> src/services/backup.rs:233:29
    | expected `String`, found `Option<String>`

error[E0308]: mismatched types
   --> src/services/restore.rs:169:26
    | expected `Option<String>`, found `String`
```

**Решение:**
- Использовать `.clone().expect("REASON")` или `.unwrap_or_default()`
- Исправить типы полей в структурах

---

### 12. Async/Sync Restore Methods (~30 ошибок)

**Файлы:** `src/services/restore.rs`

**Проблема:** Синхронные методы вызывают асинхронные store методы

**Примеры ошибок:**
```
error[E0277]: the `?` operator can only be applied to values that implement `Try`
   --> src/services/restore.rs:110:23
    | the `?` operator cannot be applied to type `Pin<Box<dyn Future<...>>`
```

**Решение:** Сделать методы `restore` асинхронными (`async fn`)

---

## ⚪ НИЗКИЙ ПРИОРИТЕТ (Предупреждения)

### 13. Unused Imports/Variables (~280 предупреждений)

**Файлы:** Все файлы проекта

**Проблема:** Неиспользуемые импорты и переменные

**Примеры:**
```
warning: unused import: `crate::error::Result`
  --> src/api/apps.rs:13:5

warning: unused variable: `state`
  --> src/api/apps.rs:31:11
```

**Решение:**
- Удалить неиспользуемые импорты
- Префиксировать неиспользуемые переменные `_`

---

## 📋 ПЛАН ИСПРАВЛЕНИЯ

### Этап 1: Критические ошибки (4-6 часов)
1. ✅ Исправить StoreWrapper сигнатуры методов
2. ✅ Исправить SQLx трейты для базовых типов
3. Исправить BoltDB транзакции

### Этап 2: Высокий приоритет (3-4 часа)
4. Исправить Config methods
5. Добавить missing struct fields
6. Исправить TemplateType Option handling
7. Исправить Exporter traits
8. Исправить GitClient lifetimes
9. Добавить Clone traits

### Этап 3: Средний приоритет (2-3 часа)
10. Добавить crate dependencies
11. Исправить type mismatches
12. Исправить async/sync restore

### Этап 4: Предупреждения (1-2 часа)
13. Удалить unused imports/variables

---

## 📝 ЗАМЕЧАНИЯ

1. **StoreWrapper** - самая большая проблема, требует внимательной сверки с трейтами
2. **SQLx трейты** - требуют понимания как работает сериализация в SQLx
3. **BoltDB** - возможно требует рефакторинга использования sled транзакций
4. После исправления всех ошибок требуется полная перепроверка `cargo build`

---

**Последнее обновление:** 2026-03-02  
**Статус:** В работе
