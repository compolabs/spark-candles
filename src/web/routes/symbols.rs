use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_okapi::openapi;
use serde_json::json;
use std::sync::Arc;

use crate::storage::trading_engine::TradingEngine;

#[openapi]
#[get("/symbols?<symbol>")]
pub async fn get_symbols(
    symbol: Option<String>,
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    if let Some(symbol) = symbol {
        if let Some(config) = trading_engine.configs.get(&symbol) {
            let symbol_data = json!({
                "symbol": config.symbol,
                "ticker": config.symbol,
                "name": config.symbol,
                "description": config.symbol,
                "type_": "crypto",
                "exchange": config.symbol,
                "timezone": "UTC",
                "minmov": 1,
                "pricescale": 100,
                "session": "0000-2400",
                "has_intraday": true,
                "has_daily": true,
                "supported_resolutions": ["1", "5", "15", "30", "60", "D", "W", "M"],
                "intraday_multipliers": ["1", "5", "15", "30", "60"],
                "default_resolution": "D",
                "pricescale": 100000,
                "format": "price"
            });
            return Json(symbol_data);
        } else {
            return Json(json!({ "status": "error", "message": "Symbol not found" }));
        }
    }

    let symbols = trading_engine.get_symbols();
    Json(json!({
        "status": "ok",
        "symbols": symbols
    }))
}

#[openapi]
#[get("/symbols_meta")]
pub async fn get_symbols_meta(
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    let symbols_meta = trading_engine.get_symbols_meta();
    Json(json!({ "status": "ok", "metadata": symbols_meta }))
}
