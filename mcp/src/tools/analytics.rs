use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_str_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "get_project_analytics".into(),
            description: "Get task execution analytics for a project: total runs, success rate, failure count, average duration.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "period": { "type": "string", "enum": ["today","week","month","year"], "description": "Time window (default: week)" }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_task_trends".into(),
            description: "Get time-series data of task success/failure counts — useful for understanding deployment frequency and reliability trends.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "period": { "type": "string", "enum": ["today","week","month","year"] }
                },
                "required": ["project_id"]
            }),
        },
        Tool {
            name: "get_system_analytics".into(),
            description: "Get system-wide analytics: total projects, tasks, users, runners, success rate.".into(),
            input_schema: json!({ "type": "object", "properties": {}, "required": [] }),
        },
        Tool {
            name: "get_project_health".into(),
            description: "Compute a health summary (healthy/degraded/critical) for a project based on recent task history. Returns success rate, run count, and recommendations.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "period": prop_str_opt("Time window: today/week/month (default: week)")
                },
                "required": ["project_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args.get("project_id").and_then(Value::as_i64).unwrap_or_default();
    let period = args.get("period").and_then(Value::as_str).unwrap_or("week");
    match name {
        "get_project_analytics" => {
            let v = client.get_query(&format!("/project/{pid}/analytics"), &[("period", period)]).await?;
            Ok(ToolResult::json(&v))
        }
        "get_task_trends" => {
            let v = client.get_query(&format!("/project/{pid}/analytics/tasks-chart"), &[("period", period)]).await?;
            Ok(ToolResult::json(&v))
        }
        "get_system_analytics" => {
            let v = client.get("/analytics/system").await?;
            Ok(ToolResult::json(&v))
        }
        "get_project_health" => {
            let analytics = client.get_query(&format!("/project/{pid}/analytics"), &[("period", period)]).await?;
            let total = analytics["total"].as_f64().unwrap_or(0.0);
            let success = analytics["success"].as_f64().unwrap_or(0.0);
            let failure = analytics["error"].as_f64().unwrap_or(0.0);
            let rate = if total > 0.0 { (success / total * 100.0 * 10.0).round() / 10.0 } else { 0.0 };
            let status = if rate >= 95.0 { "healthy" } else if rate >= 80.0 { "degraded" } else { "critical" };
            let emoji = if rate >= 95.0 { "✅" } else if rate >= 80.0 { "⚠️" } else { "🔴" };
            let summary = json!({
                "project_id": pid,
                "period": period,
                "health_status": status,
                "health_emoji": emoji,
                "total_runs": total as i64,
                "success_count": success as i64,
                "failure_count": failure as i64,
                "success_rate_pct": rate,
                "recommendation": match status {
                    "healthy" => "Project is running well.",
                    "degraded" => "Success rate below 95%. Review recent failures.",
                    _ => "Critical failure rate. Immediate investigation recommended.",
                }
            });
            Ok(ToolResult::json(&summary))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}
