use chrono::{DateTime, Duration, NaiveDateTime, Utc, Datelike};
use std::collections::HashMap;
use std::sync::RwLock;

/// Representation of a single candle (OHLCV).
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>, // Start time of the candle interval
}

/// Main store for storing and managing candles.
#[derive(Debug)]
pub struct CandleStore {
    // candles: symbol -> interval -> Vec<Candle>
    pub candles: RwLock<HashMap<String, HashMap<u64, Vec<Candle>>>>,
}

impl CandleStore {
    /// Creates a new empty `CandleStore`.
    pub fn new() -> Self {
        Self {
            candles: RwLock::new(HashMap::new()),
        }
    }

    /// Adds a price and volume to the candle store.
    pub fn add_price(
        &self,
        symbol: &str,
        interval: u64,
        price: f64,
        volume: f64,
        event_time: i64,
    ) {
        let mut candles = self.candles.write().unwrap();

        // Get the candles for the specified symbol and interval
        let symbol_candles = candles.entry(symbol.to_string()).or_default();
        let candle_list = symbol_candles.entry(interval).or_default();

        // Convert event_time to DateTime<Utc>
        let event_datetime =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(event_time, 0), Utc);

        // Calculate the period start time
        let period_start = Self::get_period_start(event_datetime, interval);

        // Check the last candle
        if let Some(last_candle) = candle_list.last_mut() {
            if last_candle.timestamp == period_start {
                // Update the current candle
                last_candle.high = last_candle.high.max(price);
                last_candle.low = last_candle.low.min(price);
                last_candle.close = price;
                last_candle.volume += volume;
                return;
            }
        }

        // Fill missing candles if there is a gap
        if let Some(last_candle) = candle_list.last() {
            // Extract needed values
            let mut missing_time = last_candle.timestamp + Duration::seconds(interval as i64);
            let last_close = last_candle.close;
            // Immutable borrow ends here

            while missing_time < period_start {
                let empty_candle = Candle {
                    open: last_close,
                    high: last_close,
                    low: last_close,
                    close: last_close,
                    volume: 0.0,
                    timestamp: missing_time,
                };
                candle_list.push(empty_candle); // Mutable borrow occurs here
                missing_time += Duration::seconds(interval as i64);
            }
        }

        // Create a new candle
        let new_candle = Candle {
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            timestamp: period_start,
        };
        candle_list.push(new_candle);

        // Limit the number of stored candles
        const MAX_CANDLES: usize = 1000000;
        if candle_list.len() > MAX_CANDLES {
            candle_list.drain(0..(candle_list.len() - MAX_CANDLES));
        }
    }

    /// Calculates the period start time for a given event time and interval.
    fn get_period_start(event_datetime: DateTime<Utc>, interval: u64) -> DateTime<Utc> {
        match interval {
            60 | 180 | 300 | 900 | 3600 => {
                // For minute and hourly intervals
                let timestamp = event_datetime.timestamp();
                let period = timestamp - (timestamp % interval as i64);
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(period, 0), Utc)
            }
            86400 => {
                // Daily intervals
                event_datetime
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap()
            }
            604800 => {
                // Weekly intervals (starting on Monday)
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
                // For other intervals, round down to the interval
                let timestamp = event_datetime.timestamp();
                let period = timestamp - (timestamp % interval as i64);
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(period, 0), Utc)
            }
        }
    }

    /// Retrieves the last `count` candles for the given symbol and interval.
    pub fn get_candles(&self, symbol: &str, interval: u64, count: usize) -> Vec<Candle> {
        let candles = self.candles.read().unwrap();
        if let Some(symbol_candles) = candles.get(symbol) {
            if let Some(interval_candles) = symbol_candles.get(&interval) {
                return interval_candles.iter().rev().take(count).cloned().collect();
            }
        }
        vec![]
    }

    /// Retrieves candles within a specified time range.
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

    /// Returns the minimum and maximum `timestamp` from the store.
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
