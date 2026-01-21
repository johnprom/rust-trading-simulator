use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
struct PriceResponse {
    asset: String,
    price: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct PricePoint {
    timestamp: i64,
    price: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct PriceHistoryResponse {
    asset: String,
    prices: Vec<PricePoint>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UserData {
    username: String,
    cash_balance: f64,
    asset_balances: HashMap<String, f64>,
}

#[derive(Clone, Debug, Serialize)]
struct TradeRequest {
    asset: String,
    side: String,
    quantity: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct TradeErrorResponse {
    error: String,
}

const API_BASE: &str = "http://localhost:3000/api";

#[component]
fn PriceChart(prices: Vec<PricePoint>) -> Element {
    if prices.is_empty() {
        return rsx! { p { "No data available" } };
    }

    // Calculate chart dimensions
    let width = 1000.0;
    let height = 300.0;
    let padding_left = 80.0;
    let padding_right = 40.0;
    let padding_top = 40.0;
    let padding_bottom = 60.0;

    // Find min and max values for scaling
    let min_price = prices.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
    let max_price = prices.iter().map(|p| p.price).fold(f64::NEG_INFINITY, f64::max);
    let price_range = if (max_price - min_price).abs() < 0.01 { 1.0 } else { max_price - min_price };

    // Time range
    let min_time = prices.first().map(|p| p.timestamp).unwrap_or(0);
    let max_time = prices.last().map(|p| p.timestamp).unwrap_or(0);

    // Generate path data for the line
    let mut path_data = String::from("M ");
    for (i, point) in prices.iter().enumerate() {
        let x = padding_left + (i as f64 / (prices.len() - 1) as f64) * (width - padding_left - padding_right);
        let y = height - padding_bottom - ((point.price - min_price) / price_range) * (height - padding_top - padding_bottom);
        if i == 0 {
            path_data.push_str(&format!("{} {} ", x, y));
        } else {
            path_data.push_str(&format!("L {} {} ", x, y));
        }
    }

    // Generate horizontal grid lines (5 lines)
    let mut h_grid_lines = Vec::new();
    for i in 0..5 {
        let y = padding_top + (i as f64 / 4.0) * (height - padding_top - padding_bottom);
        let price = max_price - (i as f64 / 4.0) * price_range;
        h_grid_lines.push((y, price));
    }

    // Generate vertical grid lines and time labels (6 marks for 0, 12, 24, 36, 48, 60 minutes ago)
    let mut v_grid_lines = Vec::new();
    for i in 0..6 {
        let x = padding_left + (i as f64 / 5.0) * (width - padding_left - padding_right);
        let minutes_ago = 60 - (i * 12);
        v_grid_lines.push((x, minutes_ago));
    }

    // Precompute fixed coordinates
    let chart_top = padding_top;
    let chart_bottom = height - padding_bottom;
    let chart_left = padding_left;
    let chart_right = width - padding_right;

    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            style: "display: block; margin: 0 auto; background: white;",

            // Horizontal grid lines with price labels
            for (y, price) in h_grid_lines.iter() {
                line {
                    x1: "{chart_left}",
                    y1: "{y}",
                    x2: "{chart_right}",
                    y2: "{y}",
                    stroke: "#e0e0e0",
                    stroke_width: "1"
                }
                text {
                    x: "{chart_left - 10.0}",
                    y: "{y + 4.0}",
                    font_size: "12",
                    fill: "#666",
                    text_anchor: "end",
                    "${price:.2}"
                }
            }

            // Vertical grid lines with time labels
            for (x, minutes) in v_grid_lines.iter() {
                line {
                    x1: "{x}",
                    y1: "{chart_top}",
                    x2: "{x}",
                    y2: "{chart_bottom}",
                    stroke: "#e0e0e0",
                    stroke_width: "1"
                }
                text {
                    x: "{x}",
                    y: "{chart_bottom + 20.0}",
                    font_size: "12",
                    fill: "#666",
                    text_anchor: "middle",
                    "{minutes}m"
                }
            }

            // Chart border
            rect {
                x: "{chart_left}",
                y: "{chart_top}",
                width: "{chart_right - chart_left}",
                height: "{chart_bottom - chart_top}",
                fill: "none",
                stroke: "#999",
                stroke_width: "2"
            }

            // Price line
            path {
                d: "{path_data}",
                fill: "none",
                stroke: "#2196F3",
                stroke_width: "2",
            }

            // Axis labels
            text {
                x: "{chart_left - 60.0}",
                y: "{(chart_top + chart_bottom) / 2.0}",
                font_size: "14",
                fill: "#333",
                text_anchor: "middle",
                transform: "rotate(-90 {chart_left - 60.0} {(chart_top + chart_bottom) / 2.0})",
                "Price (USD)"
            }
            text {
                x: "{(chart_left + chart_right) / 2.0}",
                y: "{height - 10.0}",
                font_size: "14",
                fill: "#333",
                text_anchor: "middle",
                "Time (minutes ago)"
            }
        }
    }
}

fn App() -> Element {
    let mut price = use_signal(|| 0.0);
    let mut portfolio = use_signal(|| None::<UserData>);
    let mut quantity = use_signal(|| String::from("0.01"));
    let mut status = use_signal(|| String::from(""));
    let mut price_history = use_signal(|| Vec::<PricePoint>::new());

    // Fetch price on mount and every 5 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceResponse>().await {
                        price.set(data.price);
                    }
                }
                gloo_timers::future::TimeoutFuture::new(5_000).await;
            }
        });
    });

    // Fetch price history on mount and every 30 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price/history", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceHistoryResponse>().await {
                        price_history.set(data.prices);
                    }
                }
                gloo_timers::future::TimeoutFuture::new(30_000).await;
            }
        });
    });

    // Fetch portfolio
    let fetch_portfolio = move || {
        spawn(async move {
            if let Ok(resp) = reqwest::get(format!("{}/portfolio", API_BASE)).await {
                if let Ok(data) = resp.json::<UserData>().await {
                    portfolio.set(Some(data));
                }
            }
        });
    };

    use_effect(move || {
        fetch_portfolio();
    });

    let execute_trade = move |side: &str| {
        let side = side.to_string();
        let qty = quantity().parse::<f64>().unwrap_or(0.0);

        spawn(async move {
            let trade = TradeRequest {
                asset: "BTC".to_string(),
                side: side.clone(),
                quantity: qty,
            };

            let client = reqwest::Client::new();
            match client
                .post(format!("{}/trade", API_BASE))
                .json(&trade)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        status.set(format!("{} successful!", side));
                        fetch_portfolio();
                    } else {
                        // Capture status before consuming response
                        let status_code = response.status();
                        // Try to parse the error message from the response
                        if let Ok(error_resp) = response.json::<TradeErrorResponse>().await {
                            status.set(error_resp.error);
                        } else {
                            status.set(format!("Trade failed: {}", status_code));
                        }
                    }
                }
                Err(e) => status.set(format!("Error: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "container",
            style: "max-width: 1200px; margin: 0 auto; padding: 20px; font-family: sans-serif;",

            h1 { "🚀 Trading Simulator" }

            div { class: "price-display",
                style: "background: #f0f0f0; padding: 20px; border-radius: 8px; margin: 20px 0;",
                h2 { "BTC Price" }
                p { style: "font-size: 32px; font-weight: bold;",
                    "${price:0.2}"
                }
            }

            // Price Chart
            div { class: "price-chart",
                style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                h2 { "Price History (Last Hour)" }
                if !price_history().is_empty() {
                    PriceChart { prices: price_history() }
                } else {
                    p { style: "color: #666;", "Loading price data..." }
                }
            }

            if let Some(p) = portfolio() {
                div { class: "portfolio",
                    style: "background: #e8f5e9; padding: 20px; border-radius: 8px; margin: 20px 0;",
                    h2 { "Portfolio" }
                    p { "Cash: ${p.cash_balance:.2}" }
                    p { "BTC: {p.asset_balances.get(\"BTC\").unwrap_or(&0.0):.8}" }
                }
            }

            div { class: "trade-form",
                style: "background: #fff3e0; padding: 20px; border-radius: 8px;",
                h2 { "Trade" }
                
                label { "Quantity (BTC):" }
                input {
                    r#type: "number",
                    step: "0.001",
                    value: "{quantity}",
                    oninput: move |e| quantity.set(e.value()),
                    style: "margin: 10px 0; padding: 8px; width: 100%;",
                }

                div { style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        onclick: move |_| execute_trade("Buy"),
                        style: "flex: 1; padding: 12px; background: #4caf50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Buy BTC"
                    }
                    button {
                        onclick: move |_| execute_trade("Sell"),
                        style: "flex: 1; padding: 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Sell BTC"
                    }
                }

                if !status().is_empty() {
                    p { style: "margin-top: 10px; color: #666;", "{status}" }
                }
            }
        }
    }
}

fn main() {
    launch(App);
}

// use dioxus::prelude::*;
// use dioxus::launch;

// fn app() -> Element {
//     rsx! {
//         div { "Hello dioxus world!" }
//     }
// }

// fn main() {
//     launch(app);
// }
