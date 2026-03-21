use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_access_keys".into(),
            description: "List all access keys (SSH keys, API tokens, passwords) for a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_access_key".into(),
            description: "Get metadata for a specific access key. Note: secret values are never returned.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "key_id": prop_int("Key ID")
                },
                "required": ["project_id","key_id"]
            }),
        },
        Tool {
            name: "create_access_key".into(),
            description: "Create an access key. Types: 'ssh' (provide private key), 'login_password' (provide login+password), 'none' (no auth).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "name": prop_str("Display name for the key"),
                    "type": {
                        "type": "string",
                        "enum": ["none","ssh","login_password"],
                        "description": "Key type"
                    },
                    "ssh_private_key": prop_str_opt("SSH private key contents (for type=ssh)"),
                    "login": prop_str_opt("Username/login (for type=login_password)"),
                    "password": prop_str_opt("Password (for type=login_password)")
                },
                "required": ["project_id","name","type"]
            }),
        },
        Tool {
            name: "delete_access_key".into(),
            description: "Delete an access key from a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "key_id": prop_int("Key ID to delete")
                },
                "required": ["project_id","key_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args["project_id"].as_i64().unwrap_or_default();
    match name {
        "list_access_keys" => {
            let v = client.get(&format!("/projects/{pid}/keys")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_access_key" => {
            let kid = args["key_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/keys/{kid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_access_key" => {
            let key_type = args["type"].as_str().unwrap_or("none");
            let mut body = json!({
                "project_id": pid,
                "name": args["name"],
                "type": key_type
            });
            match key_type {
                "ssh" => {
                    body["ssh"] = json!({
                        "private_key": args.get("ssh_private_key").and_then(Value::as_str).unwrap_or("")
                    });
                }
                "login_password" => {
                    body["login_password"] = json!({
                        "login": args.get("login").and_then(Value::as_str).unwrap_or(""),
                        "password": args.get("password").and_then(Value::as_str).unwrap_or("")
                    });
                }
                _ => {}
            }
            let v = client.post(&format!("/projects/{pid}/keys"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "delete_access_key" => {
            let kid = args["key_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/keys/{kid}")).await?;
            Ok(ToolResult::text(format!("Access key {kid} deleted.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
