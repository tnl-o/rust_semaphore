//! Task - операции с задачами в BoltDB
//!
//! Аналог db/bolt/task.go из Go версии

use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{Task, TaskWithTpl, TaskOutput, TaskStage, TaskStageWithResult, RetrieveQueryParams};
use chrono::Utc;

impl BoltStore {
    /// Создаёт новую задачу
    pub async fn create_task(&self, mut task: Task, max_tasks: i32) -> Result<Task> {
        task.created = Utc::now();

        let task_clone = task.clone();

        let tree = self.db.open_tree(b"tasks")
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id = self.get_next_id("tasks")?;
        let id_key = (i64::MAX - id as i64).to_be_bytes();

        let mut task_with_id = task_clone;
        task_with_id.id = id as i32;

        let str = serde_json::to_vec(&task_with_id)
            .map_err(|e| crate::error::Error::Json(e))?;

        tree.insert(id_key, str)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        // Очищаем старые задачи если нужно
        if max_tasks > 0 {
            self.clear_tasks(task.project_id, task.template_id, max_tasks).await?;
        }

        Ok(task_with_id)
    }

    /// Получает задачу по ID
    pub async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        let tree = self.db.open_tree(b"tasks")
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let key = (i64::MAX - task_id as i64).to_be_bytes();

        if let Some(v) = tree.get(key)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))? {
            let task: Task = serde_json::from_slice(&v)
                .map_err(|e| crate::error::Error::Json(e))?;
            if task.project_id == project_id {
                Ok(task)
            } else {
                Err(crate::error::Error::NotFound("Задача не найдена".to_string()))
            }
        } else {
            Err(crate::error::Error::NotFound("Задача не найдена".to_string()))
        }
    }

    /// Получает задачи шаблона
    pub async fn get_template_tasks(&self, project_id: i32, template_id: i32, params: RetrieveQueryParams) -> Result<Vec<TaskWithTpl>> {
        self.get_tasks_internal(project_id, Some(template_id), params).await
    }

    /// Получает задачи проекта
    pub async fn get_project_tasks(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<TaskWithTpl>> {
        self.get_tasks_internal(project_id, None, params).await
    }

    /// Обновляет задачу
    pub async fn update_task(&self, task: Task) -> Result<()> {
        let tree = self.db.open_tree(b"tasks")
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let key = (i64::MAX - task.id as i64).to_be_bytes();

        if tree.get(key)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?.is_none() {
            return Err(crate::error::Error::NotFound("Задача не найдена".to_string()));
        }

        let str = serde_json::to_vec(&task)
            .map_err(|e| crate::error::Error::Json(e))?;

        tree.insert(key, str)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Удаляет задачу с выводами
    pub async fn delete_task_with_outputs(&self, project_id: i32, task_id: i32) -> Result<()> {
        let tree = self.db.open_tree(b"tasks")
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let key = (i64::MAX - task_id as i64).to_be_bytes();

        if tree.get(key)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?.is_none() {
            return Err(crate::error::Error::NotFound("Задача не найдена".to_string()));
        }

        tree.remove(key)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        // Удаляем выводы задачи
        let outputs_bucket_name = format!("task_outputs_{}", task_id);
        let outputs_tree = self.db.open_tree(outputs_bucket_name.as_bytes())
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        outputs_tree.clear()
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        // Удаляем стадии задачи
        let stages_bucket_name = format!("task_stages_{}", task_id);
        let stages_tree = self.db.open_tree(stages_bucket_name.as_bytes())
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        stages_tree.clear()
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Создаёт вывод задачи
    pub async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        output.time = Utc::now();

        let bucket_name = format!("task_outputs_{}", output.task_id);
        let tree = self.db.open_tree(bucket_name.as_bytes())
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let id = self.get_next_id(&bucket_name)?;
        let id_key = (i64::MAX - id as i64).to_be_bytes();

        let mut output_with_id = output;
        output_with_id.id = id as i32;

        let str = serde_json::to_vec(&output_with_id)
            .map_err(|e| crate::error::Error::Json(e))?;

        tree.insert(id_key, str)
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(output_with_id)
    }

    /// Пакетная вставка выводов задач
    pub async fn insert_task_output_batch(&self, outputs: Vec<TaskOutput>) -> Result<()> {
        for output in outputs {
            self.create_task_output(output).await?;
        }
        Ok(())
    }

    /// Получает выводы задачи
    pub async fn get_task_outputs(&self, project_id: i32, task_id: i32, params: RetrieveQueryParams) -> Result<Vec<TaskOutput>> {
        // Проверяем существование задачи
        self.get_task(project_id, task_id).await?;

        let bucket_name = format!("task_outputs_{}", task_id);
        let tree = self.db.open_tree(bucket_name.as_bytes())
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let mut outputs = Vec::new();
        let mut count = 0;
        let mut skipped = 0;

        for item in tree.iter() {
            let (_k, v) = item
                .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

            if params.offset > 0 && skipped < params.offset {
                skipped += 1;
                continue;
            }

            let output: TaskOutput = serde_json::from_slice(&v)
                .map_err(|e| crate::error::Error::Json(e))?;
            outputs.push(output);
            count += 1;

            if let Some(limit) = params.count {
                if count >= limit {
                    break;
                }
            }
        }

        Ok(outputs)
    }

    /// Создаёт стадию задачи
    pub async fn create_task_stage(&self, stage: TaskStage) -> Result<TaskStage> {
        self.create_object(stage.task_id, "task_stages", &stage).await?;
        Ok(stage)
    }

    /// Получает стадии задачи
    pub async fn get_task_stages(&self, project_id: i32, task_id: i32) -> Result<Vec<TaskStageWithResult>> {
        // Проверяем существование задачи
        self.get_task(project_id, task_id).await?;

        let stages = self.get_objects::<TaskStage>(task_id, "task_stages", RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;

        // Конвертируем TaskStage в TaskStageWithResult
        let result = stages.iter().map(|stage| {
            TaskStageWithResult {
                stage: stage.clone(),
                start_output: None,
                end_output: None,
            }
        }).collect();

        Ok(result)
    }

    /// Завершает стадию задачи
    pub async fn end_task_stage(&self, task_id: i32, stage_id: i32, end: chrono::DateTime<Utc>) -> Result<()> {
        let stages = self.get_objects::<TaskStage>(task_id, "task_stages", RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;

        for mut stage in stages {
            if stage.id == stage_id {
                stage.end = Some(end);
                return self.update_object(task_id, "task_stages", stage).await;
            }
        }

        Err(crate::error::Error::NotFound("Стадия не найдена".to_string()))
    }

    /// Создаёт результат стадии задачи
    pub async fn create_task_stage_result(&self, task_id: i32, stage_id: i32, result: serde_json::Value) -> Result<()> {
        // TODO: Реализовать сохранение результата стадии
        Ok(())
    }

    /// Получает результат стадии задачи
    pub async fn get_task_stage_result(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<serde_json::Value> {
        // TODO: Реализовать получение результата стадии
        Ok(serde_json::json!({}))
    }

    /// Получает выводы стадии задачи
    pub async fn get_task_stage_outputs(&self, project_id: i32, task_id: i32, stage_id: i32) -> Result<Vec<TaskOutput>> {
        // TODO: Реализовать фильтрацию по stage_id
        self.get_task_outputs(project_id, task_id, RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await
    }

    /// Вспомогательный метод для получения задач
    async fn get_tasks_internal(&self, project_id: i32, template_id: Option<i32>, params: RetrieveQueryParams) -> Result<Vec<TaskWithTpl>> {
        let mut tasks_with_tpl = Vec::new();

        // Сначала получаем все задачи из БД
        let tree = self.db.open_tree(b"tasks")
            .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let mut tasks = Vec::new();
        let mut count = 0;
        let mut skipped = 0;

        for item in tree.iter() {
            let (_k, v) = item
                .map_err(|e| crate::error::Error::Database(sqlx::Error::Protocol(e.to_string())))?;

            if params.offset > 0 && skipped < params.offset {
                skipped += 1;
                continue;
            }

            let task: Task = serde_json::from_slice(&v)
                .map_err(|e| crate::error::Error::Json(e))?;

            if task.project_id != project_id {
                continue;
            }

            if let Some(tid) = template_id {
                if task.template_id != tid {
                    continue;
                }
            }

            tasks.push(task);
            count += 1;

            if let Some(limit) = params.count {
                if count >= limit {
                    break;
                }
            }
        }

        // Затем получаем информацию о шаблонах
        for task in tasks {
            let template = self.get_template(project_id, task.template_id).await;
            
            let (tpl_playbook, tpl_alias) = match template {
                Ok(tpl) => (tpl.playbook, tpl.name),
                Err(_) => (String::new(), String::new()),
            };

            let task_with_tpl = TaskWithTpl {
                task,
                tpl_playbook,
                tpl_alias,
                tpl_type: None,
                tpl_app: None,
                user_name: None,
                build_task: None,
            };

            tasks_with_tpl.push(task_with_tpl);
        }

        Ok(tasks_with_tpl)
    }

    /// Очищает старые задачи
    async fn clear_tasks(&self, project_id: i32, template_id: i32, max_tasks: i32) -> Result<()> {
        // Получаем количество задач
        let tasks = self.get_template_tasks(project_id, template_id, RetrieveQueryParams {
            offset: 0,
            count: Some(10000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;

        if tasks.len() <= max_tasks as usize {
            return Ok(());
        }

        // Удаляем старые задачи
        let to_delete = tasks.len() - max_tasks as usize;
        for task in tasks.iter().take(to_delete) {
            self.delete_task_with_outputs(project_id, task.task.id).await?;
        }

        Ok(())
    }

    /// Получает количество узлов (для статистики)
    pub async fn get_node_count(&self) -> Result<usize> {
        // TODO: Реализовать подсчёт узлов
        Ok(0)
    }

    /// Получает количество UI элементов (для статистики)
    pub async fn get_ui_count(&self) -> Result<usize> {
        // TODO: Реализовать подсчёт UI элементов
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_tasks.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_task(project_id: i32, template_id: i32) -> Task {
        Task {
            id: 0,
            created: Utc::now(),
            project_id,
            template_id,
            status: crate::models::TaskStatus::Waiting,
            message: "Test task".to_string(),
            commit_hash: None,
            commit_message: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: String::new(),
            end: None,
        }
    }

    #[tokio::test]
    async fn test_create_task() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);

        let result = db.create_task(task, 0).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created = db.create_task(task, 0).await.unwrap();

        let retrieved = db.get_task(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_get_template_tasks() {
        let db = create_test_bolt_db();

        // Создаём несколько задач для шаблона
        for i in 0..5 {
            let task = create_test_task(1, 1);
            db.create_task(task, 0).await.unwrap();
        }

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };

        let tasks = db.get_template_tasks(1, 1, params).await;
        assert!(tasks.is_ok());
        assert!(tasks.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_task() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let mut created = db.create_task(task, 0).await.unwrap();

        created.message = "Updated message".to_string();
        let result = db.update_task(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_task_with_outputs() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created = db.create_task(task, 0).await.unwrap();

        let result = db.delete_task_with_outputs(1, created.id).await;
        assert!(result.is_ok());

        let retrieved = db.get_task(1, created.id).await;
        assert!(retrieved.is_err());
    }

    #[tokio::test]
    async fn test_create_task_output() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created_task = db.create_task(task, 0).await.unwrap();

        let output = TaskOutput {
            id: 0,
            task_id: created_task.id,
            output: "Test output".to_string(),
            time: Utc::now(),
        };

        let result = db.create_task_output(output).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task_outputs() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created_task = db.create_task(task, 0).await.unwrap();

        // Создаём несколько выводов
        for i in 0..5 {
            let output = TaskOutput {
                id: 0,
                task_id: created_task.id,
                output: format!("Output {}", i),
                time: Utc::now(),
            };
            db.create_task_output(output).await.unwrap();
        }

        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };

        let outputs = db.get_task_outputs(1, created_task.id, params).await;
        assert!(outputs.is_ok());
        assert!(outputs.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_create_task_stage() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created_task = db.create_task(task, 0).await.unwrap();

        let stage = TaskStage {
            id: 0,
            task_id: created_task.id,
            start: Utc::now(),
            end: None,
            r#type: crate::models::TaskStageType::InstallRoles,
        };

        let result = db.create_task_stage(stage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_task_stages() {
        let db = create_test_bolt_db();
        let task = create_test_task(1, 1);
        let created_task = db.create_task(task, 0).await.unwrap();

        // Создаём несколько стадий
        for _ in 0..3 {
            let stage = TaskStage {
                id: 0,
                task_id: created_task.id,
                start: Utc::now(),
                end: None,
                r#type: crate::models::TaskStageType::InstallRoles,
            };
            db.create_task_stage(stage).await.unwrap();
        }

        let stages = db.get_task_stages(1, created_task.id).await;
        assert!(stages.is_ok());
        assert!(stages.unwrap().len() >= 3);
    }
}
