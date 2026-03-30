//! GraphQL Subscription корень — real-time события (v5.1)
//!
//! ## Подписки
//! - `taskCreated` — новая задача создана
//! - `taskStatus(projectId?)` — изменение статуса задачи
//! - `taskOutput(taskId)` — строки лога выполняющейся задачи
//!
//! ## Публикация (из других модулей)
//! ```rust,ignore
//! use crate::api::graphql::subscription;
//! subscription::publish_task_created(task);
//! subscription::publish_task_status(event);
//! subscription::publish_task_output(line);
//! ```

use async_graphql::{Context, Subscription, Result};
use futures_util::stream::{Stream, StreamExt};
use once_cell::sync::Lazy;
use tokio::sync::broadcast;

use super::types::{Task, TaskOutputLine, TaskStatusEvent};

// ── Broadcast channels ───────────────────────────────────────────────────────

static TASK_CREATED_TX: Lazy<broadcast::Sender<Task>> =
    Lazy::new(|| broadcast::channel(256).0);

static TASK_STATUS_TX: Lazy<broadcast::Sender<TaskStatusEvent>> =
    Lazy::new(|| broadcast::channel(256).0);

static TASK_OUTPUT_TX: Lazy<broadcast::Sender<TaskOutputLine>> =
    Lazy::new(|| broadcast::channel(1024).0);

// ── Public publish helpers ───────────────────────────────────────────────────

pub fn publish_task_created(task: Task) {
    let _ = TASK_CREATED_TX.send(task);
}

pub fn publish_task_status(event: TaskStatusEvent) {
    let _ = TASK_STATUS_TX.send(event);
}

pub fn publish_task_output(line: TaskOutputLine) {
    let _ = TASK_OUTPUT_TX.send(line);
}

// ── SubscriptionRoot ─────────────────────────────────────────────────────────

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Подписка на создание задач во всех проектах.
    async fn task_created(&self, _ctx: &Context<'_>) -> Result<impl Stream<Item = Task>> {
        Ok(recv_broadcast(TASK_CREATED_TX.subscribe()))
    }

    /// Подписка на изменения статуса задач (опционально фильтрация по project_id).
    async fn task_status(
        &self,
        _ctx: &Context<'_>,
        project_id: Option<i32>,
    ) -> Result<impl Stream<Item = TaskStatusEvent>> {
        let stream = recv_broadcast(TASK_STATUS_TX.subscribe())
            .filter(move |ev| {
                let pass = project_id.map_or(true, |pid| ev.project_id == pid);
                async move { pass }
            });
        Ok(stream)
    }

    /// Подписка на строки лога задачи по task_id.
    async fn task_output(
        &self,
        _ctx: &Context<'_>,
        task_id: i32,
    ) -> Result<impl Stream<Item = TaskOutputLine>> {
        let stream = recv_broadcast(TASK_OUTPUT_TX.subscribe())
            .filter(move |line| {
                let pass = line.task_id == task_id;
                async move { pass }
            });
        Ok(stream)
    }
}

// ── Helper ───────────────────────────────────────────────────────────────────

fn recv_broadcast<T: Clone + Send + 'static>(
    mut rx: broadcast::Receiver<T>,
) -> impl Stream<Item = T> {
    async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(item) => yield item,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("GraphQL subscription lagged {n}");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    }
}
