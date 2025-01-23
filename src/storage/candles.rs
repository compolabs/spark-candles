use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CandleStore {
    pub candles: RwLock<HashMap<String, HashMap<u64, Vec<Candle>>>>,
}

impl CandleStore {
    pub fn new() -> Self {
        Self {
            candles: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_price(&self, symbol: &str, interval: u64, price: f64, volume: f64, event_time: i64) {
        let mut candles = self.candles.write().unwrap();

        let symbol_candles = candles.entry(symbol.to_string()).or_default();
        let candle_list = symbol_candles.entry(interval).or_default();

        let event_datetime = Utc
            .timestamp_opt(event_time, 0)
            .single()
            .expect("Invalid timestamp");

        let period_start = Self::get_period_start(event_datetime, interval);

        if let Some(last_candle) = candle_list.last_mut() {
            if last_candle.timestamp == period_start {
                last_candle.high = last_candle.high.max(price);
                last_candle.low = last_candle.low.min(price);
                last_candle.close = price;
                last_candle.volume += volume;
                return;
            }
        }

        if let Some(last_candle) = candle_list.last() {
            let mut missing_time = last_candle.timestamp + Duration::seconds(interval as i64);
            let last_close = last_candle.close;

            while missing_time < period_start {
                let empty_candle = Candle {
                    open: last_close,
                    high: last_close,
                    low: last_close,
                    close: last_close,
                    volume: 0.0,
                    timestamp: missing_time,
                };
                candle_list.push(empty_candle);
                missing_time += Duration::seconds(interval as i64);
            }
        }

        let new_candle = Candle {
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            timestamp: period_start,
        };
        candle_list.push(new_candle);

        const MAX_CANDLES: usize = 1000000;
        if candle_list.len() > MAX_CANDLES {
            candle_list.drain(0..(candle_list.len() - MAX_CANDLES));
        }
    }

    fn get_period_start(event_datetime: DateTime<Utc>, interval: u64) -> DateTime<Utc> {
        match interval {
            60 | 180 | 300 | 900 | 3600 => {
                let timestamp = event_datetime.timestamp();
                let period = timestamp - (timestamp % interval as i64);
                DateTime::from_timestamp(period, 0).expect("Invalid timestamp")
            }
            86400 => event_datetime
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            604800 => {
                let naive_date = event_datetime.date_naive();
                let weekday = naive_date.weekday().num_days_from_monday();
                let start_of_week = naive_date - Duration::days(weekday as i64);
                start_of_week
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap()
            }
            _ => {
                let timestamp = event_datetime.timestamp();
                let period = timestamp - (timestamp % interval as i64);
                DateTime::from_timestamp(period, 0).expect("Invalid timestamp")
            }
        }
    }

    pub fn get_candles(&self, symbol: &str, interval: u64, count: usize) -> Vec<Candle> {
        let candles = self.candles.read().unwrap();
        if let Some(symbol_candles) = candles.get(symbol) {
            if let Some(interval_candles) = symbol_candles.get(&interval) {
                return interval_candles.iter().rev().take(count).cloned().collect();
            }
        }
        vec![]
    }

    pub fn get_candles_in_time_range(
        &self,
        symbol: &str,
        interval: u64,
        from: i64,
        to: i64,
    ) -> Vec<Candle> {
        let candles = self.candles.read().unwrap();
        if let Some(interval_candles) = candles
            .get(symbol)
            .and_then(|interval_map| interval_map.get(&interval))
        {
            interval_candles
                .iter()
                .filter(|c| {
                    let timestamp = c.timestamp.timestamp();
                    timestamp >= from && timestamp <= to
                })
                .cloned()
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_min_max_timestamps(&self) -> Option<(i64, i64)> {
        let candles = self.candles.read().unwrap();
        if candles.is_empty() {
            return None;
        }

        let timestamps: Vec<i64> = candles
            .values()
            .flat_map(|interval_map| interval_map.values())
            .flat_map(|candle_list| candle_list.iter().map(|c| c.timestamp.timestamp()))
            .collect();

        let min = timestamps.iter().min().cloned()?;
        let max = timestamps.iter().max().cloned()?;
        Some((min, max))
    }
}

impl Default for CandleStore {
    fn default() -> Self {
        Self::new()
    }
}
