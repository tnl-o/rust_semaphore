use crate::client::VelumClient;
use crate::protocol::{prop_bool, prop_int, prop_int_opt, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects".into(),
            description: "List all Velum projects the API token has access to.".into(),
            input_schema: json!({ "type": "object", "properties": {}, "required": [] }),
        },
        Tool {
            name: "get_project".into(),
            description: "Get full details of a specific project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "create_project".into(),
            description: "Create a new Velum project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": prop_str("Project display name"),
                    "alert": prop_bool("Enable task failure alerts"),
                    "alert_chat": prop_str_opt("Chat ID for Telegram alerts"),
                    "max_parallel_tasks": prop_int("Max simultaneous tasks (0 = unlimited)")
                },
                "required": ["name"]
            }),
        },
        Tool {
            name: "update_project".into(),
            description: "Update an existing project's settings.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "name": prop_str_opt("New display name"),
                    "alert": { "type": ["boolean","null"], "description": "Toggle alerts" },
                    "max_parallel_tasks": prop_int_opt("Max simultaneous tasks")
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "delete_project".into(),
            description: "Delete a project and all its resources. Irreversible.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID to delete") },
                "required": ["project_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    match name {
        "list_projects" => {
            let v = client.get("/projects").await?;
            Ok(ToolResult::json(&v))
        }
        "get_project" => {
            let id = args["project_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{id}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_project" => {
            let body = json!({
                "name": args["name"],
                "alert": args.get("alert").and_then(Value::as_bool).unwrap_or(false),
                "alert_chat": args.get("alert_chat").and_then(Value::as_str).unwrap_or(""),
                "max_parallel_tasks": args.get("max_parallel_tasks").and_then(Value::as_i64).unwrap_or(0)
            });
            let v = client.post("/projects", &body).await?;
            Ok(ToolResult::json(&v))
        }
        "update_project" => {
            let id = args["project_id"].as_i64().unwrap_or_default();
            let mut current = client.get(&format!("/projects/{id}")).await?;
            if let Some(name) = args.get("name").and_then(Value::as_str) {
                current["name"] = json!(name);
            }
            if let Some(alert) = args.get("alert").and_then(Value::as_bool) {
                current["alert"] = json!(alert);
            }
            if let Some(mpt) = args.get("max_parallel_tasks").and_then(Value::as_i64) {
                current["max_parallel_tasks"] = json!(mpt);
            }
            let v = client.put(&format!("/projects/{id}"), &current).await?;
            Ok(ToolResult::json(&v))
        }
        "delete_project" => {
            let id = args["project_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{id}")).await?;
            Ok(ToolResult::text(format!("Project {id} deleted successfully.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
