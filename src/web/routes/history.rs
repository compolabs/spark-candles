use std::sync::Arc;
use log::warn;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::json;
use rocket_okapi::openapi; 

use crate::storage::trading_engine::TradingEngine;

#[openapi]
#[get("/history?<symbol>&<resolution>&<from>&<to>")]
pub async fn get_history(
    symbol: String,
    resolution: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    let resolution = resolution.unwrap_or_else(|| "60".to_string());
    let from = from.unwrap_or(0);
    let to = to.unwrap_or(chrono::Utc::now().timestamp());

    let interval = match resolution.as_str() {
        "1" => 60,
        "5" => 300,
        "15" => 900,
        "30" => 1800,
        "60" => 3600,
        "1D" => 86400,
        "1W" => 604800,
        _ => {
            warn!("Unsupported resolution: {}", resolution);
            return Json(json!({ "status": "error", "message": "Unsupported resolution" }));
        }
    };

    if let Some(store) = trading_engine.get_store(&symbol) {
        let candles = store.get_candles_in_time_range(&symbol, interval, from, to);
        if candles.is_empty() {
            return Json(json!({ "status": "no_data" }));
        }

        let response = json!({
            "status": "ok",
            "t": candles.iter().map(|c| c.timestamp.timestamp() as u64).collect::<Vec<_>>(),
            "o": candles.iter().map(|c| c.open).collect::<Vec<_>>(),
            "h": candles.iter().map(|c| c.high).collect::<Vec<_>>(),
            "l": candles.iter().map(|c| c.low).collect::<Vec<_>>(),
            "c": candles.iter().map(|c| c.close).collect::<Vec<_>>(),
            "v": candles.iter().map(|c| c.volume).collect::<Vec<_>>(),
        });

        return Json(response);
    }

    Json(json!({ "status": "error", "message": "Symbol not found" }))
}

#[openapi]
#[get("/candles?<symbol>&<interval>")]
pub async fn get_all_candles(
    symbol: String,
    interval: u64,
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    if let Some(store) = trading_engine.get_store(&symbol) {
        let candles = store.get_candles(&symbol, interval, usize::MAX);

        if candles.is_empty() {
            return Json(json!({
                "status": "no_data",
                "message": format!("No candles found for symbol={}, interval={}", symbol, interval),
            }));
        }

        let candles_json: Vec<_> = candles
            .iter()
            .map(|c| {
                json!({
                    "timestamp": c.timestamp.timestamp(),
                    "open": c.open,
                    "high": c.high,
                    "low": c.low,
                    "close": c.close,
                    "volume": c.volume,
                })
            })
            .collect();

        return Json(json!({
            "status": "ok",
            "symbol": symbol,
            "interval": interval,
            "candles": candles_json,
        }));
    }

    Json(json!({ "status": "error", "message": "Symbol not found" }))
}
