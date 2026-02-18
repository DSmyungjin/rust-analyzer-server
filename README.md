# rust-analyzer-server

A standalone HTTP server for rust-analyzer that keeps the language server warm across requests, with installable Claude Code skills for seamless integration.

## How It Works

```
rust-analyzer-server (persistent HTTP server, rust-analyzer always warm)
  ^  REST API (localhost:15423)
  |
Claude Code skills (.claude/commands/ra-*.md)
  -> curl calls to the HTTP server
```

The server starts rust-analyzer once and keeps it running. All subsequent requests are fast because the project is already indexed.

## Prerequisites

1. **rust-analyzer** in your PATH:
   ```bash
   rustup component add rust-analyzer
   ```
2. **Rust** 1.70+ with Cargo

## Installation

### From Source

```bash
git clone https://github.com/DSmyungjin/rust-analyzer-server.git
cd rust-analyzer-server
cargo build --release
```

The binary will be at `target/release/rust-analyzer-server`.

## Usage

### Start the Server

```bash
# Start with default settings (port 15423, current directory as workspace)
rust-analyzer-server

# Specify workspace and port
rust-analyzer-server --workspace /path/to/project --port 4000

# Custom bind address
rust-analyzer-server --bind 0.0.0.0 --port 15423
```

Environment variable `RUST_ANALYZER_PORT` can also set the port.

### Install Claude Code Skills

Copy skill templates into any project:

```bash
rust-analyzer-server install /path/to/your/project
```

This creates `.claude/commands/ra-*.md` files that provide slash commands like `/ra-hover`, `/ra-definition`, `/ra-references`, etc.

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/health` | GET | Server status + workspace info |
| `/api/v1/tools` | GET | List available tools |
| `/api/v1/workspace` | GET | Current workspace |
| `/api/v1/workspace` | POST | Change workspace |
| `/api/v1/shutdown` | POST | Graceful shutdown |
| `/api/v1/{tool_name}` | POST | Call any tool |

All responses use a JSON envelope:
```json
{"ok": true, "result": {...}}
{"ok": false, "error": "..."}
```

### Example API Calls

```bash
# Health check
curl http://localhost:15423/api/v1/health

# Get hover info
curl -X POST http://localhost:15423/api/v1/rust_analyzer_hover \
  -H 'Content-Type: application/json' \
  -d '{"file_path":"src/main.rs","line":5,"character":10}'

# Find definition
curl -X POST http://localhost:15423/api/v1/rust_analyzer_definition \
  -H 'Content-Type: application/json' \
  -d '{"file_path":"src/main.rs","line":10,"character":15}'

# Workspace symbol search
curl -X POST http://localhost:15423/api/v1/rust_analyzer_workspace_symbol \
  -H 'Content-Type: application/json' \
  -d '{"query":"MyStruct"}'
```

## Available Tools

| Tool | Description |
|------|-------------|
| `rust_analyzer_hover` | Type info + docs at position |
| `rust_analyzer_definition` | Go to definition |
| `rust_analyzer_references` | Find all references |
| `rust_analyzer_workspace_symbol` | Search symbols across workspace |
| `rust_analyzer_symbols` | Document symbols for a file |
| `rust_analyzer_diagnostics` | File diagnostics (errors/warnings) |
| `rust_analyzer_workspace_diagnostics` | All workspace diagnostics |
| `rust_analyzer_incoming_calls` | Find callers of a function |
| `rust_analyzer_outgoing_calls` | Find callees of a function |
| `rust_analyzer_implementation` | Find trait implementations |
| `rust_analyzer_parent_module` | Navigate to parent module |
| `rust_analyzer_completion` | Code completions |
| `rust_analyzer_format` | Format document |
| `rust_analyzer_code_actions` | Quick fixes and refactorings |
| `rust_analyzer_inlay_hint` | Type annotations for a range |
| `rust_analyzer_set_workspace` | Change workspace root |

## Installed Skills

After running `rust-analyzer-server install`, these slash commands become available in Claude Code:

| Skill | Command | Description |
|-------|---------|-------------|
| `/ra-hover` | Hover info | Type info + docs at position |
| `/ra-definition` | Go to def | Jump to definition |
| `/ra-references` | Find refs | All references to a symbol |
| `/ra-search` | Symbol search | Workspace-wide symbol search |
| `/ra-diagnostics` | File errors | Diagnostics for a file |
| `/ra-workspace-diagnostics` | All errors | Workspace-wide diagnostics |
| `/ra-callers` | Callers | Who calls this function? |
| `/ra-callees` | Callees | What does this function call? |
| `/ra-implementations` | Impls | Find trait implementations |
| `/ra-setup` | Health check | Verify server status |
| `/ra-impact` | Impact analysis | Multi-step analysis (hover + refs + callers + impls) |

## Development

```bash
cargo build          # Build
cargo test           # Run all 35 tests
cargo run            # Run in dev mode
RUST_LOG=debug cargo run  # Verbose logging
```

## License

MIT
