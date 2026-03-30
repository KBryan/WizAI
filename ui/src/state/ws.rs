use leptos::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::api::types::*;

#[derive(Clone)]
pub struct WebSocketProvider {
    connected: ReadSignal<bool>,
    last_message: ReadSignal<Option<WebSocketMessage>>,
    tx: Arc<tokio::sync::mpsc::UnboundedSender<WebSocketMessage>>,
}

impl WebSocketProvider {
    pub fn new() -> Self {
        let (connected, set_connected) = create_signal(false);
        let (last_message, set_last_message) = create_signal(None);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();
        
        let tx_clone = tx.clone();
        let set_last = set_last_message;
        
        on_mount(move || {
            let ws_url = format!("ws://{}/ws", web_sys::window().unwrap().location().host().unwrap());
            
            spawn_local(async move {
                loop {
                    match web_sys::WebSocket::new(&ws_url) {
                        Ok(ws) => {
                            let ws_clone = ws.clone();
                            
                            ws.set_onopen(Some(Box::new(move |_| {
                                set_connected.set(true);
                                tracing::info!("WebSocket connected");
                            }).as_ref().unchecked_ref()));
                            
                            ws.set_onmessage(Some(Box::new(move |e| {
                                if let Ok(data) = e.data().as_string() {
                                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&data) {
                                        if let Some(msg_type) = msg.get("type").and_then(|t| t.as_str()) {
                                            match msg_type {
                                                "ChatResponse" => {
                                                    let content = msg.get("content")
                                                        .and_then(|c| c.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let done = msg.get("done")
                                                        .and_then(|d| d.as_bool())
                                                        .unwrap_or(false);
                                                    set_last(Some(WebSocketMessage::ChatResponse {
                                                        content,
                                                        done,
                                                    }));
                                                }
                                                "LeadUpdate" => {
                                                    if let Some(lead) = serde_json::from_value::<Lead>(msg.get("data").cloned().unwrap_or_default()).ok() {
                                                        set_last(Some(WebSocketMessage::LeadUpdate(lead)));
                                                    }
                                                }
                                                "CmaUpdate" => {
                                                    let cma_id = msg.get("cma_id")
                                                        .and_then(|id| id.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let status = msg.get("status")
                                                        .and_then(|s| s.as_str())
                                                        .unwrap_or("draft")
                                                        .to_string();
                                                    set_last(Some(WebSocketMessage::CmaUpdate { cma_id, status }));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }).as_ref().unchecked_ref()));
                            
                            ws.set_onclose(Some(Box::new(move |_| {
                                set_connected.set(false);
                                tracing::info!("WebSocket disconnected");
                            }).as_ref().unchecked_ref()));
                            
                            loop {
                                if let Some(msg) = rx.recv().await {
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        if let Err(e) = ws_clone.send_with_str(&json) {
                                            tracing::error!("Send error: {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("WebSocket error: {:?}", e);
                            set_connected.set(false);
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        }
                    }
                }
            });
        });
        
        Self {
            connected,
            last_message,
            tx: Arc::new(tx),
        }
    }
    
    pub fn send(&self, msg: WebSocketMessage) {
        let _ = self.tx.send(msg);
    }
}

impl Default for WebSocketProvider {
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
    LeadUpdate(crate::api::types::Lead),
    
    #[serde(rename = "CmaUpdate")]
    CmaUpdate { cma_id: String, status: String },
}

pub fn use_ws_state() -> WebSocketProvider {
    use_context().unwrap_or_else(|| WebSocketProvider::new())
}
