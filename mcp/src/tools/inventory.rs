use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_int_opt, prop_str, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool { name: "list_inventory".into(), description: "List all inventories in a project.".into(),
            input_schema: json!({"type":"object","properties":{"project_id":prop_int("Project ID")},"required":["project_id"]}) },
        Tool { name: "get_inventory".into(), description: "Get details of a specific inventory.".into(),
            input_schema: json!({"type":"object","properties":{"project_id":prop_int("Project ID"),"inventory_id":prop_int("Inventory ID")},"required":["project_id","inventory_id"]}) },
        Tool { name: "create_inventory".into(), description: "Create a new inventory (static YAML/INI, dynamic, terraform-workspace, or file-based).".into(),
            input_schema: json!({"type":"object","properties":{
                "project_id":prop_int("Project ID"),"name":prop_str("Display name"),
                "type":{"type":"string","enum":["static","file","dynamic-azure","dynamic-aws","dynamic-gcp","terraform-workspace"]},
                "inventory":prop_str_opt("Inventory content or path"),"ssh_key_id":prop_int_opt("SSH key ID")},"required":["project_id","name"]}) },
        Tool { name: "update_inventory".into(), description: "Update an inventory's name or content.".into(),
            input_schema: json!({"type":"object","properties":{"project_id":prop_int("Project ID"),"inventory_id":prop_int("Inventory ID"),"name":prop_str_opt("New name"),"inventory":prop_str_opt("New content")},"required":["project_id","inventory_id"]}) },
        Tool { name: "delete_inventory".into(), description: "Delete an inventory.".into(),
            input_schema: json!({"type":"object","properties":{"project_id":prop_int("Project ID"),"inventory_id":prop_int("Inventory ID")},"required":["project_id","inventory_id"]}) },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args.get("project_id").and_then(Value::as_i64).unwrap_or_default();
    match name {
        "list_inventory" => Ok(ToolResult::json(&client.get(&format!("/projects/{pid}/inventories")).await?)),
        "get_inventory" => {
            let id = args["inventory_id"].as_i64().unwrap_or_default();
            Ok(ToolResult::json(&client.get(&format!("/projects/{pid}/inventories/{id}")).await?))
        }
        "create_inventory" => {
            let body = json!({"project_id":pid,"name":args["name"],"type":args.get("type").and_then(Value::as_str).unwrap_or("static"),
                "inventory":args.get("inventory").and_then(Value::as_str).unwrap_or(""),"ssh_key_id":args.get("ssh_key_id")});
            Ok(ToolResult::json(&client.post(&format!("/projects/{pid}/inventories"), &body).await?))
        }
        "update_inventory" => {
            let id = args["inventory_id"].as_i64().unwrap_or_default();
            let mut cur = client.get(&format!("/projects/{pid}/inventories/{id}")).await?;
            if let Some(v) = args.get("name").and_then(Value::as_str) { cur["name"] = json!(v); }
            if let Some(v) = args.get("inventory").and_then(Value::as_str) { cur["inventory"] = json!(v); }
            Ok(ToolResult::json(&client.put(&format!("/projects/{pid}/inventories/{id}"), &cur).await?))
        }
        "delete_inventory" => {
            let id = args["inventory_id"].as_i64().unwrap_or_default();
            client.delete(&format!("/projects/{pid}/inventories/{id}")).await?;
            Ok(ToolResult::text(format!("Inventory {id} deleted.")))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
