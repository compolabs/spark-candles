use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi]
#[get("/config")]
pub async fn get_config() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "supports_search": true,
        "supports_group_request": false,
        "supports_marks": true,
        "supports_timescale_marks": true,
        "supports_time": true,
        "supported_resolutions": ["1", "5", "15", "30", "60", "1D", "1W", "1M"],
        "exchanges": [
            {
                "value": "CryptoExchange",
                "name": "CryptoExchange",
                "desc": "Crypto Exchange"
            }
        ],
        "symbols_types": [
            { "name": "Crypto", "value": "crypto" },
        ]
    }))
}

#[openapi]
#[get("/time")]
pub async fn get_time() -> Json<u64> {
    Json(chrono::Utc::now().timestamp() as u64)
}
