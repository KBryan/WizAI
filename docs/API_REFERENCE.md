# API Reference

## Base URL

```
http://127.0.0.1:3000
```

## Authentication

No authentication required for local development.

## Endpoints

### Agents

#### List All Agents

```http
GET /api/agents
```

**Response:**
```json
[
  {
    "id": "uuid-string",
    "name": "Agent Name",
    "role": "CEO",
    "superior": "uuid-string",
    "subordinates": ["uuid-string"],
    "status": "Idle"
  }
]
```

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Server error

---

#### Get Agent by ID

```http
GET /api/agents/:id
```

**Parameters:**
- `id` (path) - Agent UUID

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Agent Name",
  "role": "CEO",
  "superior": "uuid-string",
  "subordinates": ["uuid-string"],
  "status": "Idle"
}
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Agent not found
- `400 Bad Request` - Invalid UUID format

---

#### Create Agent

```http
POST /api/agents
```

**Request Body:**
```json
{
  "name": "My Agent",
  "role": "Specialist",
  "superior_id": "optional-uuid-string"
}
```

**Roles:**
- `CEO`, `CTO`, `CFO`, `ChiefAI`, `ChiefProduct`
- `OpenSpecExecutor`
- `SoftwareDeveloper`
- `VP`, `Director`, `Manager`, `Lead`, `Specialist`, `Intern`

**Response:**
```json
{
  "id": "uuid-string",
  "name": "My Agent",
  "role": "Specialist",
  "superior": "optional-uuid-string",
  "subordinates": [],
  "status": "Idle"
}
```

**Status Codes:**
- `201 Created` - Success
- `400 Bad Request` - Invalid role or missing name

---

#### Send Message to Agent

```http
POST /api/agents/:id
```

**Parameters:**
- `id` (path) - Agent UUID

**Request Body:**
```json
{
  "content": "Your message here"
}
```

**Response:**
```json
{
  "success": true,
  "response": "Agent's response...",
  "actions_taken": []
}
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Agent not found
- `400 Bad Request` - Invalid UUID or missing content

---

#### Get Agent Subordinates

```http
GET /api/agents/:id/subordinates
```

**Parameters:**
- `id` (path) - Agent UUID

**Response:**
```json
[
  {
    "id": "uuid-string",
    "name": "Subordinate Agent",
    "role": "Manager",
    "status": "Idle"
  }
]
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Agent not found

---

### Organization

#### Get Organization Tree

```http
GET /api/organization
```

**Response:**
```json
{
  "agents": [
    {
      "id": "uuid-string",
      "name": "CEO",
      "role": "CEO",
      "level": 0,
      "subordinates": ["uuid-string"]
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Success

---

### Skills

#### List All Skills

```http
GET /api/skills
```

**Response:**
```json
[
  {
    "name": "openspec-propose",
    "description": "Guide the agent through creating a new OpenSpec change..."
  },
  {
    "name": "typescript-dev",
    "description": "This skill enables the agent to write professional TypeScript code..."
  }
]
```

**Status Codes:**
- `200 OK` - Success

---

#### Get Skill by Name

```http
GET /api/skills/:name
```

**Parameters:**
- `name` (path) - Skill name (e.g., "typescript-dev")

**Response:**
```json
{
  "name": "typescript-dev",
  "content": "# TypeScript Developer Skill\n\n## Overview\n..."
}
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Skill not found

---

### Tasks

#### Submit Task to Agent

```http
POST /api/tasks
```

**Request Body:**
```json
{
  "agent_id": "uuid-string",
  "task": "Your task description"
}
```

**Response:**
```json
{
  "success": true,
  "response": "Agent's response...",
  "actions_taken": ["tool_name: ✓"]
}
```

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Agent not found
- `400 Bad Request` - Missing agent_id or task

---

### Real Estate Dashboard

#### Execute Real Estate CLI

```http
POST /api/realestate/execute
```

**Request Body:**
```json
{
  "command": "stats",
  "parameters": {
    "municipality": "Oshawa",
    "property_type": "detached"
  }
}
```

**Commands:**
- `fetch` - Fetch property data
- `stats` - Get market statistics
- `trends` - Get price trends
- `cma` - Generate CMA report

**Response:**
```json
{
  "success": true,
  "data": {...}
}
```

---

#### Get Real Estate Status

```http
GET /api/realestate/status
```

**Response:**
```json
{
  "cli_available": true,
  "last_updated": "2024-03-28T10:00:00Z",
  "data_sources": ["durham"]
}
```

---

### WebSocket

#### WebSocket Connection

```
ws://127.0.0.1:3000/ws
```

**Connection:**
- Connect via WebSocket for real-time updates

**Messages:**

**Client → Server:**
```json
{
  "type": "SendMessage",
  "agent_id": "uuid-string",
  "content": "Hello agent"
}
```

**Server → Client:**
```json
{
  "type": "MessageReceived",
  "agent_id": "uuid-string",
  "message": {
    "id": "uuid-string",
    "role": "Assistant",
    "content": "Hello! How can I help you?",
    "timestamp": "2024-03-28T10:00:00Z"
  }
}
```

**Events:**
- `AgentCreated`
- `MessageReceived`
- `StatusChanged`
- `ToolExecuted`
- `Error`

---

## Tool Registry

### Registered Tools

#### File System Tools

**read_file**
- Read file contents
- Parameters: `{"path": "string"}`

**write_file**
- Write content to file
- Parameters: `{"path": "string", "content": "string"}`

**list_dir**
- List directory contents
- Parameters: `{"path": "string", "recursive": "boolean"}`

**grep_code**
- Search for pattern in files
- Parameters: `{"pattern": "string", "path": "string"}`

---

#### Execution Tools

**bash**
- Execute bash command
- Parameters: `{"command": "string", "working_dir": "string"}`
- Requires approval

**search_memory**
- Search agent memory
- Parameters: `{"query": "string"}`

---

#### OpenSpec Tools

**detect_openspec_project**
- Detect OpenSpec project structure
- Parameters: `{}`

**list_changes**
- List all changes
- Parameters: `{"include_archived": "boolean"}`

**create_change_scaffold**
- Create new change scaffold
- Parameters: `{"change_name": "string"}`

**read_change_artifacts**
- Read change documentation
- Parameters: `{"change_name": "string"}`

**archive_change**
- Archive completed change
- Parameters: `{"change_name": "string"}`

---

#### CLI Generation

**generate_cli**
- Generate CLI from software
- Parameters: `{"software_path": "string", "change_name": "string"}`

**Output:** Generated CLI in `openspec/changes/{change_name}/`

---

#### Agent Management

**delegate**
- Delegate task to subordinate
- Parameters: `{"task": "string", "subordinate_role": "string"}`

---

## Error Responses

### Standard Error Format

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": {}
}
```

### Common Error Codes

- `INVALID_UUID` - Malformed UUID
- `AGENT_NOT_FOUND` - Agent doesn't exist
- `INVALID_ROLE` - Unknown agent role
- `TOOL_NOT_FOUND` - Tool not registered
- `TOOL_EXECUTION_FAILED` - Tool execution error
- `PERMISSION_DENIED` - Insufficient permissions

---

## Rate Limiting

No rate limiting implemented for local development.

## CORS

CORS is enabled for all origins in development mode.

---

## See Also

- [CLI Generator](CLI_GENERATOR.md) - Generated CLI structure
- [Local Tool Calling](LOCAL_TOOL_CALLING.md) - Tool execution flow
- [Examples](EXAMPLES.md) - Practical API usage examples
