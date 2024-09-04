use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageAI {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatCompletion {
    pub model: String,
    pub messages: Vec<MessageAI>,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIMessage {
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIChoice {
    pub message: APIMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct APIResponse {
    pub choices: Vec<APIChoice>,
}
