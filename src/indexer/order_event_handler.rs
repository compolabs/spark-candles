use crate::storage::candles::CandleStore;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PangeaOrderEvent {
    pub chain: u64,
    pub block_number: i64,
    pub block_hash: String,
    pub block_timestamp: i64,
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

pub async fn handle_order_event(candle_store: Arc<CandleStore>, event: PangeaOrderEvent, symbol: String) {
    if let Some(event_type) = event.event_type.as_deref() {
        if event_type == "Trade" {
            if let (Some(price), Some(amount)) = (event.price, event.amount) {
                let block_timestamp = event.block_timestamp;
                let intervals = vec![60, 180, 300, 900, 1800, 3600, 86400, 604800, 2592000];
                for &interval in &intervals {
                    candle_store.add_price(
                        &symbol.clone(),
                        interval,
                        price as f64,
                        amount as f64,
                        block_timestamp,
                    );
                }
            } else {
                error!("Incomplete Trade event data: {:?}", event);
            }
        }
    } else {
        error!("Event type is missing in event: {:?}", event);
    }
}
