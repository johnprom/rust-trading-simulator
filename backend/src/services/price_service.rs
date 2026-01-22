use crate::{api_client::ApiClient, models::PricePoint, state::AppState};
use chrono::{Duration as ChronoDuration, Utc};
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

async fn backfill_and_poll_asset(state: AppState, asset: &str) {
    let api_client = ApiClient::new();

    info!("Backfilling {} price data for last hour...", asset);

    // Fetch real historical data from Coinbase and interpolate to 5-second intervals
    let now = Utc::now();
    let one_hour_ago = now - ChronoDuration::hours(1);

    match api_client.fetch_historical_candles(asset, one_hour_ago, now, 60).await {
        Ok(candles) => {
            info!("Fetched {} minute candles for {} from Coinbase", candles.len(), asset);

            // Interpolate to 5-second intervals
            let interpolated = crate::api_client::ApiClient::interpolate_candles(asset, candles, 5);

            info!("Interpolated to {} data points for {}", interpolated.len(), asset);

            // Add all interpolated points to state
            for point in interpolated {
                state.add_price_point(point).await;
            }

            info!("Backfilled {} historical price data successfully", asset);
        }
        Err(e) => {
            error!("Failed to fetch {} historical data: {}", asset, e);
            info!("Falling back to simulated data for {}", asset);

            // Fallback: generate simulated data if API fails
            match api_client.fetch_price(asset, "USD").await {
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
                            asset: asset.to_string(),
                            price,
                        };

                        state.add_price_point(point).await;
                    }

                    info!("Backfilled {} with simulated data", asset);
                }
                Err(e2) => {
                    error!("Failed to fetch current {} price for simulation: {}", asset, e2);
                }
            }
        }
    }

    let mut interval = time::interval(Duration::from_secs(5));
    info!("Starting live {} price polling (5s interval)", asset);

    loop {
        interval.tick().await;

        match api_client.fetch_price(asset, "USD").await {
            Ok(price_point) => {
                info!("Fetched {} price: ${:.2}", asset, price_point.price);
                state.add_price_point(price_point).await;
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
