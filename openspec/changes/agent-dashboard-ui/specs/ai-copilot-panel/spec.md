# Spec: AI Copilot Panel

## Purpose

Natural language interface allowing agents to interact with the AI team via chat, execute commands, and view streaming responses.

## Requirements

### Requirement: Chat Interface

The copilot SHALL provide a chat interface for natural language interaction.

#### Scenario: Send message
- **WHEN** an agent types a message and presses Enter or clicks Send
- **THEN** the message SHALL appear in chat history (user style)
- **AND** an AI typing indicator SHALL appear
- **AND** the message SHALL be sent to the backend agent for processing

#### Scenario: Receive response
- **WHEN** the AI agent responds with text
- **THEN** the response SHALL stream into the chat (token by token)
- **AND** the agent SHALL be notified when complete

#### Scenario: Chat history
- **WHEN** an agent views the copilot panel
- **THEN** previous messages in the session SHALL be visible
- **AND** new session SHALL start with empty history

### Requirement: Command Recognition

The copilot SHALL recognize and execute structured commands.

#### Scenario: Lead command
- **WHEN** agent types "show me hot leads from this week"
- **THEN** the AI SHALL parse intent and return filtered lead results
- **AND** display results in a formatted table or cards

#### Scenario: CMA command
- **WHEN** agent types "generate CMA for 123 Main St"
- **THEN** the AI SHALL trigger CMA generation flow
- **AND** display progress updates
- **AND** show results when complete

#### Scenario: Market command
- **WHEN** agent types "market trends for Pickering"
- **THEN** the AI SHALL fetch and display market data
- **AND** show charts/tables as appropriate

### Requirement: Context-Aware Suggestions

The copilot SHALL provide contextual suggestions based on current page.

#### Scenario: On leads page
- **WHEN** agent opens copilot while on Leads page
- **THEN** suggestions SHALL include: "Show my hot leads", "Follow up overdue leads", "Generate lead report"

#### Scenario: On CMA page
- **WHEN** agent opens copilot while on CMA page
- **THEN** suggestions SHALL include: "Compare these properties", "Update price recommendation"

### Requirement: Collapsible Panel

The copilot panel SHALL be collapsible to maximize workspace.

#### Scenario: Minimize copilot
- **WHEN** agent clicks the minimize button
- **THEN** the copilot panel SHALL slide down and collapse
- **AND** only a floating "AI" button SHALL remain visible

#### Scenario: Expand copilot
- **WHEN** agent clicks the floating AI button
- **THEN** the copilot panel SHALL slide up and expand
- **AND** chat history SHALL be preserved

### Requirement: Streaming Response

The copilot SHALL display AI responses in real-time as they stream.

#### Scenario: Streaming text
- **WHEN** AI responds with "Thinking..."
- **THEN** each token SHALL appear immediately
- **AND** the chat container SHALL auto-scroll to bottom
- **AND** markdown formatting SHALL be rendered

## UI Components

| Component | Description |
|-----------|-------------|
| ChatHistory | Scrollable message list |
| ChatMessage | Individual message bubble (user/AI) |
| ChatInput | Text input with send button |
| TypingIndicator | "AI is typing..." animation |
| CommandSuggestions | Quick action chips |

## State

| State | Type | Description |
|-------|------|-------------|
| `messages` | Signal<Vec<ChatMessage>> | Chat history |
| `is_streaming` | Signal<bool> | AI response in progress |
| `suggestions` | Signal<Vec<String>> | Contextual suggestions |
| `panel_expanded` | Signal<bool> | Panel visibility |

## Commands Supported

| Command Pattern | Action |
|-----------------|--------|
| "show leads [filter]" | List leads with optional filter |
| "generate CMA for [address]" | Start CMA generation |
| "market trends for [area]" | Fetch market data |
| "draft [type] for [lead]" | Generate communication draft |
| "compare [address1] vs [address2]" | Property comparison |
| "follow up [lead]" | Create follow-up task |
