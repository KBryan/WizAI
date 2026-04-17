#!/bin/bash
# Test Repliers MCP Integration
# Usage: ./test_repliers_mcp.sh

set -e

echo "=========================================="
echo "Repliers MCP Integration Test"
echo "=========================================="
echo ""

# Check if REPLIERS_API_KEY is set
if [ -z "$REPLIERS_API_KEY" ]; then
    echo "❌ REPLIERS_API_KEY is not set!"
    echo "Please set it in your environment or spree/.env file"
    exit 1
fi

echo "✅ REPLIERS_API_KEY is set"
echo ""

# Check if MCP server exists
if [ ! -f "../repliers-mcp-server/mcpServer.js" ]; then
    echo "❌ Repliers MCP server not found!"
    echo "Expected at: ../repliers-mcp-server/mcpServer.js"
    exit 1
fi

echo "✅ Repliers MCP server found"
echo ""

# Run Rust tests
echo "Running Rust integration tests..."
echo "------------------------------------------"

cd ..
cargo test --manifest-path wizai2/Cargo.toml test_repliers_mcp_connection -- --nocapture 2>&1 | tail -30

echo ""
echo "=========================================="
echo "Test Complete!"
echo "=========================================="
echo ""
echo "To test the full integration, run:"
echo "  cargo test --manifest-path spree/Cargo.toml test_durham_region_search -- --nocapture"
echo ""
