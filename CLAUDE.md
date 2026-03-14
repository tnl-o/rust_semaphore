# CLAUDE.md — Инструкции для AI-агента

> Этот файл читается Claude Code при каждом запуске. Следуй порядку действий ниже.

---

## Порядок действий при каждом запуске

### 1. Проверка состояния git

```bash
git status
git log --oneline -5
git remote -v
```

- Убедись что ветка `main` (или нужная ветка)
- Проверь нет ли незакоммиченных изменений
- Если есть uncommitted changes — разберись до начала работы

---

### 2. Fetch + Merge из upstream

```bash
git fetch upstream
git log upstream/main --oneline -5   # посмотри что пришло
git merge upstream/main --no-edit
```

- Upstream: `https://github.com/alexandervashurin/semaphore`
- При конфликтах — разрешать вручную, сохраняя наши изменения как приоритет
- После merge сразу `cargo check` чтобы убедиться что всё компилируется

---

### 3. Проверка MASTER_PLAN.md

Читай `MASTER_PLAN.md`, раздел **"Текущее состояние"** и **"Известные проблемы и блокеры"**.

- Сверяй план с реальным кодом (код — источник правды)
- Если план расходится с кодом — обнови план
- Выбери следующую задачу по приоритету: 🔴 → 🟠 → 🟡

---

### 4. Начало работы

1. Создай todo-список через `TodoWrite` для задач сессии
2. Прочитай файлы которые планируешь менять перед правками
3. Помечай задачи `in_progress` перед началом, `completed` сразу после

---

### 5. В процессе работы

- `cargo check` после каждого значимого изменения в Rust
- Не накапливай изменения — коммить часто, небольшими смысловыми блоками
- Формат коммитов: **Conventional Commits**
  ```
  feat(auth): add refresh token endpoint
  fix(db): handle null values in migration
  docs(plan): update MASTER_PLAN status
  ```

---

### 6. Коммиты и пуши

```bash
# Стейджинг конкретных файлов (не git add -A)
git add rust/src/api/handlers/auth.rs rust/src/api/routes.rs

# Коммит с описанием что и зачем
git commit -m "feat(...): ..."

# Пуш в origin (только по явной просьбе пользователя)
git push origin main
```

> **Важно:** Пуш делать только когда пользователь явно попросит.

---

### 7. Обновление MASTER_PLAN.md

После завершения любой задачи:

1. Обновить статус задачи в таблице "Текущее состояние"
2. Закрыть блокер в таблице "Известные проблемы"
3. Обновить дату в заголовке: `**Последнее обновление:**`
4. Коммитить изменения плана вместе с кодом или отдельным коммитом:
   ```
   docs(plan): update MASTER_PLAN — close B-06b refresh token
   ```

---

## Ключевые факты о проекте

| Параметр | Значение |
|---|---|
| Репозиторий | `https://github.com/tnl-o/rust_semaphore` |
| Upstream | `https://github.com/alexandervashurin/semaphore` |
| Remote `origin` | наш форк |
| Remote `upstream` | alexandervashurin/semaphore |
| Язык бэкенда | Rust (axum + sqlx + tokio) |
| Фронтенд | Vanilla JS миграция (была Vue 2) |
| Рабочая директория Rust | `rust/` |
| Тесты | `cd rust && cargo test` |
| Lint | `cd rust && cargo clippy -- -D warnings` |

---

## Правила

- Всегда читай файл перед редактированием
- Не создавай новые файлы если можно отредактировать существующий
- Не добавляй комментарии, docstrings, type annotations к коду который не менял
- Не делай пуш без явного разрешения пользователя
- При merge-конфликтах: наш код (HEAD) имеет приоритет если сомневаешься
