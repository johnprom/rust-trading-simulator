use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum AppView {
    Auth,
    Dashboard,
    Markets,
    Trading(String), // Trading view for specific asset
}

#[derive(Clone, Debug, Deserialize)]
struct PriceResponse {
    asset: String,
    price: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthResponse {
    user_id: String,
    username: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Clone, Debug, Serialize)]
struct SignupRequest {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
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
    trade_history: Vec<Trade>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct Trade {
    user_id: String,
    asset: String,
    side: TradeSide,
    quantity: f64,
    price: f64,
    timestamp: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
enum TradeSide {
    Buy,
    Sell,
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

fn format_timestamp(timestamp: &str) -> String {
    // Parse ISO 8601 timestamp and format it nicely
    // Example input: "2025-01-22T10:30:00.123456789Z"
    // Example output: "Jan 22, 10:30"
    if let Some(date_part) = timestamp.split('T').next() {
        if let Some(time_part) = timestamp.split('T').nth(1) {
            let time = time_part.split(':').take(2).collect::<Vec<_>>().join(":");
            if let Some(stripped_date) = date_part.strip_prefix("2025-") {
                return format!("{} {}", stripped_date, time);
            }
        }
    }
    // Fallback to showing the raw timestamp if parsing fails
    timestamp.to_string()
}

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

    // Time range (unused, but kept for potential future use)
    let _min_time = prices.first().map(|p| p.timestamp).unwrap_or(0);
    let _max_time = prices.last().map(|p| p.timestamp).unwrap_or(0);

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
    let mut current_view = use_signal(|| AppView::Auth);
    let mut user_id = use_signal(|| String::new());
    let mut username = use_signal(|| String::new());

    let mut price = use_signal(|| 0.0);
    let mut portfolio = use_signal(|| None::<UserData>);
    let mut quantity = use_signal(|| String::from("0.01"));
    let mut status = use_signal(|| String::from(""));
    let mut price_history = use_signal(|| Vec::<PricePoint>::new());

    // Auth form state
    let mut auth_username = use_signal(|| String::new());
    let mut auth_password = use_signal(|| String::new());
    let mut auth_error = use_signal(|| String::new());

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

    // Auth handlers
    let mut handle_login = move || {
        // Validate inputs
        let uname = auth_username();
        let pwd = auth_password();

        if uname.trim().is_empty() || pwd.trim().is_empty() {
            auth_error.set("Username and password are required".to_string());
            return;
        }

        spawn(async move {
            auth_error.set(String::new());

            let client = reqwest::Client::new();
            let login_req = LoginRequest {
                username: uname,
                password: pwd,
            };

            match client
                .post(format!("{}/login", API_BASE))
                .json(&login_req)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(auth_resp) = response.json::<AuthResponse>().await {
                            user_id.set(auth_resp.user_id);
                            username.set(auth_resp.username);
                            current_view.set(AppView::Dashboard);
                        }
                    } else {
                        if let Ok(err_resp) = response.json::<ErrorResponse>().await {
                            auth_error.set(err_resp.error);
                        } else {
                            auth_error.set("Login failed".to_string());
                        }
                    }
                }
                Err(e) => auth_error.set(format!("Error: {}", e)),
            }
        });
    };

    let mut handle_signup = move || {
        // Validate inputs
        let uname = auth_username();
        let pwd = auth_password();

        if uname.trim().is_empty() || pwd.trim().is_empty() {
            auth_error.set("Username and password are required".to_string());
            return;
        }

        if pwd.len() < 6 {
            auth_error.set("Password must be at least 6 characters".to_string());
            return;
        }

        spawn(async move {
            auth_error.set(String::new());

            let client = reqwest::Client::new();
            let signup_req = SignupRequest {
                username: uname.clone(),
                password: pwd,
            };

            match client
                .post(format!("{}/signup", API_BASE))
                .json(&signup_req)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(auth_resp) = response.json::<AuthResponse>().await {
                            user_id.set(auth_resp.user_id);
                            username.set(auth_resp.username);
                            current_view.set(AppView::Dashboard);
                        }
                    } else {
                        if let Ok(err_resp) = response.json::<ErrorResponse>().await {
                            auth_error.set(err_resp.error);
                        } else {
                            auth_error.set("Signup failed".to_string());
                        }
                    }
                }
                Err(e) => auth_error.set(format!("Error: {}", e)),
            }
        });
    };

    let mut handle_guest = move || {
        user_id.set("demo_user".to_string());
        username.set("Guest".to_string());
        current_view.set(AppView::Dashboard);
    };

    let mut handle_logout = move || {
        user_id.set(String::new());
        username.set(String::new());
        auth_username.set(String::new());
        auth_password.set(String::new());
        auth_error.set(String::new());
        current_view.set(AppView::Auth);
    };

    // Fetch portfolio
    let fetch_portfolio = move || {
        let uid = user_id();
        spawn(async move {
            if let Ok(resp) = reqwest::get(format!("{}/portfolio?user_id={}", API_BASE, uid)).await {
                if let Ok(data) = resp.json::<UserData>().await {
                    portfolio.set(Some(data));
                }
            }
        });
    };

    use_effect(move || {
        // Fetch portfolio when logged in (Dashboard or Trading view)
        match current_view() {
            AppView::Dashboard | AppView::Trading(_) => {
                fetch_portfolio();
            }
            _ => {}
        }
    });

    let execute_trade = move |side: &str| {
        let side = side.to_string();
        let qty = quantity().parse::<f64>().unwrap_or(0.0);
        let uid = user_id();

        spawn(async move {
            let trade = TradeRequest {
                asset: "BTC".to_string(),
                side: side.clone(),
                quantity: qty,
            };

            let client = reqwest::Client::new();
            match client
                .post(format!("{}/trade?user_id={}", API_BASE, uid.clone()))
                .json(&trade)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        status.set(format!("{} successful!", side));
                        // Refetch portfolio after successful trade
                        if let Ok(resp) = reqwest::get(format!("{}/portfolio?user_id={}", API_BASE, uid)).await {
                            if let Ok(data) = resp.json::<UserData>().await {
                                portfolio.set(Some(data));
                            }
                        }
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

            match current_view() {
                AppView::Auth => rsx! {
                    div { style: "max-width: 500px; margin: 100px auto; text-align: center;",
                        h1 { style: "margin-bottom: 40px;", "🚀 Trading Simulator" }

                        div { style: "background: white; padding: 40px; border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);",
                            h2 { style: "margin-bottom: 30px;", "Welcome" }

                            div { style: "margin-bottom: 20px;",
                                input {
                                    r#type: "text",
                                    placeholder: "Username",
                                    value: "{auth_username}",
                                    oninput: move |e| auth_username.set(e.value()),
                                    style: "width: 100%; padding: 12px; margin-bottom: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 16px;",
                                }
                                input {
                                    r#type: "password",
                                    placeholder: "Password",
                                    value: "{auth_password}",
                                    oninput: move |e| auth_password.set(e.value()),
                                    style: "width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 16px;",
                                }
                            }

                            div { style: "display: flex; flex-direction: column; gap: 12px; margin-bottom: 20px;",
                                button {
                                    onclick: move |_| handle_login(),
                                    style: "padding: 14px; background: #2196F3; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 16px; font-weight: bold;",
                                    "Login"
                                }
                                button {
                                    onclick: move |_| handle_signup(),
                                    style: "padding: 14px; background: #4caf50; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 16px; font-weight: bold;",
                                    "Sign Up"
                                }
                            }

                            div { style: "border-top: 1px solid #ddd; padding-top: 20px; margin-top: 20px;",
                                button {
                                    onclick: move |_| handle_guest(),
                                    style: "width: 100%; padding: 14px; background: #9e9e9e; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: 16px; font-weight: bold;",
                                    "Continue as Guest"
                                }
                                p { style: "margin-top: 10px; font-size: 14px; color: #666;",
                                    "Guest profile resets on app restart"
                                }
                            }

                            if !auth_error().is_empty() {
                                p { style: "margin-top: 15px; color: #f44336; font-weight: bold;", "{auth_error}" }
                            }
                        }
                    }
                },
                AppView::Dashboard => rsx! {
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                        div {
                            h1 { style: "margin: 0;", "🚀 Trading Simulator - Dashboard" }
                            p { style: "color: #666; margin: 5px 0 0 0;", "Logged in as: {username}" }
                        }
                        button {
                            onclick: move |_| handle_logout(),
                            style: "padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                            "Logout"
                        }
                    }

                    if let Some(p) = portfolio() {
                        div { class: "portfolio",
                            style: "background: #e8f5e9; padding: 20px; border-radius: 8px; margin: 20px 0;",
                            h2 { "Portfolio" }
                            p { style: "font-size: 18px;", "Cash: ${p.cash_balance:.2}" }
                            p { style: "font-size: 18px;", "BTC: {p.asset_balances.get(\"BTC\").unwrap_or(&0.0):.8}" }
                        }

                        // Trade History
                        div { class: "trade-history",
                            style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                            h2 { "Trade History" }
                            if p.trade_history.is_empty() {
                                p { style: "color: #666;", "No trades yet" }
                            } else {
                                div { style: "overflow-x: auto;",
                                    table { style: "width: 100%; border-collapse: collapse;",
                                        thead {
                                            tr { style: "border-bottom: 2px solid #ddd;",
                                                th { style: "padding: 10px; text-align: left;", "Asset" }
                                                th { style: "padding: 10px; text-align: left;", "Side" }
                                                th { style: "padding: 10px; text-align: right;", "Quantity" }
                                                th { style: "padding: 10px; text-align: right;", "Price" }
                                                th { style: "padding: 10px; text-align: right;", "Total" }
                                                th { style: "padding: 10px; text-align: left;", "Time" }
                                            }
                                        }
                                        tbody {
                                            for trade in p.trade_history.iter().rev().take(10) {
                                                tr { style: "border-bottom: 1px solid #eee;",
                                                    td { style: "padding: 10px;", "{trade.asset}" }
                                                    td {
                                                        style: if matches!(trade.side, TradeSide::Buy) { "padding: 10px; color: #4caf50; font-weight: bold;" } else { "padding: 10px; color: #f44336; font-weight: bold;" },
                                                        "{trade.side:?}"
                                                    }
                                                    td { style: "padding: 10px; text-align: right;", "{trade.quantity:.8}" }
                                                    td { style: "padding: 10px; text-align: right;", "${trade.price:.2}" }
                                                    td { style: "padding: 10px; text-align: right;", "${trade.price * trade.quantity:.2}" }
                                                    td { style: "padding: 10px;", "{format_timestamp(&trade.timestamp)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if p.trade_history.len() > 10 {
                                    p { style: "margin-top: 10px; color: #666; font-size: 14px;",
                                        "Showing last 10 of {p.trade_history.len()} trades"
                                    }
                                }
                            }
                        }
                    }

                    div { style: "background: #fff3e0; padding: 20px; border-radius: 8px; margin: 20px 0;",
                        h2 { "Quick Trade - BTC" }

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
                },
                AppView::Markets => rsx! {
                    div { "Markets view - Coming soon" }
                },
                AppView::Trading(_asset) => rsx! {
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                        div {
                            h1 { style: "margin: 0;", "🚀 Trading Simulator" }
                            p { style: "color: #666; margin: 5px 0 0 0;", "Logged in as: {username}" }
                        }
                        button {
                            onclick: move |_| handle_logout(),
                            style: "padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                            "Logout"
                        }
                    }

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
