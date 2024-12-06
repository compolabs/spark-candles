pub mod config;
pub mod history;
pub mod symbols;

use rocket::Route;
use rocket_okapi::{openapi_get_routes, swagger_ui::SwaggerUIConfig};

pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![
        config::get_config,
        config::get_time,
        history::get_history,
        history::get_all_candles,
        symbols::get_symbols,
        symbols::get_symbols_meta
    ]
}


pub fn get_docs() -> SwaggerUIConfig {
    SwaggerUIConfig {
        url: "/openapi.json".to_string(),
        ..Default::default()
    }
}
