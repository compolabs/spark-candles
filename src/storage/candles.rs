use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;

/// Представление одной свечи (OHLCV).
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>, // Время начала интервала свечи
}

/// Основной стор для хранения и управления свечами.
#[derive(Debug)]
pub struct CandleStore {
    // candles: symbol -> interval -> Vec<Candle>
    pub candles: RwLock<HashMap<String, HashMap<u64, Vec<Candle>>>>,
}

impl CandleStore {
    /// Создает новый пустой `CandleStore`.
    pub fn new() -> Self {
        Self {
            candles: RwLock::new(HashMap::new()),
        }
    }
    /// Добавляет цену и объем в хранилище свечей.
    pub fn add_price(&self, symbol: &str, interval: u64, price: f64, volume: f64, event_time: i64) {
        let mut candles = self.candles.write().unwrap();

        // Достаем свечи для указанного символа и интервала
        let symbol_candles = candles.entry(symbol.to_string()).or_insert_with(HashMap::new);
        let candle_list = symbol_candles.entry(interval).or_insert_with(Vec::new);

        // Рассчитываем начало периода
        let period_start = if interval >= 86400 {
            // Для D/W/M устанавливаем время на 00:00 UTC
            let naive_datetime = NaiveDateTime::from_timestamp(event_time, 0);
            naive_datetime.date().and_hms(0, 0, 0).timestamp()
        } else {
            // Для меньших интервалов оставляем стандартный расчет
            event_time - (event_time % interval as i64)
        };

        // Проверяем последнюю свечу
        if let Some(last_candle) = candle_list.last_mut() {
            let last_timestamp = last_candle.timestamp.timestamp();
            if last_timestamp == period_start {
                // Обновляем текущую свечу
                last_candle.high = last_candle.high.max(price);
                last_candle.low = last_candle.low.min(price);
                last_candle.close = price;
                last_candle.volume += volume;
                return;
            }
        }

        // Заполняем пропущенные свечи, если есть разрыв
        if let Some(last_candle) = candle_list.last() {
            // Копируем данные последней свечи
            let last_close = last_candle.close;
            let mut missing_start = last_candle.timestamp.timestamp() + interval as i64;

            while missing_start < period_start {
                let empty_candle = Candle {
                    open: last_close,
                    high: last_close,
                    low: last_close,
                    close: last_close,
                    volume: 0.0,
                    timestamp: DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(missing_start, 0),
                        Utc,
                    ),
                };
                candle_list.push(empty_candle); // Теперь можем изменять candle_list
                missing_start += interval as i64;
            }
        }

        // Создаем новую свечу
        let new_candle = Candle {
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            timestamp: DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(period_start, 0),
                Utc,
            ),
        };
        candle_list.push(new_candle);

        // Ограничиваем количество хранимых свечей
        const MAX_CANDLES: usize = 1000;
        if candle_list.len() > MAX_CANDLES {
            candle_list.drain(0..(candle_list.len() - MAX_CANDLES));
        }
    }
    /// Получает последние `count` свечей для заданного символа и интервала.
    pub fn get_candles(&self, symbol: &str, interval: u64, count: usize) -> Vec<Candle> {
        let candles = self.candles.read().unwrap();
        if let Some(symbol_candles) = candles.get(symbol) {
            if let Some(interval_candles) = symbol_candles.get(&interval) {
                return interval_candles.iter().rev().take(count).cloned().collect();
            }
        }
        vec![]
    }

    /// Получает свечи в заданном временном диапазоне.
    pub fn get_candles_in_time_range_mils(
        &self,
        symbol: &str,
        interval: u64,
        from: u64,
        to: u64,
    ) -> Vec<Candle> {
        let candles = self.candles.read().unwrap();
        if let Some(interval_candles) = candles
            .get(symbol)
            .and_then(|interval_map| interval_map.get(&interval))
        {
            let mut filtered: Vec<Candle> = interval_candles
                .iter()
                .filter(|c| {
                    let timestamp = c.timestamp.timestamp() as u64;
                    timestamp >= from && timestamp <= to
                })
                .cloned()
                .collect();

            // Упорядочиваем свечи по времени в возрастающем порядке
            filtered.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            filtered
        } else {
            vec![]
        }
    }

    /// Возвращает минимальный и максимальный `timestamp` из хранилища.
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
