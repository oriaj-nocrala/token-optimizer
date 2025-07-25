#!/bin/bash

# Hot reload MCP server for iterative development
# This script watches for changes and automatically restarts the server

echo "🔥 Starting MCP Server with Hot Reload"
echo "   Port: 4080"
echo "   Watching: src/ for changes"
echo "   Press Ctrl+C to stop"
echo ""

# Kill any existing MCP server processes
pkill -f mcp_server 2>/dev/null || true

# Start cargo watch with automatic restart
cargo watch -x "run --bin mcp_server -- --port 4080" \
    --watch src/ \
    --clear \
    --delay 1 \
    --ignore "**/*.md" \
    --ignore "**/*.json" \
    --ignore ".cache/**" \
    --ignore "target/**"