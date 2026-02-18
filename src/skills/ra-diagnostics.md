Get compiler diagnostics (errors, warnings) for a file.

Usage: /ra-diagnostics <file_path>

Example: /ra-diagnostics src/main.rs

```bash
FILE="$ARGUMENTS"

RESULT=$(curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-15423}/api/v1/rust_analyzer_diagnostics" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\"}" 2>/dev/null)

if [ $? -ne 0 ] || [ -z "$RESULT" ]; then
  echo "ERROR: rust-analyzer HTTP server is not running."
  echo "Start it with: rust-analyzer-server --workspace /path/to/project"
  exit 1
fi

echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
```
