use crate::client::VelumClient;
use crate::protocol::{prop_bool, prop_int, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_templates".into(),
            description: "List all job templates in a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_template".into(),
            description: "Get full details of a specific job template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID")
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "create_template".into(),
            description: "Create a new job template (Ansible, Terraform, Bash, etc.)".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "name": prop_str("Template display name"),
                    "playbook": prop_str("Playbook path or Terraform directory"),
                    "inventory_id": prop_int("Inventory ID"),
                    "repository_id": prop_int("Repository ID"),
                    "environment_id": prop_int("Environment ID"),
                    "app": { "type": "string", "enum": ["ansible","terraform","tofu","terragrunt","bash"], "description": "Executor type" },
                    "description": prop_str_opt("Human-readable description"),
                    "arguments": prop_str_opt("Extra variables (JSON/YAML)"),
                    "allow_override_args_in_task": prop_bool("Allow argument overrides at run time"),
                    "allow_override_branch_in_task": prop_bool("Allow branch override at run time"),
                    "suppress_success_alerts": prop_bool("Do not alert on success")
                },
                "required": ["project_id","name","playbook","inventory_id","repository_id","environment_id"]
            }),
        },
        Tool {
            name: "update_template".into(),
            description: "Update selected fields of an existing template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID"),
                    "name": prop_str_opt("New name"),
                    "description": prop_str_opt("New description"),
                    "arguments": prop_str_opt("New extra_vars")
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "delete_template".into(),
            description: "Delete a job template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID to delete")
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "run_template".into(),
            description: "Run a job template immediately — creates and queues a new task. Returns the task object with its ID.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template to run"),
                    "message": prop_str_opt("Human note for this run (shows in history)"),
                    "git_branch": prop_str_opt("Override Git branch/tag/commit"),
                    "arguments": prop_str_opt("Override extra_vars (JSON string)")
                },
                "required": ["project_id","template_id"]
            }),
        },
        Tool {
            name: "stop_all_template_tasks".into(),
            description: "Stop all currently running tasks for a specific template.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "template_id": prop_int("Template ID")
                },
                "required": ["project_id","template_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args.get("project_id").and_then(Value::as_i64).unwrap_or_default();
    match name {
        "list_templates" => {
            let v = client.get(&format!("/projects/{pid}/templates")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_template" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/templates/{tid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_template" => {
            let body = json!({
                "project_id": pid,
                "name": args["name"],
                "playbook": args["playbook"],
                "inventory_id": args["inventory_id"],
                "repository_id": args["repository_id"],
                "environment_id": args["environment_id"],
                "app": args.get("app").and_then(Value::as_str).unwrap_or("ansible"),
                "description": args.get("description").and_then(Value::as_str).unwrap_or(""),
                "arguments": args.get("arguments").and_then(Value::as_str).unwrap_or(""),
                "allow_override_args_in_task": args.get("allow_override_args_in_task").and_then(Value::as_bool).unwrap_or(false),
                "allow_override_branch_in_task": args.get("allow_override_branch_in_task").and_then(Value::as_bool).unwrap_or(false),
                "suppress_success_alerts": args.get("suppress_success_alerts").and_then(Value::as_bool).unwrap_or(false)
            });
            let v = client.post(&format!("/projects/{pid}/templates"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "update_template" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let mut current = client.get(&format!("/projects/{pid}/templates/{tid}")).await?;
            if let Some(v) = args.get("name").and_then(Value::as_str) { current["name"] = json!(v); }
            if let Some(v) = args.get("description").and_then(Value::as_str) { current["description"] = json!(v); }
            if let Some(v) = args.get("arguments").and_then(Value::as_str) { current["arguments"] = json!(v); }
            let v = client.put(&format!("/projects/{pid}/templates/{tid}"), &current).await?;
            Ok(ToolResult::json(&v))
        }
        "delete_template" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/templates/{tid}")).await?;
            Ok(ToolResult::text(format!("Template {tid} deleted from project {pid}.")))
        }
        "run_template" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            let mut body = json!({ "template_id": tid });
            if let Some(m) = args.get("message").and_then(Value::as_str) { body["message"] = json!(m); }
            if let Some(b) = args.get("git_branch").and_then(Value::as_str) { body["git_branch"] = json!(b); }
            if let Some(a) = args.get("arguments").and_then(Value::as_str) { body["arguments"] = json!(a); }
            let v = client.post(&format!("/project/{pid}/tasks"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "stop_all_template_tasks" => {
            let tid = args["template_id"].as_i64().unwrap_or_default();
            client.post_empty(&format!("/projects/{pid}/templates/{tid}/stop_all_tasks")).await?;
            Ok(ToolResult::text(format!("Stop signal sent to all tasks of template {tid}.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
