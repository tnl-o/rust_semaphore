use crate::client::VelumClient;
use crate::protocol::{prop_bool, prop_int, prop_int_opt, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_tasks".into(),
            description: "List recent tasks in a project, optionally filtered by status.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "limit": prop_int_opt("Max tasks to return (default 20)"),
                    "status": { "type": ["string","null"], "enum": ["waiting","running","success","error","stopped",null], "description": "Filter by status" }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_task".into(),
            description: "Get full details of a task including status, timing, and metadata.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("Task ID")
                },
                "required": ["project_id","task_id"]
            }),
        },
        Tool {
            name: "run_task".into(),
            description: "Run a task from a template. Returns task object with ID for status tracking.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template to run"),
                    "message": prop_str_opt("Human note for this run"),
                    "git_branch": prop_str_opt("Override branch/tag/commit"),
                    "arguments": prop_str_opt("Extra variables override (JSON string)"),
                    "inventory_id": prop_int_opt("Override inventory ID")
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "stop_task".into(),
            description: "Send a stop signal to a running task.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("Task to stop")
                },
                "required": ["project_id","task_id"]
            }),
        },
        Tool {
            name: "get_task_output".into(),
            description: "Retrieve console output of a task. Optionally return only the last N lines.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("Task ID"),
                    "raw": prop_bool("Return raw unformatted text (default: false)"),
                    "last_n_lines": prop_int_opt("Return only the last N lines")
                },
                "required": ["project_id","task_id"]
            }),
        },
        Tool {
            name: "filter_tasks".into(),
            description: "Filter project tasks by template and/or execution status.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int_opt("Filter to a specific template"),
                    "status": { "type": ["string","null"], "enum": ["waiting","running","success","error","stopped",null] },
                    "limit": prop_int_opt("Max results (default 50)")
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_latest_failed_task".into(),
            description: "Return the most recent failed task in a project for quick error inspection.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_waiting_tasks".into(),
            description: "Return all tasks currently waiting in the execution queue.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "bulk_stop_tasks".into(),
            description: "Stop multiple tasks at once by providing a list of task IDs.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_ids": { "type": "array", "items": { "type": "integer" }, "description": "List of task IDs to stop" }
                },
                "required": ["project_id","task_ids"]
            }),
        },
        Tool {
            name: "confirm_task".into(),
            description: "Confirm (approve) a task waiting for human approval (gated execution).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("Task to approve")
                },
                "required": ["project_id","task_id"]
            }),
        },
        Tool {
            name: "reject_task".into(),
            description: "Reject a task waiting for human approval — it will be stopped.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("Task to reject")
                },
                "required": ["project_id","task_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args.get("project_id").and_then(Value::as_i64).unwrap_or_default();
    match name {
        "list_tasks" => {
            let all = client.get(&format!("/projects/{pid}/tasks")).await?;
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
            let status_filter = args.get("status").and_then(Value::as_str);
            let tasks = all.as_array().map(|a| {
                a.iter()
                    .filter(|t| {
                        status_filter
                            .map(|s| t["status"].as_str() == Some(s))
                            .unwrap_or(true)
                    })
                    .take(limit)
                    .cloned()
                    .collect::<Vec<_>>()
            });
            Ok(ToolResult::json(&json!(tasks)))
        }
        "get_task" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/tasks/{tid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "run_task" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let mut body = json!({ "template_id": tid });
            if let Some(m) = args.get("message").and_then(Value::as_str) { body["message"] = json!(m); }
            if let Some(b) = args.get("git_branch").and_then(Value::as_str) { body["git_branch"] = json!(b); }
            if let Some(a) = args.get("arguments").and_then(Value::as_str) { body["arguments"] = json!(a); }
            if let Some(i) = args.get("inventory_id").and_then(Value::as_i64) { body["inventory_id"] = json!(i); }
            let v = client.post(&format!("/project/{pid}/tasks"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "stop_task" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            client.post_empty(&format!("/projects/{pid}/tasks/{tid}/stop")).await?;
            Ok(ToolResult::text(format!("Stop signal sent to task {tid}.")))
        }
        "get_task_output" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            let raw = args.get("raw").and_then(Value::as_bool).unwrap_or(false);
            let path = if raw {
                format!("/projects/{pid}/tasks/{tid}/raw_output")
            } else {
                format!("/projects/{pid}/tasks/{tid}/output")
            };
            let v = client.get(&path).await?;
            let lines: Vec<String> = match &v {
                Value::String(s) => s.lines().map(String::from).collect(),
                Value::Array(arr) => arr.iter()
                    .map(|r| r.get("output").and_then(Value::as_str).unwrap_or("").to_string())
                    .collect(),
                _ => vec![v.to_string()],
            };
            let n = args.get("last_n_lines").and_then(Value::as_u64).map(|n| n as usize);
            let output = match n {
                Some(n) => lines.iter().rev().take(n).rev().cloned().collect::<Vec<_>>().join("\n"),
                None => lines.join("\n"),
            };
            Ok(ToolResult::text(output))
        }
        "filter_tasks" => {
            let all = client.get(&format!("/projects/{pid}/tasks")).await?;
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(50) as usize;
            let tpl_filter = args.get("template_id").and_then(Value::as_i64);
            let status_filter = args.get("status").and_then(Value::as_str);
            let tasks: Vec<Value> = all.as_array().map(|a| {
                a.iter()
                    .filter(|t| {
                        tpl_filter.map(|id| t["template_id"].as_i64() == Some(id)).unwrap_or(true)
                            && status_filter.map(|s| t["status"].as_str() == Some(s)).unwrap_or(true)
                    })
                    .take(limit)
                    .cloned()
                    .collect()
            }).unwrap_or_default();
            Ok(ToolResult::json(&json!(tasks)))
        }
        "get_latest_failed_task" => {
            let all = client.get(&format!("/projects/{pid}/tasks")).await?;
            let failed = all.as_array().and_then(|a| {
                a.iter()
                    .filter(|t| t["status"].as_str() == Some("error"))
                    .max_by_key(|t| t["id"].as_i64().unwrap_or(0))
                    .cloned()
            });
            match failed {
                Some(t) => Ok(ToolResult::json(&t)),
                None => Ok(ToolResult::text("No failed tasks found in this project.")),
            }
        }
        "get_waiting_tasks" => {
            let all = client.get(&format!("/projects/{pid}/tasks")).await?;
            let waiting: Vec<Value> = all.as_array().map(|a| {
                a.iter().filter(|t| t["status"].as_str() == Some("waiting")).cloned().collect()
            }).unwrap_or_default();
            Ok(ToolResult::json(&json!(waiting)))
        }
        "bulk_stop_tasks" => {
            let ids = args["task_ids"].as_array().cloned().unwrap_or_default();
            let mut results = Vec::new();
            for id_val in &ids {
                if let Some(tid) = id_val.as_i64() {
                    match client.post_empty(&format!("/projects/{pid}/tasks/{tid}/stop")).await {
                        Ok(_) => results.push(format!("✓ {tid}")),
                        Err(e) => results.push(format!("✗ {tid}: {e}")),
                    }
                }
            }
            Ok(ToolResult::text(format!("Stop results:\n{}", results.join("\n"))))
        }
        "confirm_task" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            client.post_empty(&format!("/projects/{pid}/tasks/{tid}/confirm")).await?;
            Ok(ToolResult::text(format!("Task {tid} confirmed and will proceed.")))
        }
        "reject_task" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            client.post_empty(&format!("/projects/{pid}/tasks/{tid}/reject")).await?;
            Ok(ToolResult::text(format!("Task {tid} rejected and stopped.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
