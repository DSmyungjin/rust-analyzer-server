Find all functions that call this function (incoming call hierarchy).

Usage: /ra-callers <file_path> <line> <character>

Example: /ra-callers src/main.rs 5 10

```bash
ARGS="$ARGUMENTS"
FILE=$(echo "$ARGS" | awk '{print $1}')
LINE=$(echo "$ARGS" | awk '{print $2}')
CHAR=$(echo "$ARGS" | awk '{print $3}')

RESULT=$(curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-3000}/api/v1/rust_analyzer_incoming_calls" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\",\"line\":$LINE,\"character\":$CHAR}" 2>/dev/null)

if [ $? -ne 0 ] || [ -z "$RESULT" ]; then
  echo "ERROR: rust-analyzer HTTP server is not running."
  echo "Start it with: rust-analyzer-mcp --workspace /path/to/project"
  exit 1
fi

echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
```
