Search for symbols across the workspace (fuzzy matching).

Usage: /ra-search <query>

Example: /ra-search TradeData

```bash
QUERY="$ARGUMENTS"

RESULT=$(curl -s -X POST "http://localhost:${RUST_ANALYZER_PORT:-3000}/api/v1/rust_analyzer_workspace_symbol" \
  -H 'Content-Type: application/json' \
  -d "{\"query\":\"$QUERY\"}" 2>/dev/null)

if [ $? -ne 0 ] || [ -z "$RESULT" ]; then
  echo "ERROR: rust-analyzer HTTP server is not running."
  echo "Start it with: rust-analyzer-mcp --workspace /path/to/project"
  exit 1
fi

echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
```
