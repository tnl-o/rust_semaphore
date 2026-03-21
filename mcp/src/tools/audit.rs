use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_int_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_audit_log".into(),
            description: "Retrieve the audit log showing who did what and when. Each entry shows user, action, object type, and timestamp.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int_opt("Filter to a specific project (omit for global admin log)"),
                    "limit": prop_int_opt("Max entries (default 50)")
                },
                "required": []
            }),
        },
        Tool {
            name: "get_project_events".into(),
            description: "Get recent events for a project: task runs, config changes, user actions.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "limit": prop_int_opt("Max events (default 30)")
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_system_info".into(),
            description: "Get Velum system information: version, uptime, database type, runner count, build info.".into(),
            input_schema: json!({ "type": "object", "properties": {}, "required": [] }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    match name {
        "get_audit_log" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
            let pid = args.get("project_id").and_then(Value::as_i64);
            let v = if let Some(pid) = pid {
                client.get(&format!("/project/{pid}/audit-log")).await?
            } else {
                client.get("/audit-log").await?
            };
            let entries: Vec<Value> = v.as_array().map(|a| a.iter().take(limit).cloned().collect()).unwrap_or_default();
            Ok(ToolResult::json(&json!(entries)))
        }
        "get_project_events" => {
            let pid = args["project_id"].as_i64().unwrap_or_default();
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(30) as usize;
            let v = client.get(&format!("/project/{pid}/events")).await?;
            let events: Vec<Value> = v.as_array().map(|a| a.iter().take(limit).cloned().collect()).unwrap_or_default();
            Ok(ToolResult::json(&json!(events)))
        }
        "get_system_info" => {
            let v = client.get("/info").await?;
            Ok(ToolResult::json(&v))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
