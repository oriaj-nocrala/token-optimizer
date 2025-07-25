#!/bin/bash

# MCP Tools CLI wrapper for easy testing and dogfooding
# Usage: ./scripts/mcp-tools.sh <tool> [parameters]

BASE_URL="http://localhost:4080"

function smart_context() {
    local query="$1"
    local max_tokens="${2:-4000}"
    
    curl -s -X POST "$BASE_URL/tools/smart_context" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"smart_context\",
            \"parameters\": {
                \"query\": \"$query\",
                \"max_tokens\": $max_tokens,
                \"include_tests\": false
            }
        }" | jq -r '.result.context // .error // "Error calling tool"'
}

function explore_codebase() {
    local query="$1"
    local max_results="${2:-10}"
    
    curl -s -X POST "$BASE_URL/tools/explore_codebase" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"explore_codebase\",
            \"parameters\": {
                \"query\": \"$query\",
                \"max_results\": $max_results,
                \"include_snippets\": true
            }
        }" | jq -r '.result.files[] | "📁 \(.file_path) (relevance: \(.relevance_score))\n   \(.snippet)\n"'
}

function project_overview() {
    local format="${1:-markdown}"
    
    curl -s -X POST "$BASE_URL/tools/project_overview" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"project_overview\",
            \"parameters\": {
                \"format\": \"$format\",
                \"include_health\": true
            }
        }" | jq -r '.result.overview // .error // "Error calling tool"'
}

function changes_analysis() {
    local since="${1:-last-commit}"
    
    curl -s -X POST "$BASE_URL/tools/changes_analysis" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"changes_analysis\",
            \"parameters\": {
                \"since\": \"$since\",
                \"modified_only\": true
            }
        }" | jq -r '.result.changes_summary // .error // "Error calling tool"'
}

function file_summary() {
    local file_path="$1"
    local format="${2:-markdown}"
    
    curl -s -X POST "$BASE_URL/tools/file_summary" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"file_summary\",
            \"parameters\": {
                \"file_path\": \"$file_path\",
                \"format\": \"$format\",
                \"include_complexity\": true,
                \"include_functions\": true,
                \"include_dependencies\": true
            }
        }" | jq -r '.result.summary // .error // "Error calling tool"'
}

function cache_status() {
    local include_details="${1:-false}"
    local format="${2:-markdown}"
    
    curl -s -X POST "$BASE_URL/tools/cache_status" \
        -H "Content-Type: application/json" \
        -d "{
            \"tool\": \"cache_status\",
            \"parameters\": {
                \"include_details\": $include_details,
                \"check_integrity\": false,
                \"format\": \"$format\"
            }
        }" | jq -r '.result.status_report // .error // "Error calling tool"'
}

# Main command dispatcher
case "$1" in
    "smart_context")
        if [ -z "$2" ]; then
            echo "Usage: $0 smart_context <query> [max_tokens]"
            exit 1
        fi
        smart_context "$2" "$3"
        ;;
    "explore")
        if [ -z "$2" ]; then
            echo "Usage: $0 explore <query> [max_results]"
            exit 1
        fi
        explore_codebase "$2" "$3"
        ;;
    "overview")
        project_overview "$2"
        ;;
    "changes")
        changes_analysis "$2"
        ;;
    "file")
        if [ -z "$2" ]; then
            echo "Usage: $0 file <file_path> [format]"
            exit 1
        fi
        file_summary "$2" "$3"
        ;;
    "cache")
        cache_status "$2" "$3"
        ;;
    "help"|"--help"|"-h"|"")
        echo "🎯 MCP Tools CLI - Token Optimizer"
        echo ""
        echo "Usage: $0 <command> [arguments]"
        echo ""
        echo "Commands:"
        echo "  smart_context <query> [max_tokens]  - Get optimized code context"
        echo "  explore <query> [max_results]       - Explore codebase semantically"
        echo "  overview [format]                   - Get project overview"
        echo "  changes [since]                     - Analyze recent changes"
        echo "  file <file_path> [format]           - Get detailed file analysis"
        echo "  cache [include_details] [format]    - Get cache status and health"
        echo ""
        echo "Examples:"
        echo "  $0 smart_context \"error handling patterns\" 3000"
        echo "  $0 explore \"MCP implementation\" 5"
        echo "  $0 overview markdown"
        echo "  $0 changes last-commit"
        echo "  $0 file src/mcp/tools.rs markdown"
        echo "  $0 cache true markdown"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac