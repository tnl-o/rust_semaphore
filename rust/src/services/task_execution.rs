//! Сервис выполнения задач
//!
//! Предоставляет единую точку запуска задач, используемую как HTTP-хендлером,
//! так и планировщиком (scheduler).

use std::sync::{Arc, Mutex};
use tracing::{error, info};

use crate::db::store::Store;
use crate::models::{Task, TaskOutput, Inventory, Repository, Environment};
use crate::services::task_logger::{TaskStatus, BasicLogger, TaskLogger, LogListener};
use crate::services::local_job::LocalJob;
use crate::db_lib::AccessKeyInstallerImpl;

/// Запускает задачу в фоновом потоке.
///
/// Загружает шаблон, инвентарь, репозиторий и окружение, запускает LocalJob,
/// сохраняет вывод в БД и обновляет статус задачи.
pub async fn execute_task(store: Arc<dyn Store + Send + Sync>, task: Task) {
    info!("[task_runner] Starting task {} (template {})", task.id, task.template_id);

    // Обновляем статус → Running
    match store.update_task_status(task.project_id, task.id, TaskStatus::Running).await {
        Ok(()) => info!("[task_runner] task {} status → Running", task.id),
        Err(e) => error!("[task_runner] task {} failed to set Running: {e}", task.id),
    }

    // Загружаем шаблон
    let template = match store.get_template(task.project_id, task.template_id).await {
        Ok(t) => t,
        Err(e) => {
            error!("[task_runner] task {}: failed to get template: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
            return;
        }
    };

    // Загружаем инвентарь, репозиторий, окружение
    let inventory_id = task.inventory_id.or(template.inventory_id);
    let inventory = match inventory_id {
        Some(id) => store.get_inventory(task.project_id, id).await.unwrap_or_default(),
        None => Inventory::default(),
    };

    let repository_id = task.repository_id.or(template.repository_id);
    let repository = match repository_id {
        Some(id) => store.get_repository(task.project_id, id).await.unwrap_or_default(),
        None => Repository::default(),
    };

    let environment_id = task.environment_id.or(template.environment_id);
    let environment = match environment_id {
        Some(id) => store.get_environment(task.project_id, id).await.unwrap_or_default(),
        None => Environment::default(),
    };

    // Логгер с буфером для сохранения в БД
    let log_buffer: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let buf_clone = log_buffer.clone();
    let logger = Arc::new(BasicLogger::new());
    logger.add_log_listener(Box::new(move |_time, msg| {
        let _ = buf_clone.lock().map(|mut v| v.push(msg));
    }));

    let work_dir = std::env::temp_dir().join(format!("semaphore_task_{}_{}", task.project_id, task.id));
    let tmp_dir = work_dir.join("tmp");

    if let Err(e) = tokio::fs::create_dir_all(&tmp_dir).await {
        error!("[task_runner] task {}: failed to create workdir: {e}", task.id);
        let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
        return;
    }

    let key_installer = AccessKeyInstallerImpl::new();
    let mut job = LocalJob::new(
        task.clone(),
        template,
        inventory,
        repository,
        environment,
        logger,
        key_installer,
        work_dir,
        tmp_dir,
    );

    job.store = Some(store.clone());
    let result = job.run("runner", None, "default").await;
    job.cleanup();

    // Сохраняем логи в БД
    let log_lines: Vec<String> = log_buffer.lock().map(|v| v.clone()).unwrap_or_default();
    for line in log_lines {
        let output = TaskOutput {
            id: 0,
            task_id: task.id,
            project_id: task.project_id,
            time: chrono::Utc::now(),
            output: line,
            stage_id: None,
        };
        let _ = store.create_task_output(output).await;
    }

    match result {
        Ok(()) => {
            info!("[task_runner] task {} completed successfully", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Success).await;
        }
        Err(e) => {
            error!("[task_runner] task {} failed: {e}", task.id);
            let _ = store.update_task_status(task.project_id, task.id, TaskStatus::Error).await;
        }
    }
}
