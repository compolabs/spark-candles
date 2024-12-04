use crate::storage::candles::CandleStore;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
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

pub async fn handle_order_event(
    candle_store: Arc<CandleStore>,
    event: PangeaOrderEvent,
) {
    if let Some(event_type) = event.event_type.as_deref() {
        if event_type == "Trade" {
                if let (Some(price), Some(amount)) = (event.price, event.amount) {
                    let asset = "AAPL"; 
                    let genesis_block = 1; 
                    let genesis_timestamp = 1724996333; 

                    
                    let event_time_old = genesis_timestamp + (event.block_number - genesis_block);
                    let block_timestamp = event.block_timestamp;
                    info!("================new event");
                    info!("genesis_timestamp: {:?}", genesis_timestamp);
                    info!("event_block_number: {:?}", event.block_number);
                    info!("event_time: {:?}", event_time_old);
                    info!("block_timestamp: {:?}", block_timestamp);
                    info!("----------------new event");
                    let intervals = vec![60, 180, 300, 900, 1800, 3600, 86400, 604800, 2592000];
                    for &interval in &intervals {
                        println!("=====================< INTERVAL {:?}", interval);
                        println!(
                            "Adding price to CandleStore: symbol={}, interval={}, price={}, amount={}, event_time={}",
                            asset, interval, price, amount, block_timestamp 
                        );
                        println!("===================== INTERVAL {:?} >", interval);
                        candle_store.add_price(asset, interval, price as f64, amount as f64, block_timestamp);
                    }
                } else {
                    error!("Incomplete Trade event data: {:?}", event);
                }
            }
    } else {
        error!("Event type is missing in event: {:?}", event);
    }
}

