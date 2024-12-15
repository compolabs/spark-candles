use crate::error::Error;
use crate::storage::candles::CandleStore;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone)]
pub struct TradingPairConfig {
    pub symbol: String,
    pub contract_id: String,
    pub start_block: i64,
    pub description: String,
    pub decimals: i32,
}

pub struct TradingEngine {
    pub stores: HashMap<String, Arc<CandleStore>>,
    pub configs: HashMap<String, TradingPairConfig>,
}

impl TradingEngine {
    
    pub fn new(configs: Vec<TradingPairConfig>) -> Self {
        let stores = configs
            .iter()
            .map(|pair| (pair.symbol.clone(), Arc::new(CandleStore::new())))
            .collect();
        let configs = configs
            .into_iter()
            .map(|pair| (pair.symbol.clone(), pair))
            .collect();
        Self { stores, configs }
    }

    pub fn load_config(path: &str) -> Result<Vec<TradingPairConfig>, Error> {
        let config_data = fs::read_to_string(path)?;
        let config: Vec<TradingPairConfig> = serde_json::from_str(&config_data)?;
        Ok(config)
    }

    pub fn get_store(&self, symbol: &str) -> Option<Arc<CandleStore>> {
        self.stores.get(symbol).cloned()
    }

    pub fn get_symbols(&self) -> Vec<serde_json::Value> {
        self.configs
            .values()
            .map(|config| {
                json!({
                    "symbol": config.symbol,
                    "ticker": config.symbol,
                    "name": config.description,
                    "description": config.description,
                    "type_": "crypto", 
                    "exchange": "CryptoExchange", 
                    "timezone": "Etc/UTC", 
                    "minmov": 1, 
                    "pricescale": 100, 
                    "session": "24x7", 
                    "has_intraday": true,
                    "has_daily": true,
                    "supported_resolutions": ["1", "5", "15", "30", "60", "D", "W", "M"],
                    "intraday_multipliers": ["1", "5", "15", "30", "60"],
                    "format": "price"
                })
            })
            .collect()
    }

    
    pub fn get_symbols_meta(&self) -> serde_json::Value {
        let metadata: Vec<_> = self
            .configs
            .values()
            .map(|config| {
                json!({
                    "symbol": config.symbol,
                    "contract_id": config.contract_id,
                    "start_block": config.start_block,
                    "description": config.description,
                })
            })
            .collect();
        json!({ "symbols_meta": metadata })
    }
}
