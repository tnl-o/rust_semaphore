pub mod ai_analyzer;
pub mod analytics;
pub mod audit;
pub mod environments;
pub mod inventory;
pub mod keys;
pub mod playbooks;
pub mod projects;
pub mod repositories;
pub mod runners;
pub mod schedules;
pub mod tasks;
pub mod templates;

use crate::client::VelumClient;
use crate::protocol::{Tool, ToolResult};
use anyhow::Result;
use serde_json::Value;

/// Collect all tool definitions from every module.
pub fn all_definitions() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(projects::definitions());
    tools.extend(templates::definitions());
    tools.extend(tasks::definitions());
    tools.extend(schedules::definitions());
    tools.extend(repositories::definitions());
    tools.extend(environments::definitions());
    tools.extend(keys::definitions());
    tools.extend(inventory::definitions());
    tools.extend(runners::definitions());
    tools.extend(analytics::definitions());
    tools.extend(audit::definitions());
    tools.extend(playbooks::definitions());
    tools.extend(ai_analyzer::definitions());
    tools
}

/// Dispatch a tool call to the correct module.
pub async fn dispatch(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    // Projects
    if matches!(
        name,
        "list_projects" | "get_project" | "create_project" | "update_project" | "delete_project"
    ) {
        return projects::call(name, args, client).await;
    }

    // Templates
    if matches!(
        name,
        "list_templates"
            | "get_template"
            | "create_template"
            | "update_template"
            | "delete_template"
            | "run_template"
            | "stop_all_template_tasks"
    ) {
        return templates::call(name, args, client).await;
    }

    // Tasks
    if matches!(
        name,
        "list_tasks"
            | "get_task"
            | "run_task"
            | "stop_task"
            | "get_task_output"
            | "filter_tasks"
            | "get_latest_failed_task"
            | "get_waiting_tasks"
            | "bulk_stop_tasks"
            | "confirm_task"
            | "reject_task"
    ) {
        return tasks::call(name, args, client).await;
    }

    // Schedules
    if matches!(
        name,
        "list_schedules"
            | "get_schedule"
            | "create_schedule"
            | "toggle_schedule"
            | "delete_schedule"
            | "validate_cron"
    ) {
        return schedules::call(name, args, client).await;
    }

    // Repositories
    if matches!(
        name,
        "list_repositories"
            | "get_repository"
            | "create_repository"
            | "update_repository"
            | "delete_repository"
            | "list_repository_branches"
    ) {
        return repositories::call(name, args, client).await;
    }

    // Environments
    if matches!(
        name,
        "list_environments"
            | "get_environment"
            | "create_environment"
            | "update_environment"
            | "delete_environment"
    ) {
        return environments::call(name, args, client).await;
    }

    // Keys
    if matches!(
        name,
        "list_access_keys" | "get_access_key" | "create_access_key" | "delete_access_key"
    ) {
        return keys::call(name, args, client).await;
    }

    // Inventory
    if matches!(
        name,
        "list_inventory"
            | "get_inventory"
            | "create_inventory"
            | "update_inventory"
            | "delete_inventory"
    ) {
        return inventory::call(name, args, client).await;
    }

    // Runners
    if matches!(
        name,
        "list_runners" | "get_runner_status" | "toggle_runner" | "clear_runner_cache"
    ) {
        return runners::call(name, args, client).await;
    }

    // Analytics
    if matches!(
        name,
        "get_project_analytics"
            | "get_task_trends"
            | "get_system_analytics"
            | "get_project_health"
    ) {
        return analytics::call(name, args, client).await;
    }

    // Audit / System
    if matches!(name, "get_audit_log" | "get_project_events" | "get_system_info") {
        return audit::call(name, args, client).await;
    }

    // Playbooks
    if matches!(
        name,
        "list_playbooks"
            | "get_playbook"
            | "sync_repository"
            | "run_playbook"
            | "get_playbook_history"
    ) {
        return playbooks::call(name, args, client).await;
    }

    // AI Analyzer
    if matches!(name, "analyze_task_failure" | "bulk_analyze_failures") {
        return ai_analyzer::call(name, args, client).await;
    }

    Ok(ToolResult::error(format!("Unknown tool: {name}")))
}
