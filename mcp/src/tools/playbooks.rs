use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_int_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_playbooks".into(),
            description: "List all playbooks discovered in the Git repositories of a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_playbook".into(),
            description: "Get details of a specific playbook.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "playbook_id": prop_int("Playbook ID")
                },
                "required": ["project_id","playbook_id"]
            }),
        },
        Tool {
            name: "sync_repository".into(),
            description: "Trigger a git pull/sync on a repository to refresh playbook discovery.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "repository_id": prop_int("Repository ID to sync")
                },
                "required": ["project_id","repository_id"]
            }),
        },
        Tool {
            name: "run_playbook".into(),
            description: "Run a playbook by creating and queuing a task for it. Requires a template configured for this playbook.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID associated with this playbook"),
                    "message": { "type": "string", "description": "Optional run message/description" }
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "get_playbook_history".into(),
            description: "Get recent task execution history for a specific template/playbook.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID"),
                    "limit": prop_int_opt("Number of history entries to return (default 20)")
                },
                "required": ["project_id","template_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args["project_id"].as_i64().unwrap_or_default();
    match name {
        "list_playbooks" => {
            // Playbooks are discovered from repositories — list templates and filter by type
            let v = client.get(&format!("/projects/{pid}/templates")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_playbook" => {
            let pbid = args["playbook_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/templates/{pbid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "sync_repository" => {
            let rid = args["repository_id"].as_i64().unwrap_or_default();
            match client.post_empty(&format!("/projects/{pid}/repositories/{rid}/sync")).await {
                Ok(v) => Ok(ToolResult::json(&v)),
                Err(e) => Ok(ToolResult::text(format!("Sync triggered for repository {rid} (or already in progress): {e}"))),
            }
        }
        "run_playbook" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let message = args.get("message").and_then(Value::as_str).unwrap_or("");
            let body = json!({
                "template_id": tid,
                "project_id": pid,
                "message": message,
                "debug": false,
                "dry_run": false,
                "diff": false
            });
            let v = client.post(&format!("/projects/{pid}/tasks"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "get_playbook_history" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            let all_tasks = client.get(&format!("/projects/{pid}/tasks")).await?;
            let history: Vec<Value> = all_tasks.as_array().map(|a| {
                a.iter()
                    .filter(|t| t["template_id"].as_i64() == Some(tid))
                    .take(limit)
                    .cloned()
                    .collect()
            }).unwrap_or_default();
            Ok(ToolResult::json(&json!(history)))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
