use log::warn;
use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde_json::json;
use std::sync::Arc;

use crate::storage::trading_engine::TradingEngine;

#[derive(serde::Serialize, JsonSchema)]
pub struct AdvancedChartResponse {
    s: String,
    t: Vec<u64>,
    o: Vec<f64>,
    h: Vec<f64>,
    l: Vec<f64>,
    c: Vec<f64>,
    v: Vec<f64>,
}

#[openapi]
#[get("/history?<symbol>&<resolution>&<from>&<to>&<countback>")]
pub async fn get_history(
    symbol: String,
    resolution: Option<String>,
    from: Option<i64>,
    to: Option<i64>,
    countback: Option<usize>,
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<AdvancedChartResponse> {
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
            return Json(AdvancedChartResponse {
                s: "error".to_string(),
                t: vec![],
                o: vec![],
                h: vec![],
                l: vec![],
                c: vec![],
                v: vec![],
            });
        }
    };

    if let Some(store) = trading_engine.get_store(&symbol) {
        let config = trading_engine.configs.get(&symbol);
        let decimals = config.map(|cfg| cfg.decimals).unwrap_or(9); // Дефолтное значение decimals = 9
        let divisor = 10u64.pow(decimals as u32) as f64;

        let mut candles = store.get_candles_in_time_range(&symbol, interval, from, to);

        if let Some(countback) = countback {
            if candles.len() > countback {
                candles = candles[candles.len() - countback..].to_vec();
            }
        }

        if candles.is_empty() {
            return Json(AdvancedChartResponse {
                s: "no_data".to_string(),
                t: vec![],
                o: vec![],
                h: vec![],
                l: vec![],
                c: vec![],
                v: vec![],
            });
        }

        let t: Vec<u64> = candles
            .iter()
            .map(|c| c.timestamp.timestamp() as u64)
            .collect();
        let o: Vec<f64> = candles.iter().map(|c| c.open / divisor).collect();
        let h: Vec<f64> = candles.iter().map(|c| c.high / divisor).collect();
        let l: Vec<f64> = candles.iter().map(|c| c.low / divisor).collect();
        let c: Vec<f64> = candles.iter().map(|c| c.close / divisor).collect();
        let v: Vec<f64> = candles.iter().map(|c| c.volume / divisor).collect();

        return Json(AdvancedChartResponse {
            s: "ok".to_string(),
            t,
            o,
            h,
            l,
            c,
            v,
        });
    }

    Json(AdvancedChartResponse {
        s: "error".to_string(),
        t: vec![],
        o: vec![],
        h: vec![],
        l: vec![],
        c: vec![],
        v: vec![],
    })
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
