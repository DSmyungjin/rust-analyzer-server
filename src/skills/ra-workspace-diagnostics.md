Get all compiler diagnostics across the entire workspace.

Usage: /ra-workspace-diagnostics

```bash
RESULT=$(curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-3000}/api/v1/rust_analyzer_workspace_diagnostics" \
  -H 'Content-Type: application/json' \
  -d '{}' 2>/dev/null)

if [ $? -ne 0 ] || [ -z "$RESULT" ]; then
  echo "ERROR: rust-analyzer HTTP server is not running."
  echo "Start it with: rust-analyzer-mcp --workspace /path/to/project"
  exit 1
fi

echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
```
