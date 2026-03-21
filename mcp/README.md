# velum-mcp

**Model Context Protocol (MCP) server for [Velum](https://github.com/tnl-o/rust_semaphore)** — the Rust-native Ansible / Terraform automation platform.

Connect Claude (or any MCP-compatible AI) directly to your Velum instance and control deployments, diagnose failures, manage schedules, and explore infrastructure — all through natural language.

---

## Features

| Category | Tools |
|---|---|
| **Projects** | list, get, create, update, delete |
| **Templates** | list, get, create, update, delete, run, stop-all |
| **Tasks** | list, get, run, stop, output, filter, latest-failed, waiting, bulk-stop, confirm, reject |
| **Schedules** | list, get, create, toggle, delete, validate-cron |
| **Repositories** | list, get, create, update, delete, list-branches |
| **Environments** | list, get, create, update, delete |
| **Access Keys** | list, get, create, delete |
| **Inventory** | list, get, create, update, delete |
| **Runners** | list, status, toggle, clear-cache |
| **Analytics** | project-analytics, task-trends, system-analytics, project-health |
| **Audit / System** | audit-log, project-events, system-info |
| **Playbooks** | list, get, sync-repo, run, history |
| **AI Analyzer** | analyze-task-failure, bulk-analyze-failures |

**Total: 60 tools**

---

## Why Rust?

| | velum-mcp (Rust) | semaphore-mcp (Python) |
|---|---|---|
| Binary size | ~5 MB | ~50 MB + runtime |
| Startup time | <10 ms | ~500 ms |
| Memory | ~8 MB | ~60 MB |
| Dependencies | 0 system deps | Python 3.11+, pip packages |
| License | MIT | AGPL-3.0 |

---

## Quickstart

### 1. Get an API token

In Velum → **Profile → API Tokens → New Token**. Copy the token.

### 2. Configure

```bash
cp .env.example .env
# Edit .env: set VELUM_URL and VELUM_API_TOKEN
```

### 3a. Run locally (stdio — for Claude Desktop / Claude Code)

```bash
cargo build --release
./target/release/velum-mcp
```

### 3b. Run via Docker (HTTP mode)

```bash
docker compose up --build -d
# MCP endpoint: http://localhost:8500/mcp
```

---

## Claude Desktop configuration

Add to `~/.claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "velum": {
      "command": "/path/to/velum-mcp",
      "env": {
        "VELUM_URL": "http://localhost:3000",
        "VELUM_API_TOKEN": "your_token_here"
      }
    }
  }
}
```

## Claude Code configuration

```bash
claude mcp add velum /path/to/velum-mcp \
  --env VELUM_URL=http://localhost:3000 \
  --env VELUM_API_TOKEN=your_token_here
```

---

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `VELUM_URL` | `http://localhost:3000` | Velum base URL |
| `VELUM_API_TOKEN` | *(required)* | API token from Velum profile |
| `VELUM_TIMEOUT_SECS` | `30` | HTTP request timeout |
| `MCP_TRANSPORT` | `stdio` | `stdio` or `http` |
| `MCP_PORT` | `8500` | Port for HTTP transport |
| `RUST_LOG` | `velum_mcp=info` | Log level |

---

## AI-Powered Failure Analysis

The `analyze_task_failure` tool retrieves task output and structures it as a rich prompt so Claude can diagnose failures **in its own context** — no external API key required.

Example prompt to Claude:
> *"Analyze the latest failed task in project 1 and suggest fixes"*

Claude will call `bulk_analyze_failures`, receive the console output, and provide root cause analysis, ranked reasons, and remediation steps.

---

## Building from Source

```bash
# Debug build
cargo build

# Release build (optimized, LTO, stripped — ~5 MB)
cargo build --release

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings
```

---

## License

MIT — see [LICENSE](../LICENSE)

Part of the **Velum** project: full Rust rewrite of Semaphore with AI-native features.
