use rocket::{get, State};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use serde_json::json;
use std::sync::Arc;

use crate::storage::trading_engine::TradingEngine;

#[openapi]
#[get("/symbols")]
pub async fn get_symbols(trading_engine: &State<Arc<TradingEngine>>) -> Json<serde_json::Value> {
    let symbols = trading_engine.get_symbols();
    Json(json!({ "status": "ok", "symbols": symbols }))
}

#[openapi]
#[get("/symbols_meta")]
pub async fn get_symbols_meta(trading_engine: &State<Arc<TradingEngine>>) -> Json<serde_json::Value> {
    let symbols_meta = trading_engine.get_symbols_meta();
    Json(json!({ "status": "ok", "metadata": symbols_meta }))
}
