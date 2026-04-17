use crate::agent::core::AgentEvent;
use crate::AppState;
use axum::{
    extract::{ws::{Message, WebSocket}, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    Subscribe { agent_id: String },
    SendMessage { agent_id: String, content: String },
    CreateAgent { name: String, role: String, superior_id: Option<String> },
    GetOrganization,
    Ping,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketResponse {
    Connected { session_id: String },
    Event(AgentEvent),
    AgentCreated { agent_id: String },
    OrganizationTree { tree: Vec<AgentNode> },
    Error { message: String },
    Pong,
}

#[derive(Debug, Serialize)]
pub struct AgentNode {
    pub id: String,
    pub name: String,
    pub role: String,
    pub superior: Option<String>,
    pub subordinates: Vec<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let session_id = Uuid::new_v4().to_string();
    info!("WebSocket connection established: {}", session_id);

    // Split the socket into sink and stream
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::channel::<WebSocketResponse>(100);

    // Send initial connection message
    let connected_msg = serde_json::to_string(&WebSocketResponse::Connected {
        session_id: session_id.clone(),
    })
    .unwrap();
    
    if sink.send(Message::Text(connected_msg)).await.is_err() {
        error!("Failed to send connection message");
        return;
    }

    // Spawn task to handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from the stream
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(ws_msg) => {
                        let response = handle_message(ws_msg, state.clone()).await;
                        if let Err(e) = tx.send(response).await {
                            warn!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                        let error_response = WebSocketResponse::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        if let Err(e) = tx.send(error_response).await {
                            warn!("Failed to send error response: {}", e);
                            break;
                        }
                    }
                }
            }
            Message::Close(_) => {
                info!("WebSocket connection closed: {}", session_id);
                break;
            }
            Message::Ping(data) => {
                // Axum automatically handles Ping/Pong, but we can forward it if needed
                debug!("Received ping from session: {}", session_id);
            }
            _ => {}
        }
    }

    // Clean up
    drop(tx);
    send_task.abort();
    info!("WebSocket handler ended for session: {}", session_id);
}

async fn handle_message(msg: WebSocketMessage, state: Arc<AppState>) -> WebSocketResponse {
    match msg {
        WebSocketMessage::Subscribe { agent_id } => {
            debug!("Subscribing to agent {}", agent_id);
            WebSocketResponse::Event(AgentEvent::MessageAdded {
                agent_id: crate::agent::core::AgentId::new(),
                message: crate::agent::core::Message {
                    id: Uuid::new_v4(),
                    role: crate::agent::core::MessageRole::System,
                    content: format!("Subscribed to agent {}", agent_id),
                    timestamp: chrono::Utc::now(),
                    metadata: None,
                },
            })
        }
        
        WebSocketMessage::SendMessage { agent_id, content } => {
            debug!("Sending message to agent {}: {}", agent_id, content);
            
            // Parse the agent_id for the response
            let agent_uuid = match agent_id.parse::<Uuid>() {
                Ok(uuid) => uuid,
                Err(_) => {
                    return WebSocketResponse::Error {
                        message: format!("Invalid agent ID: {}", agent_id),
                    };
                }
            };
            let response_agent_id = crate::agent::core::AgentId(agent_uuid);
            
            // Actually process the message through the agent system
            match process_agent_message(&state, &agent_id, &content).await {
                Ok(response_content) => {
                    WebSocketResponse::Event(AgentEvent::MessageAdded {
                        agent_id: response_agent_id,
                        message: crate::agent::core::Message {
                            id: Uuid::new_v4(),
                            role: crate::agent::core::MessageRole::Assistant,
                            content: response_content,
                            timestamp: chrono::Utc::now(),
                            metadata: None,
                        },
                    })
                }
                Err(e) => {
                    error!("Failed to process agent message: {}", e);
                    WebSocketResponse::Error {
                        message: format!("Failed to process message: {}", e),
                    }
                }
            }
        }
        
        WebSocketMessage::CreateAgent { name, role, superior_id } => {
            debug!("Creating agent {} with role {}", name, role);
            match create_agent(&state, &name, &role, superior_id.as_deref()).await {
                Ok(agent_id) => {
                    WebSocketResponse::AgentCreated { agent_id }
                }
                Err(e) => {
                    error!("Failed to create agent: {}", e);
                    WebSocketResponse::Error {
                        message: format!("Failed to create agent: {}", e),
                    }
                }
            }
        }
        
        WebSocketMessage::GetOrganization => {
            match get_organization_tree(&state).await {
                Ok(tree) => WebSocketResponse::OrganizationTree { tree },
                Err(e) => {
                    error!("Failed to get organization tree: {}", e);
                    WebSocketResponse::Error {
                        message: format!("Failed to get organization: {}", e),
                    }
                }
            }
        }
        
        WebSocketMessage::Ping => WebSocketResponse::Pong,
    }
}

async fn process_agent_message(
    state: &AppState,
    agent_id_str: &str,
    content: &str,
) -> anyhow::Result<String> {
    use crate::agent::core::AgentId;
    
    let agent_uuid = agent_id_str.parse::<Uuid>()
        .map_err(|e| anyhow::anyhow!("Invalid agent ID: {}", e))?;
    let agent_id = AgentId(agent_uuid);
    
    // Verify agent exists
    let registry = state.agent_registry.read().await;
    if registry.get_agent(agent_id).is_none() {
        return Err(anyhow::anyhow!("Agent not found: {}", agent_id_str));
    }
    drop(registry);
    
    // Add user message - needs write access
    {
        let mut registry = state.agent_registry.write().await;
        let user_msg = crate::agent::core::Message {
            id: Uuid::new_v4(),
            role: crate::agent::core::MessageRole::User,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        };
        registry.add_message(agent_id, user_msg).await?;
    }
    
    // Execute task
    let task_request = crate::agent::executor::TaskRequest {
        task: content.to_string(),
        context: None,
    };
    let executor = crate::agent::executor::AgentExecutor::new(agent_id, state.agent_registry.clone()).await?;
    let result = executor.execute_task(task_request).await?;
    
    Ok(result.response)
}

async fn create_agent(
    state: &AppState,
    name: &str,
    role: &str,
    superior_id: Option<&str>,
) -> anyhow::Result<String> {
    use crate::agent::core::{AgentId, AgentRole};
    use std::str::FromStr;
    
    let superior = superior_id
        .map(|s| s.parse::<Uuid>())
        .transpose()
        .map_err(|e| anyhow::anyhow!("Invalid superior ID: {}", e))?
        .map(AgentId);
    
    let role = match role {
        "CEO" => AgentRole::CEO,
        "CTO" => AgentRole::CTO,
        "CFO" => AgentRole::CFO,
        "VP" => AgentRole::VP,
        "Manager" => AgentRole::Manager,
        "Director" => AgentRole::Director,
        "Lead" => AgentRole::Lead,
        "Specialist" => AgentRole::Specialist,
        "Intern" => AgentRole::Intern,
        _ => AgentRole::Specialist,
    };
    
    // Generate a system prompt for the agent
    let system_prompt = format!("You are a {} agent named {}. Help users accomplish their tasks.", role.name(), name);
    
    let mut registry = state.agent_registry.write().await;
    let agent_id = registry.create_agent(name.to_string(), role, superior, system_prompt).await?;
    
    Ok(agent_id.0.to_string())
}

async fn get_organization_tree(state: &AppState) -> anyhow::Result<Vec<AgentNode>> {
    let registry = state.agent_registry.read().await;
    let agent_ids = registry.list_agents();
    
    let mut tree = Vec::new();
    for agent_id in agent_ids {
        if let Some(agent) = registry.get_agent(agent_id) {
            let agent_guard = agent.read().await;
            tree.push(AgentNode {
                id: agent_guard.id.0.to_string(),
                name: agent_guard.name.clone(),
                role: format!("{:?}", agent_guard.role),
                superior: agent_guard.superior.map(|s| s.0.to_string()),
                subordinates: agent_guard.subordinates.iter().map(|s| s.0.to_string()).collect(),
            });
        }
    }
    
    Ok(tree)
}