use crate::models::general::llm::{Message, ChatCompletion};
use dotenv::dotenv;
use reqwest::{Client, ClientBuilder, RequestBuilder, Response};
use std::env;
use reqwest::header::{HeaderMap,HeaderValue};

// Call large language model (i.e GPT-4)
pub async fn call_gpt(messages:Vec<Message>) {
    dotenv().ok();
    // Extend API information
    let api_key:String = env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY not found in environment variable");
    let api_org:String = env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG not found in environment variable");
    // conform our endpoint
    let url:&str = "https://api.openai.com/v1/chat/completions";
    // create headers
    let mut headers:HeaderMap = HeaderMap::new();

    // create api key header
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap()
    );

    // create open ai org header
    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(api_org.as_str()).unwrap()
    );
    // create client
    let client:Client = Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // create chat application
    let chat_completion:ChatCompletion = ChatCompletion {
        model:"gpt-4o-2024-05-13".to_string(),
        messages,
        temperature:0.1
    };
    //  troubleshooting
    let res_raw:Response = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .unwrap();

    dbg!(res_raw.text().await.unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn tests_call_to_openai() {
        let message = Message {
            role:"user".to_string(),
            content:"Hi there this is a test. What is web3".to_string()
        };
        let messages = vec!(message);
        call_gpt(messages).await;
    }
}