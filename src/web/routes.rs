use std::sync::Arc;

use log::{info, warn};
use rocket::serde::json::Json;
use rocket::{get, Route, State};
use rocket_okapi::swagger_ui::SwaggerUIConfig;
use rocket_okapi::{openapi, openapi_get_routes, JsonSchema};
use serde::Serialize;
use serde_json::{json, Value};

use crate::storage::candles::CandleStore;


#[derive(serde::Serialize, JsonSchema)]
pub struct AdvancedChartResponse {
    s: String,            // Статус ("ok" или "no_data")
    t: Vec<u64>,          // Временные метки
    o: Vec<f64>,          // Открытие
    h: Vec<f64>,          // Максимум
    l: Vec<f64>,          // Минимум
    c: Vec<f64>,          // Закрытие
    v: Vec<f64>,          // Объём
}

#[openapi]
#[get("/timestamps")]
fn get_timestamps(candle_store: &State<Arc<CandleStore>>) -> Json<Option<(i64, i64)>> {
    let min_max = candle_store.get_min_max_timestamps();
    Json(min_max)
}



#[derive(Serialize, JsonSchema)]
struct ConfigResponse {
    supports_search: bool,
    supports_group_request: bool,
    supports_marks: bool,
    supports_timescale_marks: bool,
    supports_time: bool,
    exchanges: Vec<Exchange>,
    symbols_types: Vec<SymbolType>,
    supported_resolutions: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
struct Exchange {
    value: String,
    name: String,
    desc: String,
}

#[derive(Serialize, JsonSchema)]
struct SymbolType {
    name: String,
    value: String,
}

#[openapi]
#[get("/config")]
fn get_config() -> Json<ConfigResponse> {
    let config = ConfigResponse {
        supports_search: true,
        supports_group_request: false,
        supports_marks: true,
        supports_timescale_marks: true,
        supports_time: true,
        exchanges: vec![
            Exchange {
                value: "".to_string(),
                name: "All Exchanges".to_string(),
                desc: "".to_string(),
            },
            Exchange {
                value: "CryptoExchange".to_string(),
                name: "CryptoExchange".to_string(),
                desc: "CryptoExchange".to_string(),
            },
        ],
        symbols_types: vec![
            SymbolType {
                name: "All types".to_string(),
                value: "".to_string(),
            },
            SymbolType {
                name: "Crypto".to_string(),
                value: "crypto".to_string(),
            },
        ],
        supported_resolutions: vec![
            "1".to_string(),
            "5".to_string(),
            "15".to_string(),
            "30".to_string(),
            "60".to_string(),
            "D".to_string(),
            "W".to_string(),
            "M".to_string(),
        ],
    };

    Json(config)
}


#[openapi]
#[get("/time")]
fn get_time() -> Json<u64> {
    let timestamp = chrono::Utc::now().timestamp() as u64;
    Json(timestamp)
}

#[derive(Serialize, JsonSchema)]
pub struct SymbolInfo {
    pub symbol: String,
    pub ticker: String,
    pub name: String,
    pub description: String,
    pub type_: String,
    pub exchange: String,
    pub timezone: String,
    pub minmov: u32,
    pub pricescale: u32,
    pub session: String,
    pub has_intraday: bool,
    pub has_daily: bool,
    pub supported_resolutions: Vec<String>,
    pub intraday_multipliers: Vec<String>,
    pub format: String,
}


#[openapi]
#[get("/symbols?<symbol>")]
fn get_symbols(symbol: Option<String>) -> Json<SymbolInfo> {
    let symbol = symbol.unwrap_or_else(|| "AAPL".to_string()); // Используем `AAPL` по умолчанию

    let symbols = vec![
        SymbolInfo {
            symbol: "BTC/USD".to_string(),
            ticker: "BTC/USD".to_string(),
            name: "Bitcoin / US Dollar".to_string(),
            description: "BTC to USD".to_string(),
            type_: "crypto".to_string(),
            exchange: "CryptoExchange".to_string(),
            timezone: "Etc/UTC".to_string(),
            minmov: 1,
            pricescale: 100,
            session: "24x7".to_string(),
            has_intraday: true,
            has_daily: true,
            supported_resolutions: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
                "D".to_string(),
                "W".to_string(),
                "M".to_string(),
            ],
            intraday_multipliers: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
            ],
            format: "price".to_string(),
        },
        SymbolInfo {
            symbol: "ETH/USD".to_string(),
            ticker: "ETH/USD".to_string(),
            name: "Ethereum / US Dollar".to_string(),
            description: "ETH to USD".to_string(),
            type_: "crypto".to_string(),
            exchange: "CryptoExchange".to_string(),
            timezone: "Etc/UTC".to_string(),
            minmov: 1,
            pricescale: 100,
            session: "24x7".to_string(),
            has_intraday: true,
            has_daily: true,
            supported_resolutions: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
                "D".to_string(),
                "W".to_string(),
                "M".to_string(),
            ],
            intraday_multipliers: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
            ],
            format: "price".to_string(),
        },
        // Символ для AAPL
        SymbolInfo {
            symbol: "AAPL".to_string(),
            ticker: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            description: "Apple Stock".to_string(),
            type_: "stock".to_string(),
            exchange: "NASDAQ".to_string(),
            timezone: "America/New_York".to_string(),
            minmov: 1,
            pricescale: 100,
            session: "0930-1600".to_string(),
            has_intraday: true,
            has_daily: true,
            supported_resolutions: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
                "D".to_string(),
                "W".to_string(),
                "M".to_string(),
            ],
            intraday_multipliers: vec![
                "1".to_string(),
                "5".to_string(),
                "15".to_string(),
                "30".to_string(),
                "60".to_string(),
            ],
            format: "price".to_string(),
        },
    ];

    // Поиск символа
    let result = symbols.into_iter().find(|s| s.symbol == symbol).unwrap_or_else(|| SymbolInfo {
        symbol: "AAPL".to_string(),
        ticker: "AAPL".to_string(),
        name: "Apple Inc.".to_string(),
        description: "Apple Stock".to_string(),
        type_: "stock".to_string(),
        exchange: "NASDAQ".to_string(),
        timezone: "America/New_York".to_string(),
        minmov: 1,
        pricescale: 100,
        session: "0930-1600".to_string(),
        has_intraday: true,
        has_daily: true,
        supported_resolutions: vec![
            "1".to_string(),
            "5".to_string(),
            "15".to_string(),
            "30".to_string(),
            "60".to_string(),
            "D".to_string(),
            "W".to_string(),
            "M".to_string(),
        ],
        intraday_multipliers: vec![
            "1".to_string(),
            "5".to_string(),
            "15".to_string(),
            "30".to_string(),
            "60".to_string(),
        ],
        format: "price".to_string(),
    });

    Json(result)
}

#[openapi]
#[get("/history?<symbol>&<resolution>&<from>&<to>")]
fn get_history(
    candle_store: &State<Arc<CandleStore>>,
    symbol: Option<String>,
    resolution: Option<u64>,
    from: Option<u64>,
    to: Option<u64>,
) -> Json<AdvancedChartResponse> {
    // Логируем входящие параметры
    let symbol = symbol.unwrap_or_default();
    let resolution = resolution.unwrap_or(60);
    let from = from.unwrap_or(0);
    let to = to.unwrap_or(chrono::Utc::now().timestamp() as u64);

    info!(
        "Received /history request: symbol={}, resolution={}, from={}, to={}",
        symbol, resolution, from, to
    );

    // Проверяем данные в CandleStore
    let candles = candle_store.get_candles_in_time_range_mils(&symbol, resolution, from, to);

    if candles.is_empty() {
        warn!(
            "No candles found for symbol={}, resolution={}, from={}, to={}",
            symbol, resolution, from, to
        );
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

    // Формируем ответ
    let t: Vec<u64> = candles.iter().map(|c| c.timestamp.timestamp() as u64).collect();
    let o: Vec<f64> = candles.iter().map(|c| c.open).collect();
    let h: Vec<f64> = candles.iter().map(|c| c.high).collect();
    let l: Vec<f64> = candles.iter().map(|c| c.low).collect();
    let c: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let v: Vec<f64> = candles.iter().map(|c| c.volume).collect();

    info!(
        "Returning {} candles for symbol={}, resolution={}, from={}, to={}",
        candles.len(),
        symbol,
        resolution,
        from,
        to
    );

    Json(AdvancedChartResponse {
        s: "ok".to_string(),
        t,
        o,
        h,
        l,
        c,
        v,
    })
}

#[openapi]
#[get("/candles?<symbol>&<interval>&<from>&<to>")]
pub fn get_candles(
    candle_store: &State<Arc<CandleStore>>,
    symbol: String,
    interval: u64,
    from: u64,
    to: u64,
) -> Json<AdvancedChartResponse> {
    let candles = candle_store
        .get_candles_in_time_range_mils(&symbol, interval, from, to);
    //info!("=====================");
    //info!("candle_store: {:?}", candle_store);
    //info!("=====================");

    if candles.is_empty() {
        Json(AdvancedChartResponse {
            s: "no_data".to_string(),
            t: vec![],
            o: vec![],
            h: vec![],
            l: vec![],
            c: vec![],
            v: vec![],
        })
    } else {
        let t: Vec<u64> = candles.iter().map(|c| c.timestamp.timestamp() as u64).collect();
        let o: Vec<f64> = candles.iter().map(|c| c.open).collect();
        let h: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let l: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let c: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let v: Vec<f64> = candles.iter().map(|c| c.volume).collect();

        Json(AdvancedChartResponse {
            s: "ok".to_string(),
            t,
            o,
            h,
            l,
            c,
            v,
        })
    }
}


#[openapi]
#[get("/symbols_meta")]
fn get_symbols_meta(candle_store: &State<Arc<CandleStore>>) -> Json<Value> {
    let candles_lock = candle_store.candles.read().unwrap(); // Получаем блокировку для чтения
    let symbols_meta: Vec<_> = candles_lock
        .iter()
        .map(|(symbol, intervals_map)| {
            let intervals: Vec<u64> = intervals_map.keys().cloned().collect();
            json!({
                "symbol": symbol,
                "intervals": intervals,
            })
        })
        .collect();

    Json(json!({ "symbols": symbols_meta }))
}

#[openapi]
#[get("/timestamps_meta?<symbol>&<interval>")]
fn get_timestamps_meta(
    candle_store: &State<Arc<CandleStore>>,
    symbol: String,
    interval: u64,
) -> Json<Value> {
    // Получаем все свечи в заданном интервале
    let candles = candle_store
        .get_candles_in_time_range_mils(&symbol, interval, 0, u64::MAX);

    if candles.is_empty() {
        return Json(json!({ "status": "no_data" }));
    }

    // Извлекаем временные метки
    let mut timestamps: Vec<u64> = candles.iter().map(|c| c.timestamp.timestamp() as u64).collect();

    // Гарантируем сортировку временных меток
    timestamps.sort();

    let mut meta = json!({
        "status": "ok",
        "count": candles.len(),
        "first_timestamp": timestamps.first(),
        "last_timestamp": timestamps.last(),
        "timestamps": timestamps,
    });

    // Проверяем, есть ли дублирующиеся временные метки
    let mut unique_timestamps = timestamps.clone();
    unique_timestamps.dedup();
    if unique_timestamps.len() != timestamps.len() {
        meta["duplicates"] = json!(true);
    }

    Json(meta)
}


pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![
        get_config,
        get_time,
        get_symbols,
        get_candles,
        get_timestamps,
        get_history,
        get_symbols_meta,
        get_timestamps_meta
    ]
}

pub fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}
