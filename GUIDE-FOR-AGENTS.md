## rust-analyzer Server (Agent Guide)

> Paste this section into any Rust project's `CLAUDE.md` to give all agents (including GSD subagents) access to rust-analyzer intelligence via HTTP API.

---

### Copy below this line into your project CLAUDE.md:

---

## rust-analyzer HTTP API

A persistent rust-analyzer server runs on `localhost:${RUST_ANALYZER_PORT:-15423}`. **Use this instead of Grep/Glob for code structure analysis.** All responses are token-optimized (85-94% reduction vs raw LSP).

### Setup (run once per workspace)

```bash
# Check current workspace
curl -s http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/workspace

# Set workspace (skips if already set to same path)
curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_set_workspace" \
  -H 'Content-Type: application/json' -d '{"workspace":"/absolute/path/to/project"}'
```

### API Reference

All endpoints: `POST http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/{tool_name}`

| Tool | Params | Use For |
|------|--------|---------|
| `rust_analyzer_hover` | `file_path, line, character` | Type info + docs for symbol |
| `rust_analyzer_definition` | `file_path, line, character` | Go to definition |
| `rust_analyzer_references` | `file_path, line, character` | Find all usages (impact analysis) |
| `rust_analyzer_workspace_symbol` | `query` | Fuzzy search symbols across project |
| `rust_analyzer_diagnostics` | `file_path` | Errors/warnings for a file |
| `rust_analyzer_workspace_diagnostics` | `{}` | All errors/warnings project-wide |
| `rust_analyzer_incoming_calls` | `file_path, line, character` | Who calls this function? |
| `rust_analyzer_outgoing_calls` | `file_path, line, character` | What does this function call? |
| `rust_analyzer_implementation` | `file_path, line, character` | Find trait implementations |
| `rust_analyzer_completion` | `file_path, line, character` | Code completions |
| `rust_analyzer_symbols` | `file_path` | All symbols in a file |
| `rust_analyzer_code_actions` | `file_path, line, character` | Available refactorings/fixes |

### curl Templates

**Symbol at position** (hover, definition, references, callers, implementations):
```bash
curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_hover" \
  -H 'Content-Type: application/json' \
  -d '{"file_path":"src/main.rs","line":5,"character":10}'
```

**Search symbols** (find structs, functions, traits by name):
```bash
curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_workspace_symbol" \
  -H 'Content-Type: application/json' -d '{"query":"MyStruct"}'
```

**File diagnostics**:
```bash
curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_diagnostics" \
  -H 'Content-Type: application/json' -d '{"file_path":"src/main.rs"}'
```

**Project-wide diagnostics**:
```bash
curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_workspace_diagnostics" \
  -H 'Content-Type: application/json' -d '{}'
```

### When to Use What

| Need | Use | NOT |
|------|-----|-----|
| Find struct/fn/trait location | `workspace_symbol` | Grep |
| Understand type of a symbol | `hover` | Guessing |
| Find all usages before refactor | `references` | Grep (misses re-exports) |
| Check errors after edit | `diagnostics` | `cargo check` (slower) |
| Trace call chain | `incoming_calls` / `outgoing_calls` | Manual reading |
| Find trait implementors | `implementation` | Grep (misses blanket impls) |
| Text/string literal search | Grep | rust-analyzer |

### Workflow Pattern

```
1. workspace_symbol("TargetName") → find file:line
2. Read file for context
3. hover(file, line, char) → understand types
4. references(file, line, char) → find all usages
5. Edit code
6. diagnostics(file) → verify no errors
```

### Response Format

All responses: `{"ok": true, "result": {...}}` or `{"ok": false, "error": "..."}`

### Health Check

```bash
curl -s http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/health
```

If server is not running: `rust-analyzer-server --workspace /path/to/project`
