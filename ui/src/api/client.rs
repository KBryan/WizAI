use wasm_bindgen::prelude::*;

const API_BASE: &str = "/api";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn fetch(url: &str, options: JsValue) -> js_sys::Promise;
}

pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: API_BASE.to_string(),
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let opts = web_sys::RequestInit::new();
        opts.method("GET");
        opts.mode(web_sys::RequestMode::Cors);

        let request = web_sys::Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Request error: {:?}", e))?;

        let window = web_sys::window().ok_or("No window")?;
        let resp_value = window
            .fetch_with_request(&request)
            .await
            .map_err(|e| format!("Fetch error: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into()
            .map_err(|_| "Failed to cast response")?;

        if resp.ok() {
            let json_str = resp.text()
                .await
                .map_err(|e| format!("Text error: {:?}", e))?
                .as_string()
                .ok_or("Failed to get text")?;
            
            serde_json::from_str(&json_str)
                .map_err(|e| format!("JSON parse error: {} - Response: {}", e, json_str))
        } else {
            Err(format!("HTTP error: {}", resp.status()))
        }
    }

    pub async fn post<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let body_json = serde_json::to_string(body).map_err(|e| e.to_string())?;
        
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.body(Some(&JsValue::from_str(&body_json)));
        opts.mode(web_sys::RequestMode::Cors);
        
        let request = web_sys::Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Request error: {:?}", e))?;
        
        let headers = request.headers()
            .map_err(|e| format!("Headers error: {:?}", e))?;
        headers.set("Content-Type", "application/json")
            .map_err(|e| format!("Header error: {:?}", e))?;
        
        let window = web_sys::window().ok_or("No window")?;
        let resp_value = window
            .fetch_with_request(&request)
            .await
            .map_err(|e| format!("Fetch error: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into()
            .map_err(|_| "Failed to cast response")?;

        if resp.ok() {
            let json_str = resp.text()
                .await
                .map_err(|e| format!("Text error: {:?}", e))?
                .as_string()
                .ok_or("Failed to get text")?;
            
            serde_json::from_str(&json_str)
                .map_err(|e| format!("JSON parse error: {}", e))
        } else {
            Err(format!("HTTP error: {}", resp.status()))
        }
    }

    pub async fn put<T: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let body_json = serde_json::to_string(body).map_err(|e| e.to_string())?;
        
        let mut opts = web_sys::RequestInit::new();
        opts.method("PUT");
        opts.body(Some(&JsValue::from_str(&body_json)));
        opts.mode(web_sys::RequestMode::Cors);
        
        let request = web_sys::Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Request error: {:?}", e))?;
        
        let headers = request.headers()
            .map_err(|e| format!("Headers error: {:?}", e))?;
        headers.set("Content-Type", "application/json")
            .map_err(|e| format!("Header error: {:?}", e))?;
        
        let window = web_sys::window().ok_or("No window")?;
        let resp_value = window
            .fetch_with_request(&request)
            .await
            .map_err(|e| format!("Fetch error: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into()
            .map_err(|_| "Failed to cast response")?;

        if resp.ok() {
            let json_str = resp.text()
                .await
                .map_err(|e| format!("Text error: {:?}", e))?
                .as_string()
                .ok_or("Failed to get text")?;
            
            serde_json::from_str(&json_str)
                .map_err(|e| format!("JSON parse error: {}", e))
        } else {
            Err(format!("HTTP error: {}", resp.status()))
        }
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
