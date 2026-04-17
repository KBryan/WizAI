# Quick Start Guide

## 1. Configure API Key

```bash
cd /Users/kwamebryan/Documents/GitHub/WizAI/wizai2
export VENICE_API_KEY="your_api_key_here"
```

Or create a `.env` file:
```bash
echo "VENICE_API_KEY=your_api_key_here" > .env
```

## 2. Run the Server

```bash
cargo run
```

You should see:
```
Starting Spree Agent Framework...
MemoryStore initialized at spree.db
Creating initial CEO agent...
Created CEO agent with ID: ...
Creating initial C-level team...
Created CTO agent: ...
Created CFO agent: ...
Created Chief AI Officer agent: ...
Created Chief Product Officer agent: ...
Initial team created successfully!
Server starting on http://127.0.0.1:3000
```

## 3. Test the API

In a new terminal:

### List all agents
```bash
curl http://localhost:3000/api/agents | jq
```

Expected output:
```json
[
  {
    "id": "...",
    "name": "Spree CEO",
    "role": "CEO",
    "superior": null,
    "subordinates": [...],
    "status": "Idle"
  },
  ...
]
```

### Get organization tree
```bash
curl http://localhost:3000/api/organization | jq
```

### Create a new agent
```bash
# Replace <ceo-id> with actual CEO ID from previous command
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Development Manager",
    "role": "Manager",
    "superior_id": "<ceo-id>"
  }'
```

### Submit a task
```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "<ceo-id>",
    "task": "Review our current technology stack and suggest improvements"
  }'
```

## 4. Open Web UI

Visit: http://localhost:3000

You should see the Spree Agent Framework landing page with API documentation.

## 5. Test WebSocket (Optional)

Create a simple HTML file:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Spree WebSocket Test</title>
</head>
<body>
    <h1>WebSocket Test</h1>
    <div id="output"></div>
    <script>
        const ws = new WebSocket('ws://localhost:3000/ws');
        const output = document.getElementById('output');
        
        ws.onopen = () => {
            output.innerHTML += '<p>Connected!</p>';
            ws.send(JSON.stringify({ type: 'GetOrganization' }));
        };
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            output.innerHTML += '<pre>' + JSON.stringify(data, null, 2) + '</pre>';
        };
        
        ws.onerror = (error) => {
            output.innerHTML += '<p>Error: ' + error + '</p>';
        };
    </script>
</body>
</html>
```

Open this file in a browser to test WebSocket connectivity.

## Troubleshooting

### "VENICE_API_KEY not set"
```bash
export VENICE_API_KEY="your_key"
```

### "Address already in use"
```bash
# Kill existing process
lsof -ti:3000 | xargs kill -9
# Or use different port
PORT=3001 cargo run
```

### "Failed to compile"
```bash
# Clean and rebuild
cargo clean
cargo build
```

## Next Steps

1. ✅ Server is running
2. ✅ API is accessible
3. ✅ CEO and C-level team created
4. 🔄 Create your first subordinate agent
5. 🔄 Submit a task and see delegation in action
6. 🔄 Monitor agent activities in logs

## Project Structure

- `/api/agents` - Agent management
- `/api/organization` - Org hierarchy
- `/api/tasks` - Task submission
- `/ws` - WebSocket for real-time updates
- `/` - Web UI landing page

## Architecture

```
User
  └── CEO (Spree CEO)
       ├── CTO (Spree CTO)
       ├── CFO (Spree CFO)
       ├── Chief AI Officer
       └── Chief Product Officer
```

Each C-level can create subordinates:
- CTO → VP Engineering → Engineering Managers → Team Leads → Engineers
- CFO → VP Finance → Finance Managers → Accountants
- etc.

## Success! 🎉

Your Spree Agent Framework is now running!
