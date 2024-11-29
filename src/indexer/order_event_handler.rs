use crate::storage::candles::CandleStore;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct PangeaOrderEvent {
    pub chain: u64,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_hash: String,
    pub transaction_index: u64,
    pub log_index: u64,
    pub market_id: String,
    pub order_id: String,
    pub event_type: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<u128>,
    pub asset_type: Option<String>,
    pub order_type: Option<String>,
    pub price: Option<u128>,
    pub user: Option<String>,
    pub order_matcher: Option<String>,
    pub owner: Option<String>,
    pub limit_type: Option<String>,
}

pub async fn handle_order_event(
    candle_store: Arc<CandleStore>,
    event: PangeaOrderEvent,
) {
    if let Some(event_type) = event.event_type.as_deref() {
        match event_type {
            "Trade" => {
                if let (Some(price), Some(amount)) = (event.price, event.amount) {
                    let asset = "AAPL"; // Фиксированный символ
                    let genesis_block = 0; // Блок, соответствующий `genesis_timestamp`
                    let genesis_timestamp = 1724996333; // Unix timestamp первого блока

                    // Вычисляем точное время события
                    let event_time = genesis_timestamp + (event.block_number - genesis_block) as i64;

                    info!(
                        "Processing Trade event for asset: {}, price: {}, amount: {}, time: {}",
                        asset, price, amount, event_time
                    );

                    // Поддерживаемые интервалы свечей (1m, 3m, 5m, 15m, 1h, 1d, 1w)
                    let intervals = vec![60, 180, 300, 900, 3600, 86400, 604800];
                    for &interval in &intervals {
                        candle_store.add_price(asset, interval, price as f64, amount as f64, event_time);
                    }
                } else {
                    error!("Incomplete Trade event data: {:?}", event);
                }
            }
            _ => {}
        }
    } else {
        error!("Event type is missing in event: {:?}", event);
    }
}

