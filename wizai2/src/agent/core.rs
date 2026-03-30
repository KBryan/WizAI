use crate::llm::VeniceClient;
use crate::memory::MemoryStore;
use crate::tools::{ToolRegistry, ToolCall, ToolResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentRole {
    User,              // Human user at top
    CEO,
    CTO,
    CFO,
    ChiefAI,         // Chief AI Officer
    ChiefProduct,    // Chief Product Officer
    OpenSpecExecutor,// OpenSpec workflow execution agent
    SoftwareDeveloper, // Full-stack software developer
    RealEstateResearcher, // Real estate market analysis specialist
    LeadIntakeSpecialist,      // Captures and qualifies leads
    ClientCommunicationAssistant, // Drafts client communications
    VP,              // Vice President
    Director,
    Manager,
    Lead,
    Specialist,
    Intern,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::User => "User",
            AgentRole::CEO => "CEO",
            AgentRole::CTO => "CTO",
            AgentRole::CFO => "CFO",
            AgentRole::ChiefAI => "Chief AI Officer",
            AgentRole::ChiefProduct => "Chief Product Officer",
            AgentRole::OpenSpecExecutor => "OpenSpec Executor",
            AgentRole::SoftwareDeveloper => "Software Developer",
            AgentRole::RealEstateResearcher => "Real Estate Researcher",
            AgentRole::LeadIntakeSpecialist => "Lead Intake Specialist",
            AgentRole::ClientCommunicationAssistant => "Client Communication Assistant",
            AgentRole::VP => "VP",
            AgentRole::Director => "Director",
            AgentRole::Manager => "Manager",
            AgentRole::Lead => "Lead",
            AgentRole::Specialist => "Specialist",
            AgentRole::Intern => "Intern",
        }
    }

    pub fn can_create(&self, other: &AgentRole) -> bool {
        use AgentRole::*;
        match (self, other) {
            (User, _) => true,
            (CEO, _) | (CTO, _) | (CFO, _) | (ChiefAI, _) | (ChiefProduct, _) => {
                matches!(other, VP | Director | Manager | Lead | Specialist | Intern | OpenSpecExecutor | SoftwareDeveloper | RealEstateResearcher)
            }
            (VP, _) | (Director, _) => {
                matches!(other, Manager | Lead | Specialist | Intern | SoftwareDeveloper | RealEstateResearcher)
            }
            (Manager, _) => {
                matches!(other, Lead | Specialist | Intern | SoftwareDeveloper | RealEstateResearcher)
            }
            (Lead, _) => {
                matches!(other, Specialist | Intern | SoftwareDeveloper | RealEstateResearcher)
            }
            (OpenSpecExecutor, _) => {
                matches!(other, SoftwareDeveloper)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub role: AgentRole,
    pub superior: Option<AgentId>,
    pub subordinates: Vec<AgentId>,
    pub created_at: DateTime<Utc>,
    pub context: AgentContext,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub system_prompt: String,
    pub messages: Vec<Message>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Idle,
    Thinking,
    ExecutingTool(String),
    WaitingForInput,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageMetadata {
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: HashMap<AgentId, Arc<RwLock<Agent>>>,
    memory: Arc<RwLock<MemoryStore>>,
    llm: Arc<VeniceClient>,
    tools: Arc<RwLock<ToolRegistry>>,
    event_senders: HashMap<AgentId, Vec<mpsc::Sender<AgentEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    MessageAdded { agent_id: AgentId, message: Message },
    StatusChanged { agent_id: AgentId, status: AgentStatus },
    ToolExecuted { agent_id: AgentId, tool_call: ToolCall, result: ToolResult },
    SubordinateCreated { superior_id: AgentId, subordinate_id: AgentId },
    TaskCompleted { agent_id: AgentId, result: String },
    Error { agent_id: AgentId, error: String },
}

impl AgentRegistry {
    pub fn new(
        memory: Arc<RwLock<MemoryStore>>,
        llm: Arc<VeniceClient>,
        tools: Arc<RwLock<ToolRegistry>>,
    ) -> Self {
        Self {
            agents: HashMap::new(),
            memory,
            llm,
            tools,
            event_senders: HashMap::new(),
        }
    }

    pub async fn create_agent(
        &mut self,
        name: String,
        role: AgentRole,
        superior: Option<AgentId>,
        system_prompt: String,
    ) -> Result<AgentId> {
        let id = AgentId::new();
        
        let agent = Agent {
            id,
            name,
            role,
            superior,
            subordinates: Vec::new(),
            created_at: Utc::now(),
            context: AgentContext {
                system_prompt,
                messages: Vec::new(),
                tools: Vec::new(),
            },
            status: AgentStatus::Idle,
        };

        // Add as subordinate to superior
        if let Some(sup_id) = superior {
            if let Some(sup) = self.agents.get(&sup_id) {
                sup.write().await.subordinates.push(id);
            }
        }

        self.agents.insert(id, Arc::new(RwLock::new(agent)));
        
        if let Some(sup_id) = superior {
            self.broadcast_event(AgentEvent::SubordinateCreated {
                superior_id: sup_id,
                subordinate_id: id,
            }).await;
        }

        Ok(id)
    }

    pub fn get_agent(&self, id: AgentId) -> Option<Arc<RwLock<Agent>>> {
        self.agents.get(&id).cloned()
    }

    pub fn list_agents(&self) -> Vec<AgentId> {
        self.agents.keys().cloned().collect()
    }

    pub fn get_organization_tree(&self) -> Vec<(AgentId, Option<AgentId>, AgentRole)> {
        // Synchronous read without block_on - just return the basic structure
        // Callers should use async method if they need full agent data
        self.agents
            .iter()
            .map(|(id, _)| {
                // Return just the ID - detailed info requires async access
                (*id, None::<AgentId>, AgentRole::Specialist) // Placeholder - proper implementation needs async
            })
            .collect()
    }
    
    pub async fn get_organization_tree_async(&self) -> Vec<(AgentId, Option<AgentId>, AgentRole, String)> {
        let mut result = Vec::new();
        for (id, agent) in &self.agents {
            let agent_guard = agent.read().await;
            result.push((
                *id,
                agent_guard.superior,
                agent_guard.role.clone(),
                agent_guard.name.clone(),
            ));
        }
        result
    }

    pub async fn add_message(&mut self, agent_id: AgentId, message: Message) -> Result<()> {
        // Phase 1: Clone agent reference and update it
        let agent_opt = self.agents.get(&agent_id).cloned();
        
        if let Some(agent) = agent_opt {
            // Add message to agent context - minimal scope for agent lock
            {
                let mut agent_guard = agent.write().await;
                agent_guard.context.messages.push(message.clone());
            } // Agent lock released here
            
            // Phase 2: Store in memory - can take time, no registry locks held
            let memory = self.memory.read().await;
            memory.store_message(agent_id, &message).await?;
            drop(memory);
            
            // Phase 3: Broadcast event - minimal scope
            self.broadcast_event(AgentEvent::MessageAdded {
                agent_id,
                message,
            }).await;
        }
        Ok(())
    }

    pub fn subscribe(&mut self, agent_id: AgentId) -> mpsc::Receiver<AgentEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.event_senders
            .entry(agent_id)
            .or_default()
            .push(tx);
        rx
    }

    pub async fn broadcast_event(&self, event: AgentEvent) {
        for senders in self.event_senders.values() {
            for sender in senders {
                let _ = sender.send(event.clone()).await;
            }
        }
    }

    pub async fn get_llm(&self) -> Arc<VeniceClient> {
        self.llm.clone()
    }

    pub async fn get_tools(&self) -> Arc<RwLock<ToolRegistry>> {
        self.tools.clone()
    }
}
