use rocket::{get, State};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use serde_json::json;
use std::sync::Arc;

use log::info;

use crate::storage::trading_engine::TradingEngine;

#[openapi]
#[get("/symbols?<symbol>")]
pub async fn get_symbols(
    symbol: Option<String>, // Параметр запроса
    trading_engine: &State<Arc<TradingEngine>>,
) -> Json<serde_json::Value> {
    let a = trading_engine.configs.keys();
    info!("==================================");
    info!("a {:?}",a);
    info!("==================================");
    // Если символ указан, ищем его в конфигурации
    if let Some(symbol) = symbol {
        if let Some(config) = trading_engine.configs.get(&symbol) {
            // Формируем объект с полной информацией о символе
            let symbol_data = json!({
                "symbol": config.symbol,
                "ticker": config.symbol,
                "name": config.description,
                "description": config.description,
                "type_": "crypto", // Подставь нужный тип
                "exchange": "CryptoExchange", // Зависит от твоей логики
                "timezone": "Etc/UTC",
                "minmov": 1,
                "pricescale": 100,
                "session": "24x7",
                "has_intraday": true,
                "has_daily": true,
                "supported_resolutions": ["1", "5", "15", "30", "60", "D", "W", "M"],
                "intraday_multipliers": ["1", "5", "15", "30", "60"],
                "format": "price"
            });
            return Json(symbol_data);
        } else {
            // Если символ не найден, возвращаем ошибку
            return Json(json!({ "status": "error", "message": "Symbol not found" }));
        }
    }

    // Если параметр `symbol` не указан, возвращаем все символы
    let symbols = trading_engine.get_symbols();
    Json(json!({
        "status": "ok",
        "symbols": symbols
    }))
}

#[openapi]
#[get("/symbols_meta")]
pub async fn get_symbols_meta(trading_engine: &State<Arc<TradingEngine>>) -> Json<serde_json::Value> {
    let symbols_meta = trading_engine.get_symbols_meta();
    Json(json!({ "status": "ok", "metadata": symbols_meta }))
}
