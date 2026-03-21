use crate::client::VelumClient;
use crate::protocol::{prop_bool, prop_int, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_runners".into(),
            description: "List all registered runner agents across the system with their status and last heartbeat time.".into(),
            input_schema: json!({ "type": "object", "properties": {}, "required": [] }),
        },
        Tool {
            name: "get_runner_status".into(),
            description: "Get current status and last heartbeat of a specific runner agent.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "runner_id": prop_int("Runner ID") },
                "required": ["runner_id"]
            }),
        },
        Tool {
            name: "toggle_runner".into(),
            description: "Enable or disable a runner agent. Disabled runners won't pick up new tasks.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "runner_id": prop_int("Runner ID"),
                    "active": prop_bool("true to enable, false to disable")
                },
                "required": ["runner_id","active"]
            }),
        },
        Tool {
            name: "clear_runner_cache".into(),
            description: "Clear the Git repository cache on a runner. Useful when the runner has stale cached content.".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "runner_id": prop_int("Runner ID") },
                "required": ["runner_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    match name {
        "list_runners" => {
            let v = client.get("/runners").await?;
            Ok(ToolResult::json(&v))
        }
        "get_runner_status" => {
            let rid = args["runner_id"].as_i64().unwrap_or_default();
            let runners = client.get("/runners").await?;
            let runner = runners.as_array().and_then(|a| {
                a.iter().find(|r| r["id"].as_i64() == Some(rid)).cloned()
            });
            match runner {
                Some(r) => Ok(ToolResult::json(&r)),
                None => Ok(ToolResult::error(format!("Runner {rid} not found."))),
            }
        }
        "toggle_runner" => {
            let rid = args["runner_id"].as_i64().unwrap_or_default();
            let active = args["active"].as_bool().unwrap_or(true);
            let runners = client.get("/runners").await?;
            let mut runner = runners.as_array()
                .and_then(|a| a.iter().find(|r| r["id"].as_i64() == Some(rid)).cloned())
                .ok_or_else(|| anyhow::anyhow!("Runner {rid} not found"))?;
            runner["active"] = json!(active);
            client.put(&format!("/runners/{rid}"), &runner).await?;
            let state = if active { "enabled" } else { "disabled" };
            let rname = runner["name"].as_str().unwrap_or("runner");
            Ok(ToolResult::text(format!("Runner {rid} ({rname}) {state}.")))
        }
        "clear_runner_cache" => {
            let rid = args["runner_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/runners/{rid}/cache")).await?;
            Ok(ToolResult::text(format!("Cache cleared for runner {rid}.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
