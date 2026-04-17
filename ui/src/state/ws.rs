use crate::api::types::*;
use leptos::*;
use std::cell::RefCell;
use std::sync::Arc;

thread_local! {
    static MESSAGE_HANDLER: RefCell<Option<Box<dyn Fn(WebSocketMessage) + Send + Sync>>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct WebSocketState {
    pub connected: bool,
    pub last_message: Option<WebSocketMessage>,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            connected: false,
            last_message: None,
        }
    }
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum WebSocketMessage {
    #[serde(rename = "Subscribe")]
    Subscribe { agent_id: String },

    #[serde(rename = "SendMessage")]
    SendMessage { agent_id: String, content: String },

    #[serde(rename = "ChatMessage")]
    ChatMessage { content: String },

    #[serde(rename = "ChatResponse")]
    ChatResponse { content: String, done: bool },

    #[serde(rename = "LeadUpdate")]
    LeadUpdate(Lead),

    #[serde(rename = "CmaUpdate")]
    CmaUpdate { cma_id: String, status: String },
}

pub fn use_ws_state() -> WebSocketState {
    WebSocketState::new()
}

pub fn set_ws_message_handler<F>(_handler: F)
where
    F: Fn(WebSocketMessage) + Send + Sync + 'static,
{
}

pub fn send_ws_message(_msg: WebSocketMessage) {}
