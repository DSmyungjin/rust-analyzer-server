Multi-step impact analysis: hover + references + callers + implementations.

Analyzes a symbol to understand its type, where it's used, who calls it, and what implements it.

Usage: /ra-impact <file_path> <line> <character>

Example: /ra-impact src/main.rs 5 10

```bash
ARGS="$ARGUMENTS"
FILE=$(echo "$ARGS" | awk '{print $1}')
LINE=$(echo "$ARGS" | awk '{print $2}')
CHAR=$(echo "$ARGS" | awk '{print $3}')
PORT="${RUST_ANALYZER_PORT:-3000}"
BASE="http://localhost:${PORT}/api/v1"

echo "=== Impact Analysis: $FILE:$LINE:$CHAR ==="
echo ""

echo "--- Hover (type info) ---"
curl -s -X POST "$BASE/rust_analyzer_hover" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\",\"line\":$LINE,\"character\":$CHAR}" 2>/dev/null | python3 -m json.tool 2>/dev/null

echo ""
echo "--- References ---"
curl -s -X POST "$BASE/rust_analyzer_references" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\",\"line\":$LINE,\"character\":$CHAR}" 2>/dev/null | python3 -m json.tool 2>/dev/null

echo ""
echo "--- Incoming Calls (callers) ---"
curl -s -X POST "$BASE/rust_analyzer_incoming_calls" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\",\"line\":$LINE,\"character\":$CHAR}" 2>/dev/null | python3 -m json.tool 2>/dev/null

echo ""
echo "--- Implementations ---"
curl -s -X POST "$BASE/rust_analyzer_implementation" \
  -H 'Content-Type: application/json' \
  -d "{\"file_path\":\"$FILE\",\"line\":$LINE,\"character\":$CHAR}" 2>/dev/null | python3 -m json.tool 2>/dev/null
```
