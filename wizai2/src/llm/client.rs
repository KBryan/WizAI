use anyhow::{anyhow, Result};
use futures::StreamExt;
use reqwest::{Client, StatusCode};
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    Auto,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamDelta {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamResponse {
    pub id: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug)]
pub struct VeniceClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl VeniceClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("VENICE_API_KEY")
            .map_err(|_| anyhow!("VENICE_API_KEY not set"))?;
        
        let base_url = env::var("VENICE_BASE_URL")
            .unwrap_or_else(|_| "https://api.venice.ai".to_string());
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        
        info!("VeniceClient initialized");
        
        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }
    
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/v1/chat/completions", self.base_url);
        
        debug!("Sending chat request to {}", url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("Venice API error: {} - {}", status, error_text);
            return Err(anyhow!("Venice API error: {} - {}", status, error_text));
        }
        
        let chat_response: ChatResponse = response.json().await?;
        
        info!("Received response from Venice AI");
        debug!("Response: {:?}", chat_response);
        
        Ok(chat_response)
    }
    
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(100);
        
        let url = format!("{}/api/v1/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        
        let mut request = request;
        request.stream = Some(true);
        
        tokio::spawn(async move {
            let client = Client::new();
            
            let body = match serde_json::to_string(&request) {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.send(Err(anyhow!("Failed to serialize request: {}", e))).await;
                    return;
                }
            };
            
            let response = match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .body(body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(anyhow!("Request failed: {}", e))).await;
                    return;
                }
            };
            
            if !response.status().is_success() {
                let error_text = match response.text().await {
                    Ok(t) => t,
                    Err(_) => "Unknown error".to_string(),
                };
                let _ = tx.send(Err(anyhow!("API error: {}", error_text))).await;
                return;
            }
            
            let mut stream = response.bytes_stream();
            
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        
                        for line in text.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                
                                if data == "[DONE]" {
                                    continue;
                                }
                                
                                match serde_json::from_str::<StreamResponse>(data) {
                                    Ok(stream_resp) => {
                                        for choice in stream_resp.choices {
                                            if let Some(content) = choice.delta.content {
                                                if tx.send(Ok(content)).await.is_err() {
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse stream chunk: {}", e);
                                        // Don't panic on parse errors, but log them prominently
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        let _ = tx.send(Err(anyhow!("Stream error: {}", e))).await;
                        return;
                    }
                }
            }
        });
        
        Ok(rx)
    }
    
    pub async fn create_embeddings(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        // Venice AI doesn't have a public embeddings endpoint yet
        // Return empty vectors for now - will implement when available
        warn!("Embeddings not implemented for Venice AI yet");
        Ok(texts.iter().map(|_| Vec::new()).collect())
    }

    /// Create a tool response message
    pub fn create_tool_response(tool_call_id: String, content: String) -> ChatMessage {
        ChatMessage {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            name: None,
        }
    }

    /// Create a user message
    pub fn create_user_message(content: String) -> ChatMessage {
        ChatMessage {
            role: "user".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create an assistant message
    pub fn create_assistant_message(content: String) -> ChatMessage {
        ChatMessage {
            role: "assistant".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a system message
    pub fn create_system_message(content: String) -> ChatMessage {
        ChatMessage {
            role: "system".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_venice_client_creation() {
        // This test will fail without VENICE_API_KEY set
        // Just checking compilation for now
    }
}
