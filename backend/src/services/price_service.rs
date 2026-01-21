use crate::{api_client::ApiClient, models::PricePoint, state::AppState};
use chrono::{Duration as ChronoDuration, Utc};
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

pub async fn start_price_polling(state: AppState) {
    let api_client = ApiClient::new();

    info!("Backfilling price data for last hour...");

    // Fetch real historical data from Coinbase and interpolate to 5-second intervals
    let now = Utc::now();
    let one_hour_ago = now - ChronoDuration::hours(1);

    match api_client.fetch_historical_candles(one_hour_ago, now, 60).await {
        Ok(candles) => {
            info!("Fetched {} minute candles from Coinbase", candles.len());

            // Interpolate to 5-second intervals
            let interpolated = crate::api_client::ApiClient::interpolate_candles(candles, 5);

            info!("Interpolated to {} data points", interpolated.len());

            // Add all interpolated points to state
            for point in interpolated {
                state.add_price_point(point).await;
            }

            info!("Backfilled historical price data successfully");
        }
        Err(e) => {
            error!("Failed to fetch historical data: {}", e);
            info!("Falling back to simulated data");

            // Fallback: generate simulated data if API fails
            match api_client.fetch_btc_price().await {
                Ok(current_price) => {
                    let base_price = current_price.price;

                    for i in (0..720).rev() {
                        let time_offset = ChronoDuration::seconds((i * 5) as i64);
                        let timestamp = now - time_offset;

                        let trend = (i as f64 / 100.0).sin() * base_price * 0.01;
                        let short_term = (i as f64 / 20.0).sin() * base_price * 0.005;
                        let noise = ((i * 7) as f64).sin() * base_price * 0.0002;

                        let price = base_price + trend + short_term + noise;

                        let point = PricePoint {
                            timestamp,
                            asset: "BTC".to_string(),
                            price,
                        };

                        state.add_price_point(point).await;
                    }

                    info!("Backfilled with simulated data");
                }
                Err(e2) => {
                    error!("Failed to fetch current price for simulation: {}", e2);
                }
            }
        }
    }

    let mut interval = time::interval(Duration::from_secs(5));
    info!("Starting live price polling (5s interval)");

    loop {
        interval.tick().await;

        match api_client.fetch_btc_price().await {
            Ok(price_point) => {
                info!("Fetched BTC price: ${:.2}", price_point.price);
                state.add_price_point(price_point).await;
            }
            Err(e) => {
                error!("Failed to fetch price: {}", e);
                // Resiliency: Continue polling despite errors
            }
        }
    }
}
