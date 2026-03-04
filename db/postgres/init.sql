-- ============================================================================
-- Минимальная инициализация БД PostgreSQL для Semaphore
-- ============================================================================
-- Этот файл автоматически применяется при первом запуске PostgreSQL через
-- docker-entrypoint-initdb.d/
--
-- Использование:
--   docker-compose -f docker-compose.postgres.yml up -d
--
-- Или через скрипт:
--   ./scripts/postgres-quick-start.sh
-- ============================================================================

-- Таблица миграций (создаётся первой)

CREATE TABLE IF NOT EXISTS migration (
    version BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

-- Таблица пользователей
CREATE TABLE IF NOT EXISTS "user" (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица проектов
CREATE TABLE IF NOT EXISTS project (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Таблица связей пользователей с проектами
CREATE TABLE IF NOT EXISTS project_user (
    project_id BIGINT REFERENCES project(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES "user"(id) ON DELETE CASCADE,
    admin BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (project_id, user_id)
);

-- Индексы для производительности
CREATE INDEX IF NOT EXISTS idx_user_email ON "user"(email);
CREATE INDEX IF NOT EXISTS idx_user_username ON "user"(username);
CREATE INDEX IF NOT EXISTS idx_project_name ON project(name);
