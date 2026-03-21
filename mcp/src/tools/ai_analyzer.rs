/// AI-powered task failure analysis.
///
/// Retrieves task output and structures it as a rich prompt so the
/// calling AI assistant (Claude) can diagnose failures in its own context —
/// no external API key or second LLM call required.
use crate::client::VelumClient;
use crate::protocol::{prop_int, prop_int_opt, Tool, ToolResult};
use serde_json::{json, Value};
use anyhow::Result;

pub fn definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "analyze_task_failure".into(),
            description: "Retrieve output from a failed task and structure it for AI diagnosis. \
                Returns the task metadata and console output so you (Claude) can identify \
                the root cause and suggest fixes — no external API key needed.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "task_id": prop_int("The failed task ID to analyse"),
                    "last_n_lines": prop_int_opt("Lines of output to include (default 100)")
                },
                "required": ["project_id","task_id"]
            }),
        },
        Tool {
            name: "bulk_analyze_failures".into(),
            description: "Fetch the N most recent failed tasks and structure them for bulk AI analysis. \
                Returns metadata + output snippets for each failure.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": prop_int("Project ID"),
                    "limit": prop_int_opt("Number of recent failures to include (default 5)")
                },
                "required": ["project_id"]
            }),
        },
    ]
}

pub async fn call(name: &str, args: &Value, client: &VelumClient) -> Result<ToolResult> {
    let pid = args["project_id"].as_i64().unwrap_or_default();
    match name {
        "analyze_task_failure" => {
            let tid = args["task_id"].as_i64().unwrap_or_default();
            let n = args.get("last_n_lines").and_then(Value::as_u64).unwrap_or(100) as usize;
            let result = fetch_failure_analysis(client, pid, tid, n).await?;
            Ok(ToolResult::json(&result))
        }
        "bulk_analyze_failures" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(5) as usize;
            let all = client.get(&format!("/projects/{pid}/tasks")).await?;
            let failed: Vec<Value> = all.as_array().map(|a| {
                let mut f: Vec<Value> = a.iter()
                    .filter(|t| t["status"].as_str() == Some("error"))
                    .cloned()
                    .collect();
                f.sort_by_key(|t| -(t["id"].as_i64().unwrap_or(0)));
                f.truncate(limit);
                f
            }).unwrap_or_default();

            let mut results = Vec::new();
            for task in &failed {
                if let Some(tid) = task["id"].as_i64() {
                    match fetch_failure_analysis(client, pid, tid, 50).await {
                        Ok(r) => results.push(r),
                        Err(e) => results.push(json!({"task_id": tid, "error": e.to_string()})),
                    }
                }
            }
            Ok(ToolResult::json(&json!(results)))
        }
        _ => Ok(ToolResult::error(format!("Unknown tool: {name}"))),
    }
}

async fn fetch_failure_analysis(client: &VelumClient, pid: i64, tid: i64, n: usize) -> Result<Value> {
    let task = client.get(&format!("/projects/{pid}/tasks/{tid}")).await?;
    let status = task["status"].as_str().unwrap_or("unknown");

    if status != "error" && status != "stopped" {
        return Ok(json!({
            "error": format!("Task {tid} has status '{status}' — analysis is for failed tasks only."),
            "task": task
        }));
    }

    let raw = client.get(&format!("/projects/{pid}/tasks/{tid}/raw_output")).await.unwrap_or(Value::Null);
    let lines: Vec<String> = match &raw {
        Value::String(s) => s.lines().map(String::from).collect(),
        Value::Array(arr) => arr.iter().map(|r| r.as_str().unwrap_or("").to_string()).collect(),
        _ => vec![],
    };

    let total_lines = lines.len();
    let tail: Vec<&String> = lines.iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect();
    let output_tail = tail.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
    let displayed = tail.len();

    Ok(json!({
        "task_id": tid,
        "project_id": pid,
        "template_id": task.get("template_id"),
        "status": status,
        "started": task.get("start"),
        "finished": task.get("end"),
        "total_output_lines": total_lines,
        "output_tail": output_tail,
        "analysis_guidance": format!(
            "Task #{tid} failed (status: {status}). Template ID: {}. \
             Started: {}, Finished: {}.\n\n\
             Last {displayed}/{total_lines} lines of console output:\n\
             ```\n{output_tail}\n```\n\n\
             Please diagnose:\n\
             1. Root cause of the failure\n\
             2. Most likely reasons (ranked by probability)\n\
             3. Specific remediation steps\n\
             4. Preventive measures for the future",
            task.get("template_id").and_then(Value::as_i64).unwrap_or(0),
            task.get("start").and_then(Value::as_str).unwrap_or("unknown"),
            task.get("end").and_then(Value::as_str).unwrap_or("unknown"),
        )
    }))
}
