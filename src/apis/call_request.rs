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

// Call large language model (i.e GPT-4 currently using gpt-4o-2024-05-13)
pub async fn call_gpt(
    messages: Vec<MessageAI>,
) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();
    // Extend API information
    let api_key: String =
        env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY not found in environment variable");
    let api_org: String =
        env::var("OPEN_AI_ORG").expect("OPEN_AI_ORG not found in environment variable");
    // conform our endpoint
    let url: &str = "https://api.openai.com/v1/chat/completions";
    // create headers
    let mut headers: HeaderMap = HeaderMap::new();

    // create api key header
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key)).map_err(
            |e: InvalidHeaderValue| -> Box<dyn std::error::Error + Send> { Box::new(e) },
        )?,
    );

    // create open ai org header
    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(api_org.as_str()).map_err(
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
        model: "gpt-4o-2024-05-13".to_string(),
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
    async fn tests_call_to_openai() {
        let message = MessageAI {
            role: "user".to_string(),
            content: "Hi there this is a test. What is web3".to_string(),
        };
        let messages = vec![message];
        let res = call_gpt(messages).await;
        if let Ok(res_str) = res {
            dbg!(res_str);
            assert!(true)
        } else {
            assert!(false)
        }
    }
}
