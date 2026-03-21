use crate::client::VelumClient;
use crate::protocol::{prop_bool, prop_int, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_schedules".into(),
            description: "List all cron schedules for a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_schedule".into(),
            description: "Get details of a specific cron schedule.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "schedule_id": prop_int("Schedule ID")
                },
                "required": ["project_id","schedule_id"]
            }),
        },
        Tool {
            name: "create_schedule".into(),
            description: "Create a new cron schedule for a template. Examples: '0 3 * * *' = daily 3AM, '*/15 * * * *' = every 15min, '0 0 * * MON' = weekly Monday.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template to run on schedule"),
                    "cron": prop_str("Cron expression, e.g. '0 3 * * *'"),
                    "name": prop_str_opt("Human-readable schedule name"),
                    "active": prop_bool("Enable immediately (default true)")
                },
                "required": ["project_id","template_id","cron"]
            }),
        },
        Tool {
            name: "toggle_schedule".into(),
            description: "Enable or disable a cron schedule without deleting it.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "schedule_id": prop_int("Schedule ID"),
                    "active": prop_bool("true to enable, false to disable")
                },
                "required": ["project_id","schedule_id","active"]
            }),
        },
        Tool {
            name: "delete_schedule".into(),
            description: "Permanently delete a cron schedule.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "schedule_id": prop_int("Schedule ID to delete")
                },
                "required": ["project_id","schedule_id"]
            }),
        },
        Tool {
            name: "validate_cron".into(),
            description: "Validate a cron expression and check if it parses correctly.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Any project ID (needed for API route)"),
                    "cron": prop_str("Cron expression to validate, e.g. '0 3 * * MON-FRI'")
                },
                "required": ["project_id","cron"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args.get("project_id").and_then(Value::as_i64).unwrap_or_default();
    match name {
        "list_schedules" => {
            let v = client.get(&format!("/projects/{pid}/schedules")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_schedule" => {
            let sid = args["schedule_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/schedules/{sid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_schedule" => {
            let body = json!({
                "project_id": pid,
                "template_id": args["template_id"],
                "cron": args["cron"],
                "name": args.get("name").and_then(Value::as_str)
                    .unwrap_or(&format!("Schedule for template {}", args["template_id"])),
                "active": args.get("active").and_then(Value::as_bool).unwrap_or(true)
            });
            let v = client.post(&format!("/projects/{pid}/schedules"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "toggle_schedule" => {
            let sid = args["schedule_id"].as_i64().unwrap_or_default();
            let active = args["active"].as_bool().unwrap_or(true);
            client.put(&format!("/project/{pid}/schedules/{sid}/active"), &json!({"active": active})).await?;
            let state = if active { "enabled" } else { "disabled" };
            Ok(ToolResult::text(format!("Schedule {sid} {state}.")))
        }
        "delete_schedule" => {
            let sid = args["schedule_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/schedules/{sid}")).await?;
            Ok(ToolResult::text(format!("Schedule {sid} deleted.")))
        }
        "validate_cron" => {
            let cron = args["cron"].as_str().unwrap_or_default();
            let body = json!({ "cron": cron });
            match client.post(&format!("/projects/{pid}/schedules/validate"), &body).await {
                Ok(v) => Ok(ToolResult::json(&v)),
                Err(e) => Ok(ToolResult::text(format!("Invalid cron expression '{cron}': {e}"))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
