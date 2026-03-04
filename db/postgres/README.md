# PostgreSQL для Semaphore

## Быстрый старт с Docker Compose

### 1. Запуск PostgreSQL

```bash
docker-compose -f docker-compose.postgres.yml up -d
```

Это создаст:
- Контейнер `semaphore_postgres` на порту **5433** (чтобы не конфликтовал с 5432)
- БД `semaphore` с пользователем `semaphore` / паролем `semaphore_pass`
- Применит минимальную миграцию из `db/postgres/init.sql`

### 2. Проверка подключения

```bash
docker exec -it semaphore_postgres psql -U semaphore -d semaphore
```

Внутри psql:
```sql
\dt              # показать таблицы
SELECT * FROM migration;  # проверить таблицу миграций
\q               # выйти
```

### 3. Запуск Semaphore с PostgreSQL

Создайте `.env` файл:

```env
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_HOST=localhost
SEMAPHORE_DB_PORT=5433
SEMAPHORE_DB_USER=semaphore
SEMAPHORE_DB_PASS=semaphore_pass
SEMAPHORE_DB_NAME=semaphore

SEMAPHORE_HTTP_PORT=3000
SEMAPHORE_ADMIN=admin
SEMAPHORE_ADMIN_PASSWORD=changeme
SEMAPHORE_ADMIN_NAME=Administrator
SEMAPHORE_ADMIN_EMAIL=admin@localhost

RUST_LOG=info
```

Запустите сервер:

```bash
cargo run -- server
```

Или с готовым connection string:

```bash
export DATABASE_URL="postgres://semaphore:semaphore_pass@localhost:5433/semaphore?sslmode=disable"
cargo run -- server
```

## Connection String формат

```
postgres://USER:PASSWORD@HOST:PORT/DB_NAME?OPTIONS
```

Примеры опций для PostgreSQL:
- `sslmode=disable` - отключить SSL (для локальной разработки)
- `connect_timeout=10` - таймаут подключения в секундах
- `application_name=semaphore` - имя приложения для логов PostgreSQL

Полный пример:
```
postgres://semaphore:semaphore_pass@localhost:5432/semaphore?sslmode=disable&connect_timeout=10
```

## Кастомные настройки PostgreSQL

Для продакшена рекомендуется настроить в `postgresql.conf`:

```conf
# Память
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB

# Логирование
log_statement = 'ddl'
log_min_duration_statement = 1000

# Подключения
max_connections = 100
```

И в `pg_hba.conf` настроить доступ:

```conf
# Локальный доступ
local   semaphore   semaphore                       trust
# IPv4
host    semaphore   semaphore   127.0.0.1/32        md5
# IPv6
host    semaphore   semaphore   ::1/128             md5
```

## Остановка и очистка

```bash
# Остановить контейнер
docker-compose -f docker-compose.postgres.yml down

# Остановить и удалить данные (volume)
docker-compose -f docker-compose.postgres.yml down -v
```

## Структура БД

Минимальная схема включает:

| Таблица | Описание |
|---------|----------|
| `migration` | Таблица версионирования миграций |
| `user` | Пользователи системы |
| `project` | Проекты |
| `project_user` | Связи пользователей с проектами |

Полная схема применяется автоматически при первом запуске Semaphore.

## Troubleshooting

### Ошибка "unable to open database file"

Убедитесь что:
1. PostgreSQL запущен: `docker ps | grep postgres`
2. Connection string правильный: `postgres://user:pass@host:port/dbname`
3. БД существует: `psql -h localhost -U semaphore -l | grep semaphore`

### Ошибка подключения

Проверьте логи PostgreSQL:
```bash
docker logs semaphore_postgres
```

Пересоздайте контейнер:
```bash
docker-compose -f docker-compose.postgres.yml down -v
docker-compose -f docker-compose.postgres.yml up -d
```
