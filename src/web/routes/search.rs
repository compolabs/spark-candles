use rocket::{get, State};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use std::sync::Arc;
use serde_json::json;

use crate::storage::trading_engine::TradingEngine;

#[openapi]
#[get("/search?<query>&<type_>&<exchange>&<limit>")]
pub async fn search(
    query: Option<String>,
    type_: Option<String>,
    exchange: Option<String>,
    limit: Option<usize>,
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    
    let configs = &trading_engine.configs;

    
    let query = query.unwrap_or_default().to_lowercase();
    let type_ = type_.unwrap_or_default();
    let exchange = exchange.unwrap_or_default();
    let limit = limit.unwrap_or(30);

    let results: Vec<_> = configs
        .values()
        .filter(|config| {
            
            (config.symbol.to_lowercase().contains(&query)
                || config.description.to_lowercase().contains(&query))
                &&
            
            (type_.is_empty() || type_ == "crypto") 
                &&
            
            (exchange.is_empty() || exchange == "CryptoExchange")
        })
        .take(limit) 
        .map(|config| {
            json!({
                "symbol": config.symbol,
                "full_name": format!("CryptoExchange:{}", config.symbol),
                "description": config.description,
                "exchange": "CryptoExchange",
                "type": "crypto"
            })
        })
        .collect();

    Json(json!(results))
}
