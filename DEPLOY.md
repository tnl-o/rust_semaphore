# DEPLOY — Руководство по развёртыванию Velum

Velum — Rust/Axum переписывание [Semaphore](https://github.com/semaphoreui/semaphore).
Поддерживает SQLite (demo/dev) и PostgreSQL (prod).

---

## Быстрый старт — Demo (1 команда)

```bash
docker compose -f deploy/demo/docker-compose.yml up --build -d
```

Открыть: **http://localhost:8088** · Логин: **admin / admin123**

---

## Варианты развёртывания

### 1. Demo — попробовать Velum

SQLite, без настройки, порт 8088.

```bash
cd deploy/demo
cp .env.example .env
docker compose up --build -d
```

Наполнить тестовыми данными:
```bash
bash fill-sqlite-demo-data.sh
```

---

### 2. Dev — разработка с hot-reload

PostgreSQL + cargo-watch. Backend перекомпилируется при изменении кода.

```bash
cd deploy/dev
cp .env.example .env
docker compose up -d
```

URL: http://localhost:3000

**Нативный запуск** (быстрее для разработки — только DB в Docker):

```bash
# Запустить только PostgreSQL
cd deploy/dev && docker compose up -d db

# Backend на хосте
cd rust
VELUM_DB_URL="postgres://velum:velum_dev_pass@localhost:5432/velum" \
VELUM_WEB_PATH=../web/public \
VELUM_ADMIN=admin VELUM_ADMIN_PASSWORD=admin123 \
cargo run -- server
```

---

### 3. Prod — продакшен с PostgreSQL + Nginx

PostgreSQL с изолированной сетью, Nginx как reverse-proxy.

```bash
cd deploy/prod
cp .env.example .env
nano .env           # ← обязательно сменить пароли!
docker compose up -d --build
```

URL: http://localhost:80

**Минимальный чеклист безопасности перед запуском:**

- [ ] Сменить `POSTGRES_PASSWORD` (≥16 символов)
- [ ] Сгенерировать `VELUM_JWT_SECRET`: `openssl rand -hex 32`
- [ ] Сменить `VELUM_ADMIN_PASSWORD`
- [ ] Настроить HTTPS (Let's Encrypt + certbot)
- [ ] Убедиться что PostgreSQL не экспонирован на публичный порт

---

### 4. Нативный запуск (без Docker)

```bash
# Сборка
cd rust && cargo build --release

# Запуск (SQLite)
VELUM_DB_PATH=/var/lib/velum/velum.db \
VELUM_WEB_PATH=/opt/velum/web \
VELUM_ADMIN=admin \
VELUM_ADMIN_PASSWORD=admin123 \
./target/release/velum server --host 0.0.0.0 --port 3000
```

Или используй готовый скрипт:
```bash
bash start-server.sh
```

---

### 5. DEB-пакет (Debian/Ubuntu)

```bash
# Сборка пакета
bash scripts/build-deb.sh

# Установка
sudo dpkg -i dist/velum_*.deb
sudo systemctl enable --now velum

# Конфиг: /etc/velum/velum.conf
# Логи:   journalctl -u velum -f
```

Подробнее: [docs/DEB_PACKAGE.md](docs/DEB_PACKAGE.md)

---

## Переменные окружения

| Переменная | По умолчанию | Описание |
|---|---|---|
| `VELUM_DB_DIALECT` | `sqlite` | Диалект БД: `sqlite` / `postgres` / `mysql` |
| `VELUM_DB_PATH` | `/tmp/velum.db` | Путь к SQLite-файлу |
| `VELUM_DB_URL` | — | Строка подключения PostgreSQL/MySQL |
| `VELUM_WEB_PATH` | `./web/public` | Путь к статическим файлам UI |
| `VELUM_TMP_PATH` | `/tmp/velum` | Временная папка для задач |
| `VELUM_JWT_SECRET` | `secret` | JWT-секрет (обязательно сменить!) |
| `VELUM_ADMIN` | — | Логин первого администратора |
| `VELUM_ADMIN_PASSWORD` | — | Пароль первого администратора |
| `VELUM_ADMIN_EMAIL` | — | Email первого администратора |
| `VELUM_LDAP_*` | — | LDAP-настройки (опционально) |
| `VELUM_OIDC_*` | — | OIDC-настройки (опционально) |
| `RUST_LOG` | `info` | Уровень логирования |

---

## Команды разработки

```bash
# Компиляция
cd rust && cargo build

# Линтер (должен быть 0 warnings)
cd rust && cargo clippy -- -D warnings

# Тесты
cd rust && cargo test

# Сборка релизного бинаря
cd rust && cargo build --release
```

---

## Обновление

```bash
# Получить последние изменения
git pull origin main

# Пересобрать и перезапустить (prod)
cd deploy/prod && docker compose up -d --build

# Demo
cd deploy/demo && docker compose up --build -d
```

---

## Структура проекта

```
velum/
├── rust/               # Backend — Rust (Axum, SQLx, Tokio)
│   └── src/
│       ├── api/        # HTTP handlers, middleware, routes
│       ├── db/         # Database adapters (SQLite, PostgreSQL, MySQL)
│       ├── services/   # Бизнес-логика (task runner, scheduler, auth)
│       └── cli/        # CLI команды (server, fill-demo-data, ...)
├── web/
│   └── public/         # Frontend — Vanilla JS + Material Design
│       ├── app.js      # API client, sidebar, утилиты
│       ├── styles.css  # Material Design CSS
│       └── *.html      # 30+ страниц UI
├── db/
│   ├── postgres/       # PostgreSQL: init.sql, migrations
│   └── migrations/     # SQLite migrations
├── deploy/             # Конфиги развёртывания
│   ├── demo/           # SQLite demo (docker-compose + .env.example)
│   ├── dev/            # Разработка с hot-reload
│   └── prod/           # Продакшен (PostgreSQL + Nginx)
├── docs/               # Документация
│   ├── technical/      # API, Auth, Config, Performance
│   ├── guides/         # Setup, Testing, Demo Data
│   ├── releases/       # Changelog, Release notes
│   ├── future/         # Roadmap, планируемые фичи
│   └── archive/        # Старые отчёты
├── scripts/            # Вспомогательные скрипты (build-deb, apply-migration)
└── DEPLOY.md           # ← этот файл
```

---

## Поддерживаемые БД

| БД | Статус | Использование |
|---|---|---|
| SQLite | ✅ Prod-ready | Demo, маленькие деплои |
| PostgreSQL 13+ | ✅ Prod-ready | Рекомендован для продакшена |
| MySQL 8+ | ✅ Поддерживается | Альтернатива PostgreSQL |

---

## Помощь

- [GitHub Issues](https://github.com/tnl-o/velum/issues)
- [docs/guides/TROUBLESHOOTING.md](docs/guides/TROUBLESHOOTING.md)
- [docs/technical/API.md](docs/technical/API.md)
