use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_repositories".into(),
            description: "List all Git repositories configured for a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "project_id": prop_int("Project ID") },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_repository".into(),
            description: "Get details of a specific Git repository (URL, branch, SSH key, etc.).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "repository_id": prop_int("Repository ID")
                },
                "required": ["project_id","repository_id"]
            }),
        },
        Tool {
            name: "create_repository".into(),
            description: "Add a new Git repository to a project. Specify the clone URL and optional branch/SSH key.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "name": prop_str("Display name for the repository"),
                    "git_url": prop_str("Git clone URL (https:// or git@)"),
                    "git_branch": prop_str_opt("Branch to track (default: main)"),
                    "ssh_key_id": { "type": "integer", "description": "SSH key ID to use for authentication (optional)" }
                },
                "required": ["project_id","name","git_url"]
            }),
        },
        Tool {
            name: "update_repository".into(),
            description: "Update an existing Git repository configuration.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "repository_id": prop_int("Repository ID"),
                    "name": prop_str_opt("New display name"),
                    "git_url": prop_str_opt("New Git URL"),
                    "git_branch": prop_str_opt("New branch"),
                    "ssh_key_id": { "type": "integer", "description": "New SSH key ID (optional)" }
                },
                "required": ["project_id","repository_id"]
            }),
        },
        Tool {
            name: "delete_repository".into(),
            description: "Delete a Git repository from a project.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "repository_id": prop_int("Repository ID to delete")
                },
                "required": ["project_id","repository_id"]
            }),
        },
        Tool {
            name: "list_repository_branches".into(),
            description: "List available branches for a repository (fetched live from remote).".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "repository_id": prop_int("Repository ID")
                },
                "required": ["project_id","repository_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args["project_id"].as_i64().unwrap_or_default();
    match name {
        "list_repositories" => {
            let v = client.get(&format!("/projects/{pid}/repositories")).await?;
            Ok(ToolResult::json(&v))
        }
        "get_repository" => {
            let rid = args["repository_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/repositories/{rid}")).await?;
            Ok(ToolResult::json(&v))
        }
        "create_repository" => {
            let body = json!({
                "project_id": pid,
                "name": args["name"],
                "git_url": args["git_url"],
                "git_branch": args.get("git_branch").and_then(Value::as_str).unwrap_or("main"),
                "ssh_key_id": args.get("ssh_key_id").and_then(Value::as_i64).unwrap_or(0)
            });
            let v = client.post(&format!("/projects/{pid}/repositories"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "update_repository" => {
            let rid = args["repository_id"].as_i64().unwrap_or_default();
            let existing = client.get(&format!("/projects/{pid}/repositories/{rid}")).await?;
            let mut body = existing.clone();
            if let Some(n) = args.get("name").and_then(Value::as_str) { body["name"] = json!(n); }
            if let Some(u) = args.get("git_url").and_then(Value::as_str) { body["git_url"] = json!(u); }
            if let Some(b) = args.get("git_branch").and_then(Value::as_str) { body["git_branch"] = json!(b); }
            if let Some(k) = args.get("ssh_key_id").and_then(Value::as_i64) { body["ssh_key_id"] = json!(k); }
            let v = client.put(&format!("/projects/{pid}/repositories/{rid}"), &body).await?;
            Ok(ToolResult::json(&v))
        }
        "delete_repository" => {
            let rid = args["repository_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/repositories/{rid}")).await?;
            Ok(ToolResult::text(format!("Repository {rid} deleted.")))
        }
        "list_repository_branches" => {
            let rid = args["repository_id"].as_i64().unwrap_or_default();
            let v = client.get(&format!("/projects/{pid}/repositories/{rid}/refs")).await?;
            Ok(ToolResult::json(&v))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
