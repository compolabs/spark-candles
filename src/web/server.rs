use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::storage::trading_engine::TradingEngine;
use crate::web::routes::{get_docs, get_routes};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Build, Config, Rocket};
use rocket::{Request, Response};
use rocket_okapi::swagger_ui::make_swagger_ui;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        ));
        res.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ));
    }
}

pub fn rocket(port: u16, trading_engine: Arc<TradingEngine>) -> Rocket<Build> {
    let config = Config {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        port,
        ..Config::default()
    };

    rocket::custom(config)
        .manage(trading_engine)
        .mount("/", get_routes())
        .mount("/swagger", make_swagger_ui(&get_docs()))
        .attach(CORS)
}
