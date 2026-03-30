#!/bin/bash

# Spree Research + Payment Integration Demo
# This script demonstrates the combined architecture

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       SPREE RESEARCH + PAYMENT INTEGRATION DEMO              ║"
echo "║  Autoresearch-style experiments with mppx-style micropayments ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if wizai2 is already running
if curl -s http://localhost:3000 > /dev/null; then
    echo "✓ Spree server is already running on http://localhost:3000"
else
    echo "Starting Spree server..."
    cd "$(dirname "$0")"
    
    # Check for API key
    if [ -z "$VENICE_API_KEY" ]; then
        if [ -f .env ]; then
            export $(cat .env | grep -v '^#' | xargs)
        fi
    fi
    
    if [ -z "$VENICE_API_KEY" ]; then
        echo "⚠️  Warning: VENICE_API_KEY not set. Some features may not work."
        echo "   Set it with: export VENICE_API_KEY=your_key_here"
        echo ""
    fi
    
    # Build and run in background
    echo "Building Spree server..."
    cargo build --release 2>&1 | tail -5
    
    echo "Starting server..."
    ./target/release/wizai2-agent &
    SERVER_PID=$!
    
    # Wait for server to start
    echo "Waiting for server to start..."
    for i in {1..30}; do
        if curl -s http://localhost:3000 > /dev/null 2>&1; then
            echo "✓ Server started successfully"
            break
        fi
        sleep 1
    done
    
    if ! curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "✗ Server failed to start"
        exit 1
    fi
fi

echo ""
echo "═══════════════════════════════════════════════════"
echo "📡 API ENDPOINTS"
echo "═══════════════════════════════════════════════════"
echo ""

# Test organization endpoint
echo "1. Checking organization structure..."
ORG_RESPONSE=$(curl -s http://localhost:3000/api/organization)
echo "   Response: $ORG_RESPONSE"
echo ""

# Get CEO ID
echo "2. Getting CEO agent ID..."
CEO_ID=$(echo "$ORG_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -z "$CEO_ID" ]; then
    echo "   ✗ Could not find CEO ID"
else
    echo "   ✓ CEO ID: $CEO_ID"
fi
echo ""

# Create a research manager
echo "3. Creating Research Manager..."
if [ -n "$CEO_ID" ]; then
    MANAGER_RESPONSE=$(curl -s -X POST http://localhost:3000/api/agents \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"Research Manager\",\"role\":\"Manager\",\"superior_id\":\"$CEO_ID\"}")
    echo "   Response: $MANAGER_RESPONSE"
    MANAGER_ID=$(echo "$MANAGER_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    if [ -n "$MANAGER_ID" ]; then
        echo "   ✓ Created Research Manager: $MANAGER_ID"
    fi
fi
echo ""

# Create researchers
echo "4. Creating Research Agents..."
if [ -n "$MANAGER_ID" ]; then
    for i in 1 2 3; do
        RESEARCHER_RESPONSE=$(curl -s -X POST http://localhost:3000/api/agents \
            -H "Content-Type: application/json" \
            -d "{\"name\":\"Researcher $i\",\"role\":\"Specialist\",\"superior_id\":\"$MANAGER_ID\"}")
        RESEARCHER_ID=$(echo "$RESEARCHER_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        if [ -n "$RESEARCHER_ID" ]; then
            echo "   ✓ Created Researcher $i: $RESEARCHER_ID"
        fi
    done
fi
echo ""

# Show updated organization
echo "5. Updated organization structure..."
curl -s http://localhost:3000/api/organization | jq . 2>/dev/null || curl -s http://localhost:3000/api/organization
echo ""

# Submit a task
echo "6. Submitting research task..."
if [ -n "$CEO_ID" ]; then
    TASK_RESPONSE=$(curl -s -X POST http://localhost:3000/api/tasks \
        -H "Content-Type: application/json" \
        -d "{\"agent_id\":\"$CEO_ID\",\"task\":\"Design an experiment to test different learning rate schedules for our nano-LLM\"}")
    echo "   Task submitted: $TASK_RESPONSE"
fi
echo ""

echo "═══════════════════════════════════════════════════"
echo "✅ DEMO COMPLETE"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Key Features Demonstrated:"
echo "  ✓ Hierarchical agent creation (CEO → Manager → Researchers)"
echo "  ✓ REST API for agent management"
echo "  ✓ Task delegation through organization"
echo "  ✓ SQLite persistence"
echo ""
echo "Next Steps:"
echo "  - Visit http://localhost:3000 for web UI"
echo "  - Connect to ws://localhost:3000/ws for real-time updates"
echo "  - View agent activity in logs"
echo ""
echo "Payment System (Phase 2):"
echo "  - Per-token LLM pricing like mppx"
echo "  - Per-experiment budget management like autoresearch"
echo "  - Agent-to-agent micropayments"
echo "  - Cost tracking and invoicing"
echo ""

# Optionally kill the server if we started it
if [ -n "$SERVER_PID" ]; then
    echo "Press Enter to stop the server, or Ctrl+C to keep it running..."
    read
    kill $SERVER_PID 2>/dev/null
    echo "Server stopped"
fi
