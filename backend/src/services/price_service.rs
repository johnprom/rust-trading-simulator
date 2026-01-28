use crate::{api_client::ApiClient, models::{PricePoint, Candle}, state::AppState};
use chrono::{Duration as ChronoDuration, Utc};
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

async fn backfill_and_poll_asset(state: AppState, asset: &str) {
    let api_client = ApiClient::new();
    let now = Utc::now();

    // STEP 1: Backfill 1 hour of high-frequency 5-second data (for 1h chart)
    info!("Backfilling {} high-frequency data for last 1 hour...", asset);
    let one_hour_ago = now - ChronoDuration::hours(1);

    match api_client.fetch_historical_candles(asset, one_hour_ago, now, 60).await {
        Ok(candles) => {
            info!("Fetched {} one-minute candles for {} from Coinbase", candles.len(), asset);

            // Interpolate to 5-second intervals (12x multiplication: 60 candles × 12 = 720 points)
            let interpolated = crate::api_client::ApiClient::interpolate_candles(asset, candles, 5);
            info!("Interpolated {} 5-second data points for {}", interpolated.len(), asset);

            for point in interpolated {
                state.add_price_point(point).await;
            }

            info!("Backfilled {} high-frequency data successfully", asset);
        }
        Err(e) => {
            error!("Failed to fetch {} 1h data: {}", asset, e);

            // Fallback: generate 1 hour of simulated 5-second data
            if let Ok(current_price) = api_client.fetch_price(asset, "USD").await {
                let base_price = current_price.price;
                for i in (0..720).rev() {
                    let time_offset = ChronoDuration::seconds((i * 5) as i64);
                    let timestamp = now - time_offset;
                    let trend = (i as f64 / 100.0).sin() * base_price * 0.01;
                    let short_term = (i as f64 / 20.0).sin() * base_price * 0.005;
                    let noise = ((i * 7) as f64).sin() * base_price * 0.0002;
                    let price = base_price + trend + short_term + noise;

                    state.add_price_point(PricePoint {
                        timestamp,
                        asset: asset.to_string(),
                        price,
                    }).await;
                }
                info!("Backfilled {} with simulated high-frequency data", asset);
            }
        }
    }

    // STEP 2: Backfill 24 hours of low-frequency 5-minute candles (for 8h/24h charts)
    info!("Backfilling {} low-frequency candles for last 24 hours...", asset);
    let twenty_four_hours_ago = now - ChronoDuration::hours(24);

    match api_client.fetch_historical_candles(asset, twenty_four_hours_ago, now, 300).await {
        Ok(candles) => {
            info!("Fetched {} five-minute candles for {} from Coinbase", candles.len(), asset);

            // Store 5-minute candles directly (no interpolation) for 8h/24h charts
            for (timestamp, price) in &candles {
                state.add_candle(PricePoint {
                    timestamp: *timestamp,
                    asset: asset.to_string(),
                    price: *price,
                }).await;
            }

            info!("Backfilled {} low-frequency candles successfully", asset);
        }
        Err(e) => {
            error!("Failed to fetch {} 24h candle data: {}", asset, e);

            // Fallback: generate 24 hours of simulated 5-minute candles
            if let Ok(current_price) = api_client.fetch_price(asset, "USD").await {
                let base_price = current_price.price;
                for i in (0..288).rev() {
                    let time_offset = ChronoDuration::minutes((i * 5) as i64);
                    let timestamp = now - time_offset;
                    let trend = (i as f64 / 10.0).sin() * base_price * 0.01;
                    let short_term = (i as f64 / 3.0).sin() * base_price * 0.005;
                    let noise = ((i * 7) as f64).sin() * base_price * 0.0002;
                    let price = base_price + trend + short_term + noise;

                    state.add_candle(PricePoint {
                        timestamp,
                        asset: asset.to_string(),
                        price,
                    }).await;
                }
                info!("Backfilled {} with simulated low-frequency candles", asset);
            }
        }
    }

    // STEP 3: Backfill 1 hour of 1-minute OHLC candles (for 1h candlestick view)
    info!("Backfilling {} 1-minute OHLC candles for last 1 hour...", asset);
    match api_client.fetch_ohlc_candles(asset, one_hour_ago, now, 60).await {
        Ok(candles) => {
            info!("Fetched {} 1-minute OHLC candles for {} from Coinbase", candles.len(), asset);
            for candle in candles {
                state.add_ohlc_candle_1m(candle).await;
            }
            info!("Backfilled {} 1-minute OHLC candles successfully", asset);
        }
        Err(e) => {
            error!("Failed to fetch {} 1h OHLC candle data: {}", asset, e);
        }
    }

    // STEP 4: Backfill 24 hours of 5-minute OHLC candles (for 8h/24h candlestick views)
    info!("Backfilling {} 5-minute OHLC candles for last 24 hours...", asset);
    match api_client.fetch_ohlc_candles(asset, twenty_four_hours_ago, now, 300).await {
        Ok(candles) => {
            info!("Fetched {} 5-minute OHLC candles for {} from Coinbase", candles.len(), asset);
            for candle in candles {
                state.add_ohlc_candle_5m(candle).await;
            }
            info!("Backfilled {} 5-minute OHLC candles successfully", asset);
        }
        Err(e) => {
            error!("Failed to fetch {} 24h OHLC candle data: {}", asset, e);
        }
    }

    let mut interval = time::interval(Duration::from_secs(5));
    info!("Starting live {} price polling (5s interval)", asset);

    let mut tick_counter = 0u32;

    // OHLC accumulators for 1-minute candles
    let mut current_1m_open: Option<f64> = None;
    let mut current_1m_high: f64 = 0.0;
    let mut current_1m_low: f64 = f64::INFINITY;
    let mut current_1m_close: f64 = 0.0;
    let mut current_1m_start: Option<chrono::DateTime<Utc>> = None;

    // OHLC accumulators for 5-minute candles
    let mut current_5m_open: Option<f64> = None;
    let mut current_5m_high: f64 = 0.0;
    let mut current_5m_low: f64 = f64::INFINITY;
    let mut current_5m_close: f64 = 0.0;
    let mut current_5m_start: Option<chrono::DateTime<Utc>> = None;

    loop {
        interval.tick().await;
        tick_counter += 1;

        match api_client.fetch_price(asset, "USD").await {
            Ok(price_point) => {
                let price = price_point.price;
                let timestamp = price_point.timestamp;

                info!("Fetched {} price: ${:.2}", asset, price);
                state.add_price_point(price_point.clone()).await;

                // Update 1-minute OHLC accumulator
                if current_1m_open.is_none() {
                    current_1m_open = Some(price);
                    current_1m_start = Some(timestamp);
                }
                current_1m_high = current_1m_high.max(price);
                current_1m_low = current_1m_low.min(price);
                current_1m_close = price;

                // Update 5-minute OHLC accumulator
                if current_5m_open.is_none() {
                    current_5m_open = Some(price);
                    current_5m_start = Some(timestamp);
                }
                current_5m_high = current_5m_high.max(price);
                current_5m_low = current_5m_low.min(price);
                current_5m_close = price;

                // Every 1 minute (12 ticks at 5-second intervals), emit 1-minute OHLC candle
                if tick_counter % 12 == 0 {
                    if let (Some(open), Some(start_time)) = (current_1m_open, current_1m_start) {
                        let candle = Candle {
                            timestamp: start_time,
                            asset: asset.to_string(),
                            open,
                            high: current_1m_high,
                            low: current_1m_low,
                            close: current_1m_close,
                        };
                        state.add_ohlc_candle_1m(candle).await;
                        info!("Added {} 1-minute OHLC candle: O={:.2} H={:.2} L={:.2} C={:.2}",
                              asset, open, current_1m_high, current_1m_low, current_1m_close);

                        // Reset 1-minute accumulator
                        current_1m_open = None;
                        current_1m_high = 0.0;
                        current_1m_low = f64::INFINITY;
                        current_1m_start = None;
                    }
                }

                // Every 5 minutes (60 ticks at 5-second intervals), emit 5-minute OHLC candle
                if tick_counter % 60 == 0 {
                    // Add to old candle_window for backward compatibility
                    state.add_candle(price_point).await;
                    info!("Added {} 5-minute candle", asset);

                    // Add 5-minute OHLC candle
                    if let (Some(open), Some(start_time)) = (current_5m_open, current_5m_start) {
                        let candle = Candle {
                            timestamp: start_time,
                            asset: asset.to_string(),
                            open,
                            high: current_5m_high,
                            low: current_5m_low,
                            close: current_5m_close,
                        };
                        state.add_ohlc_candle_5m(candle).await;
                        info!("Added {} 5-minute OHLC candle: O={:.2} H={:.2} L={:.2} C={:.2}",
                              asset, open, current_5m_high, current_5m_low, current_5m_close);

                        // Reset 5-minute accumulator
                        current_5m_open = None;
                        current_5m_high = 0.0;
                        current_5m_low = f64::INFINITY;
                        current_5m_start = None;
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch {} price: {}", asset, e);
                // Resiliency: Continue polling despite errors
            }
        }
    }
}

pub async fn start_price_polling(state: AppState) {
    // Spawn separate tasks for each asset
    let btc_state = state.clone();
    tokio::spawn(async move {
        backfill_and_poll_asset(btc_state, "BTC").await;
    });

    let eth_state = state.clone();
    tokio::spawn(async move {
        backfill_and_poll_asset(eth_state, "ETH").await;
    });

    info!("Started price polling for BTC and ETH");
}
