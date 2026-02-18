Health check and workspace setup for the rust-analyzer HTTP server.

Usage: /ra-setup [workspace_path]

If no workspace_path is given, just checks the server health.

```bash
PORT="${RUST_ANALYZER_PORT:-3000}"
WORKSPACE="$ARGUMENTS"

# Health check
HEALTH=$(curl -s "http://localhost:${PORT}/api/v1/health" 2>/dev/null)
if [ $? -ne 0 ] || [ -z "$HEALTH" ]; then
  echo "ERROR: rust-analyzer HTTP server is not running on port ${PORT}."
  echo "Start it with: rust-analyzer-server --workspace /path/to/project"
  echo ""
  echo "Or set a custom port: RUST_ANALYZER_PORT=4000 rust-analyzer-server -p 4000"
  exit 1
fi

echo "Server health:"
echo "$HEALTH" | python3 -m json.tool 2>/dev/null || echo "$HEALTH"

# Set workspace if provided
if [ -n "$WORKSPACE" ]; then
  echo ""
  echo "Setting workspace to: $WORKSPACE"
  RESULT=$(curl -s -X POST "http://localhost:${PORT}/api/v1/workspace" \
    -H 'Content-Type: application/json' \
    -d "{\"workspace_path\":\"$WORKSPACE\"}" 2>/dev/null)
  echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
fi
```
