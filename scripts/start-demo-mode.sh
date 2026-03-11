#!/bin/bash
# ============================================================================
# Скрипт запуска демо-режима Semaphore UI
# ============================================================================
# Этот скрипт:
#   1. Запускает PostgreSQL контейнер с демо-данными
#   2. Инициализирует БД демонстрационными данными
# ============================================================================
# Использование:
#   ./scripts/start-demo-mode.sh [--clean] [--stop] [--logs] [--status]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DB_DIR="$PROJECT_DIR/db/postgres"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

check_docker() {
    if ! command -v docker &> /dev/null; then
        log_error "Docker не найден."
        exit 1
    fi
    if ! docker ps &> /dev/null; then
        log_error "Docker daemon не запущен."
        exit 1
    fi
}

stop_demo() {
    log_info "Остановка демо-режима..."
    cd "$PROJECT_DIR"
    docker-compose -f docker-compose.postgres.yml down 2>/dev/null || true
    log_success "Демо-режим остановлен."
}

clean_db() {
    log_warning "Очистка данных БД..."
    docker volume rm semaphore_postgres_data 2>/dev/null || true
    log_success "Данные БД очищены."
}

show_logs() {
    cd "$PROJECT_DIR"
    docker-compose -f docker-compose.postgres.yml logs -f
}

show_status() {
    echo "=== Статус демо-режима ==="
    CONTAINER_STATUS=$(docker ps -f name=semaphore-db --format "{{.Status}}" 2>/dev/null || echo "не запущен")
    echo "PostgreSQL контейнер: ${CONTAINER_STATUS:-не запущен}"
    
    if docker ps -f name=semaphore-db --format "{{.Names}}" | grep -q semaphore-db; then
        echo ""
        echo "Статистика БД:"
        docker exec semaphore-db psql -U semaphore -d semaphore -c "
            SELECT 'Users' as entity, COUNT(*) as count FROM \"user\"
            UNION ALL SELECT 'Projects', COUNT(*) FROM project
            UNION ALL SELECT 'Templates', COUNT(*) FROM template
            UNION ALL SELECT 'Tasks', COUNT(*) FROM task
            UNION ALL SELECT 'Schedules', COUNT(*) FROM schedule
            ORDER BY entity;
        " 2>/dev/null || echo "Не удалось получить статистику"
    fi
    
    echo ""
    echo "Доступ: http://localhost:3000"
    echo "Учётные данные (пароль: demo123):"
    echo "  - admin, john.doe, jane.smith, devops"
}

start_postgres() {
    log_info "Запуск PostgreSQL контейнера..."
    cd "$PROJECT_DIR"
    
    if docker ps -f name=semaphore-db --format "{{.Names}}" | grep -q semaphore-db; then
        log_warning "PostgreSQL контейнер уже запущен."
        return 0
    fi
    
    docker-compose -f docker-compose.postgres.yml up -d
    
    log_info "Ожидание готовности БД..."
    for i in {1..30}; do
        if docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT 1;" &> /dev/null; then
            log_success "БД готова!"
            break
        fi
        [ $i -eq 30 ] && { log_error "Таймаут ожидания готовности БД."; exit 1; }
        sleep 1
    done
}

load_demo_data() {
    log_info "Загрузка демонстрационных данных..."
    
    DEMO_MODE=$(docker exec semaphore-db psql -U semaphore -d semaphore -t -c "SELECT value FROM option WHERE key='demo_mode';" 2>/dev/null | tr -d ' ')
    
    if [ "$DEMO_MODE" = "true" ]; then
        log_warning "Демо-данные уже загружены."
        return 0
    fi
    
    docker exec -i semaphore-db psql -U semaphore -d semaphore < "$DB_DIR/fill-demo-data.sql"
    
    if [ $? -eq 0 ]; then
        log_success "Демо-данные успешно загружены!"
    else
        log_error "Ошибка при загрузке демо-данных."
        exit 1
    fi
}

main() {
    case "${1:-}" in
        --clean)
            check_docker
            stop_demo
            clean_db
            start_postgres
            load_demo_data
            log_success "Демо-режим готов!"
            ;;
        --stop)
            check_docker
            stop_demo
            ;;
        --logs)
            check_docker
            show_logs
            ;;
        --status)
            show_status
            ;;
        --help|-h)
            echo "Использование: $0 [--clean] [--stop] [--logs] [--status]"
            echo ""
            echo "Опции:"
            echo "  --clean  - Очистить БД и начать заново"
            echo "  --stop   - Остановить демо-режим"
            echo "  --logs   - Показать логи PostgreSQL"
            echo "  --status - Показать статус"
            echo "  --help   - Показать справку"
            echo ""
            echo "Без опций: инициализировать и запустить демо-режим"
            ;;
        *)
            check_docker
            log_info "Инициализация демо-режима Semaphore UI..."
            start_postgres
            load_demo_data
            echo ""
            log_success "✅ Демо-режим успешно инициализирован!"
            echo ""
            echo "=============================================="
            echo "  ДЕМО-РЕЖИМ ГОТОВ К РАБОТЕ"
            echo "=============================================="
            echo ""
            echo "📌 Доступ: http://localhost:3000"
            echo ""
            echo "🔐 Учётные данные (пароль: demo123):"
            echo "   • admin - администратор"
            echo "   • john.doe - менеджер Web Application"
            echo "   • jane.smith - менеджер Database"
            echo "   • devops - исполнитель задач"
            echo ""
            echo "📊 Демонстрационные данные:"
            echo "   • 4 проекта"
            echo "   • 12 шаблонов задач"
            echo "   • 4 расписания"
            echo "   • 6 задач"
            echo ""
            echo "📝 Следующие шаги:"
            echo "   1. Запустите backend: ./start.sh hybrid"
            echo "   2. Откройте: http://localhost:3000"
            echo ""
            echo "🛑 Остановка: $0 --stop"
            echo "📋 Статус: $0 --status"
            echo "=============================================="
            ;;
    esac
}

main "$@"
