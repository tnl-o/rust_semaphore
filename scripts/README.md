# Скрипты запуска Semaphore UI

Эта директория содержит скрипты для запуска сервера Semaphore с различными базами данных.

## Быстрый старт

### SQLite (рекомендуется для тестирования)

```bash
./scripts/run-sqlite.sh
```

### SQLite (тестовая БД в /tmp)

```bash
./scripts/run-test.sh
```

### MySQL

```bash
# С настройками по умолчанию
./scripts/run-mysql.sh

# С кастомными настройками
export SEMAPHORE_DB_HOST=db.example.com
export SEMAPHORE_DB_PORT=3307
export SEMAPHORE_DB_USER=myuser
export SEMAPHORE_DB_PASS=mypassword
export SEMAPHORE_DB_NAME=mydb
./scripts/run-mysql.sh
```

### PostgreSQL

```bash
# С настройками по умолчанию
./scripts/run-postgres.sh

# С кастомными настройками
export SEMAPHORE_DB_HOST=db.example.com
export SEMAPHORE_DB_PORT=5433
export SEMAPHORE_DB_USER=myuser
export SEMAPHORE_DB_PASS=mypassword
export SEMAPHORE_DB_NAME=mydb
./scripts/run-postgres.sh
```

## Переменные окружения

### Общие

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_PATH` | Путь к frontend | `./web/public` |

### SQLite

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_PATH` | Путь к файлу БД | `/var/lib/semaphore/semaphore.db` |

### MySQL

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_HOST` | Хост MySQL | `localhost` |
| `SEMAPHORE_DB_PORT` | Порт MySQL | `3306` |
| `SEMAPHORE_DB_USER` | Пользователь MySQL | `semaphore` |
| `SEMAPHORE_DB_PASS` | Пароль MySQL | `semaphore` |
| `SEMAPHORE_DB_NAME` | Имя базы данных | `semaphore` |

### PostgreSQL

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_HOST` | Хост PostgreSQL | `localhost` |
| `SEMAPHORE_DB_PORT` | Порт PostgreSQL | `5432` |
| `SEMAPHORE_DB_USER` | Пользователь PostgreSQL | `semaphore` |
| `SEMAPHORE_DB_PASS` | Пароль PostgreSQL | `semaphore` |
| `SEMAPHORE_DB_NAME` | Имя базы данных | `semaphore` |

## Создание пользователя

После первого запуска создайте администратора:

```bash
cd rust
cargo run -- user add \
    --username admin \
    --name "Administrator" \
    --email admin@localhost \
    --password admin123 \
    --admin
```

## Тестовый доступ

Для тестовой БД (`run-test.sh`):
- Логин: `admin`
- Пароль: `admin123`

Frontend доступен по адресу: http://localhost:3000
