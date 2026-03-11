-- ============================================================================
-- Скрипт наполнения PostgreSQL тестовыми данными для Semaphore UI
-- ============================================================================
-- Использование:
--   docker exec -i semaphore-db psql -U semaphore -d semaphore < fill-demo-data.sql
-- ============================================================================

-- ============================================================================
-- Пользователи (пароль для всех: demo123, хеш bcrypt)
-- ============================================================================
INSERT INTO "user" (username, name, email, password, admin, external, alert, pro, created) VALUES
    ('admin', 'Administrator', 'admin@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', true, false, false, false, NOW()),
    ('john.doe', 'John Doe', 'john@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', false, false, false, false, NOW()),
    ('jane.smith', 'Jane Smith', 'jane@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', false, false, false, false, NOW()),
    ('devops', 'DevOps User', 'devops@localhost', '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzS3MebAJu', false, false, false, false, NOW())
ON CONFLICT (username) DO NOTHING;

-- ============================================================================
-- Проекты
-- ============================================================================
INSERT INTO project (name, created, alert, max_parallel_tasks, type) VALUES
    ('Demo Project', NOW(), false, 0, 'default'),
    ('Infrastructure', NOW(), false, 0, 'default'),
    ('Web Applications', NOW(), false, 0, 'default'),
    ('Database Cluster', NOW(), false, 0, 'default')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- Связи пользователей с проектами
-- ============================================================================
INSERT INTO project_user (project_id, user_id, role, created)
SELECT 1, u.id, 'owner', NOW() FROM "user" u WHERE u.username = 'admin'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 1, u.id, 'manager', NOW() FROM "user" u WHERE u.username IN ('john.doe', 'jane.smith')
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 1, u.id, 'worker', NOW() FROM "user" u WHERE u.username = 'devops'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 2, u.id, 'owner', NOW() FROM "user" u WHERE u.username = 'admin'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 2, u.id, 'manager', NOW() FROM "user" u WHERE u.username IN ('john.doe')
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 2, u.id, 'worker', NOW() FROM "user" u WHERE u.username = 'devops'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 3, u.id, 'owner', NOW() FROM "user" u WHERE u.username = 'admin'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 3, u.id, 'manager', NOW() FROM "user" u WHERE u.username IN ('jane.smith')
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 3, u.id, 'worker', NOW() FROM "user" u WHERE u.username = 'devops'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 4, u.id, 'owner', NOW() FROM "user" u WHERE u.username = 'admin'
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 4, u.id, 'manager', NOW() FROM "user" u WHERE u.username IN ('john.doe', 'jane.smith')
ON CONFLICT (project_id, user_id) DO NOTHING;

INSERT INTO project_user (project_id, user_id, role, created)
SELECT 4, u.id, 'worker', NOW() FROM "user" u WHERE u.username = 'devops'
ON CONFLICT (project_id, user_id) DO NOTHING;

-- ============================================================================
-- Опции
-- ============================================================================
INSERT INTO option (key, value) VALUES
    ('demo_mode', 'true'),
    ('jwt_secret', 'demo-secret-key-12345'),
    ('session_timeout', '86400')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

-- ============================================================================
-- Проверка результатов
-- ============================================================================
SELECT 'Users created: ' || COUNT(*) FROM "user";
SELECT 'Projects created: ' || COUNT(*) FROM project;
SELECT 'Project-User links: ' || COUNT(*) FROM project_user;
