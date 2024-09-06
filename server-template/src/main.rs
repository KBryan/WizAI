
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use async_trait::async_trait;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EthPrice {
    price: f64,
}

struct AppState {
    http_client: HttpClient,
}

async fn get_eth_price(app_state: web::Data<AppState>) -> impl Responder {
    let client = &app_state.http_client;
    let response = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(price) = json["ethereum"]["usd"].as_f64() {
                    return HttpResponse::Ok().json(EthPrice { price });
                }
            }
            HttpResponse::InternalServerError().body("Failed to parse price")
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch price"),
    }
}

async fn get_eth_price_binance(app_state: web::Data<AppState>) -> impl Responder {
    let client = &app_state.http_client;
    let response = client
        .get("https://api.binance.com/api/v3/ticker/price?symbol=ETHUSDT")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(price) = json["price"].as_str() {
                    if let Ok(price_f64) = price.parse::<f64>() {
                        return HttpResponse::Ok().json(EthPrice { price: price_f64 });
                    }
                }
            }
            HttpResponse::InternalServerError().body("Failed to parse price")
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch price"),
    }
}

async fn get_eth_price_kraken(app_state: web::Data<AppState>) -> impl Responder {
    let client = &app_state.http_client;
    let response = client
        .get("https://api.kraken.com/0/public/Ticker?pair=ETHUSD")
        .send()
        .await;

    match response {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(price) = json["result"]["XETHZUSD"]["c"][0].as_str() {
                    if let Ok(price_f64) = price.parse::<f64>() {
                        return HttpResponse::Ok().json(EthPrice { price: price_f64 });
                    }
                }
            }
            HttpResponse::InternalServerError().body("Failed to parse price")
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch price"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let http_client = HttpClient::new();

    let data = web::Data::new(AppState {
        http_client,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600)
            )
            .app_data(data.clone())
            .route("/eth_price", web::get().to(get_eth_price))
            .route("/eth_price_binance", web::get().to(get_eth_price_binance))
            .route("/eth_price_kraken", web::get().to(get_eth_price_kraken))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
