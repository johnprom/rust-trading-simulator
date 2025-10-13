use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
struct PriceResponse {
    asset: String,
    price: f64,
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

const API_BASE: &str = "http://localhost:3000/api";

fn App() -> Element {
    let mut price = use_signal(|| 0.0);
    let mut portfolio = use_signal(|| None::<UserData>);
    let mut quantity = use_signal(|| String::from("0.01"));
    let mut status = use_signal(|| String::from(""));

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
                Ok(_) => {
                    status.set(format!("{} successful!", side));
                    fetch_portfolio();
                }
                Err(e) => status.set(format!("Error: {}", e)),
            }
        });
    };

    rsx! {
        div { class: "container",
            style: "max-width: 800px; margin: 0 auto; padding: 20px; font-family: sans-serif;",
            
            h1 { "🚀 Trading Simulator" }
            
            div { class: "price-display",
                style: "background: #f0f0f0; padding: 20px; border-radius: 8px; margin: 20px 0;",
                h2 { "BTC Price" }
                p { style: "font-size: 32px; font-weight: bold;",
                    "${price:0.2}"
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
