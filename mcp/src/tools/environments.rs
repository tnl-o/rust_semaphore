use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_environments".into(),
            description: "List all environment configurations for a project. Environments hold Ansible extra-vars and secrets.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_environment".into(),
            description: "Get details of a specific environment including its variable JSON.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "environment_id": prop_int("Environment ID")
                },
                "required": ["project_id","environment_id"]
            }),
        },
        Tool {
            name: "create_environment".into(),
            description: "Create a new environment with a JSON blob of extra variables. Example json: '{\"DEPLOY_ENV\":\"prod\",\"MAX_RETRIES\":\"3\"}'.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "name": prop_str("Environment name"),
                    "json": prop_str_opt("JSON object of extra-vars (e.g. '{\"KEY\":\"VALUE\"}')"),
                    "env": prop_str_opt("Shell environment variables as JSON object")
                },
                "required": ["project_id","name"]
            }),
        },
        Tool {
            name: "update_environment".into(),
            description: "Update an existing environment name or variable JSON.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "environment_id": prop_int("Environment ID"),
                    "name": prop_str_opt("New name"),
                    "json": prop_str_opt("New JSON extra-vars"),
                    "env": prop_str_opt("New shell env vars JSON")
                },
                "required": ["project_id","environment_id"]
            }),
        },
        Tool {
            name: "delete_environment".into(),
            description: "Delete an environment from a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "environment_id": prop_int("Environment ID to delete")
                },
                "required": ["project_id","environment_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args["project_id"].as_i64().unwrap_or_default();
    match name {
        "list_environments" => {
            let v = client.get(&format!("/projects/{pid}/environment")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_environment" => {
            let eid = args["environment_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/environment/{eid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_environment" => {
            let body = json!({
                "project_id": pid,
                "name": args["name"],
                "json": args.get("json").and_then(Value::as_str).unwrap_or("{}"),
                "env": args.get("env").and_then(Value::as_str).unwrap_or("{}")
            });
            let v = client.post(&format!("/projects/{pid}/environment"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "update_environment" => {
            let eid = args["environment_id"].as_i64().unwrap_or_default();
            let existing = client.get(&format!("/projects/{pid}/environment/{eid}")).await?;
            let mut body = existing.clone();
            if let Some(n) = args.get("name").and_then(Value::as_str) { body["name"] = json!(n); }
            if let Some(j) = args.get("json").and_then(Value::as_str) { body["json"] = json!(j); }
            if let Some(e) = args.get("env").and_then(Value::as_str) { body["env"] = json!(e); }
            let v = client.put(&format!("/projects/{pid}/environment/{eid}"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "delete_environment" => {
            let eid = args["environment_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/environment/{eid}")).await?;
            Ok(ToolResult::text(format!("Environment {eid} deleted.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
