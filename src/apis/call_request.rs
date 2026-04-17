use crate::models::general::llm::{APIResponse, ChatCompletion, MessageAI};
use dotenv::{dotenv, Error};
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use reqwest::{Client, ClientBuilder, RequestBuilder, Response};
use serde::Deserialize;
use std::env;

// This `derive` requires the `serde` dependency.
#[derive(Deserialize)]
struct Ip {
    origin: String,
}

// Call Venice AI API for chat completions
pub async fn call_llm(
    messages: Vec<MessageAI>,
) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();
    // Extend API information
    let api_key: String =
        env::var("VENICE_API_KEY").expect("VENICE_API_KEY not found in environment variable");
    // conform our endpoint
    let url: &str = "https://api.venice.ai/api/v1/chat/completions";
    // create headers
    let mut headers: HeaderMap = HeaderMap::new();

    // create api key header
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key)).map_err(
            |e: InvalidHeaderValue| -> Box<dyn std::error::Error + Send> { Box::new(e) },
        )?,
    );

    // create client
    let client: Client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e: reqwest::Error| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    // create chat application
    let chat_completion: ChatCompletion = ChatCompletion {
        model: "venice-uncensored".to_string(),
        messages,
        temperature: 0.1,
    };

    // extract api response
    let res: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e: reqwest::Error| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e: reqwest::Error| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    Ok(res.choices[0].message.content.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "Requires VENICE_API_KEY environment variable"]
    async fn tests_call_to_venice() {
        let message = MessageAI {
            role: "user".to_string(),
            content: "Hi there this is a test. What is web3".to_string(),
        };
        let messages = vec![message];
        let res = call_llm(messages).await;
        if let Ok(res_str) = res {
            dbg!(res_str);
            assert!(true)
        } else {
            assert!(false)
        }
    }
}
