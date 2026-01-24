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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
enum TransactionType {
    Trade,
    Deposit,
    Withdrawal,
}

fn default_transaction_type() -> TransactionType {
    TransactionType::Trade
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
    #[serde(default = "default_transaction_type")]
    transaction_type: TransactionType,
    #[serde(alias = "asset")]  // Backward compat
    base_asset: String,
    #[serde(default = "default_quote_usd")]
    quote_asset: String,
    side: TradeSide,
    quantity: f64,
    price: f64,
    timestamp: String,
    #[serde(default)]
    base_usd_price: Option<f64>,
    #[serde(default)]
    quote_usd_price: Option<f64>,
}

fn default_quote_usd() -> String {
    "USD".to_string()
}

impl Trade {
    fn asset(&self) -> &str {
        &self.base_asset
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
enum TradeSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize)]
struct TradeRequest {
    asset: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_asset: Option<String>,
    side: String,
    quantity: f64,
}

#[derive(Clone, Debug, Serialize)]
struct DepositRequest {
    amount: f64,
}

#[derive(Clone, Debug, Serialize)]
struct WithdrawalRequest {
    amount: f64,
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

    // Multi-asset price tracking
    let mut btc_price = use_signal(|| 0.0);
    let mut eth_price = use_signal(|| 0.0);
    let mut btc_history = use_signal(|| Vec::<PricePoint>::new());
    let mut eth_history = use_signal(|| Vec::<PricePoint>::new());

    let mut portfolio = use_signal(|| None::<UserData>);
    let mut quantity = use_signal(|| String::from("0.01"));
    let mut status = use_signal(|| String::from(""));
    let mut deposit_amount = use_signal(|| String::from("100"));
    let mut withdrawal_amount = use_signal(|| String::from("100"));

    // Auth form state
    let mut auth_username = use_signal(|| String::new());
    let mut auth_password = use_signal(|| String::new());
    let mut auth_error = use_signal(|| String::new());

    // Fetch BTC price on mount and every 5 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price?asset=BTC", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceResponse>().await {
                        btc_price.set(data.price);
                    }
                }
                gloo_timers::future::TimeoutFuture::new(5_000).await;
            }
        });
    });

    // Fetch ETH price on mount and every 5 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price?asset=ETH", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceResponse>().await {
                        eth_price.set(data.price);
                    }
                }
                gloo_timers::future::TimeoutFuture::new(5_000).await;
            }
        });
    });

    // Fetch BTC price history on mount and every 30 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price/history?asset=BTC", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceHistoryResponse>().await {
                        btc_history.set(data.prices);
                    }
                }
                gloo_timers::future::TimeoutFuture::new(30_000).await;
            }
        });
    });

    // Fetch ETH price history on mount and every 30 seconds
    use_effect(move || {
        spawn(async move {
            loop {
                if let Ok(resp) = reqwest::get(format!("{}/price/history?asset=ETH", API_BASE)).await {
                    if let Ok(data) = resp.json::<PriceHistoryResponse>().await {
                        eth_history.set(data.prices);
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

    let execute_trade = move |side: &str, asset: &str, quote_asset_opt: Option<String>| {
        let side = side.to_string();
        let asset = asset.to_string();
        let qty = quantity().parse::<f64>().unwrap_or(0.0);
        let uid = user_id();

        spawn(async move {
            let trade = TradeRequest {
                asset: asset.clone(),
                quote_asset: quote_asset_opt,
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

    let execute_deposit = move || {
        let amount = deposit_amount().parse::<f64>().unwrap_or(0.0);
        let uid = user_id();

        spawn(async move {
            let request = DepositRequest { amount };
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/deposit?user_id={}", API_BASE, uid.clone()))
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        status.set(format!("Deposit of ${:.2} successful!", amount));
                        // Refetch portfolio
                        if let Ok(resp) = reqwest::get(format!("{}/portfolio?user_id={}", API_BASE, uid)).await {
                            if let Ok(data) = resp.json::<UserData>().await {
                                portfolio.set(Some(data));
                            }
                        }
                    } else {
                        if let Ok(error_resp) = response.json::<TradeErrorResponse>().await {
                            status.set(error_resp.error);
                        } else {
                            status.set("Deposit failed".to_string());
                        }
                    }
                }
                Err(e) => status.set(format!("Error: {}", e)),
            }
        });
    };

    let execute_withdrawal = move || {
        let amount = withdrawal_amount().parse::<f64>().unwrap_or(0.0);
        let uid = user_id();

        spawn(async move {
            let request = WithdrawalRequest { amount };
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/withdrawal?user_id={}", API_BASE, uid.clone()))
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        status.set(format!("Withdrawal of ${:.2} successful!", amount));
                        // Refetch portfolio
                        if let Ok(resp) = reqwest::get(format!("{}/portfolio?user_id={}", API_BASE, uid)).await {
                            if let Ok(data) = resp.json::<UserData>().await {
                                portfolio.set(Some(data));
                            }
                        }
                    } else {
                        if let Ok(error_resp) = response.json::<TradeErrorResponse>().await {
                            status.set(error_resp.error);
                        } else {
                            status.set("Withdrawal failed".to_string());
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
                        h1 { style: "margin-bottom: 40px;", "Trading Simulator" }

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
                            h1 { style: "margin: 0;", "Trading Simulator - Dashboard" }
                            p { style: "color: #666; margin: 5px 0 0 0;", "Logged in as: {username}" }
                        }
                        div { style: "display: flex; gap: 10px;",
                            button {
                                disabled: true,
                                style: "padding: 10px 20px; background: #ccc; color: #666; border: none; border-radius: 4px; cursor: default; font-size: 14px; font-weight: bold;",
                                "Dashboard"
                            }
                            button {
                                onclick: move |_| current_view.set(AppView::Markets),
                                style: "padding: 10px 20px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                "Markets"
                            }
                            button {
                                onclick: move |_| handle_logout(),
                                style: "padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                "Logout"
                            }
                        }
                    }

                    if let Some(p) = portfolio() {
                        div { class: "portfolio",
                            style: "background: #e8f5e9; padding: 20px; border-radius: 8px; margin: 20px 0;",
                            h2 { "Portfolio" }
                            {
                                let usd_balance = p.asset_balances.get("USD").copied().unwrap_or(0.0);
                                rsx! {
                                    p { style: "font-size: 18px; font-weight: bold; margin-bottom: 10px;", "Cash: ${usd_balance:.2}" }
                                }
                            }

                            if !p.asset_balances.is_empty() {
                                h3 { style: "margin-top: 20px; margin-bottom: 10px;", "Assets" }
                                for (asset, balance) in p.asset_balances.iter() {
                                    if asset != "USD" && *balance > 0.0 {
                                        p { style: "font-size: 16px; margin: 5px 0;", "{asset}: {balance:.8}" }
                                    }
                                }
                            }
                        }

                        // Deposit/Withdrawal Controls
                        div { class: "funding",
                            style: "background: #fff3e0; padding: 20px; border-radius: 8px; margin: 20px 0;",
                            h2 { "Account Funding" }

                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin-top: 15px;",
                                // Deposit section
                                div { style: "background: white; padding: 15px; border-radius: 4px; border: 1px solid #ddd;",
                                    h3 { style: "margin-top: 0; color: #4caf50;", "Deposit" }
                                    p { style: "font-size: 12px; color: #666; margin: 5px 0;", "Min: $10 | Max: $100,000" }
                                    input {
                                        r#type: "number",
                                        value: "{deposit_amount}",
                                        oninput: move |e| deposit_amount.set(e.value().clone()),
                                        style: "width: 100%; padding: 10px; margin: 10px 0; font-size: 16px; border: 1px solid #ddd; border-radius: 4px;",
                                        placeholder: "Amount"
                                    }
                                    button {
                                        onclick: move |_| execute_deposit(),
                                        style: "width: 100%; padding: 12px; background: #4caf50; color: white; border: none; border-radius: 4px; font-size: 16px; font-weight: bold; cursor: pointer;",
                                        "Deposit Funds"
                                    }
                                }

                                // Withdrawal section
                                div { style: "background: white; padding: 15px; border-radius: 4px; border: 1px solid #ddd;",
                                    h3 { style: "margin-top: 0; color: #f44336;", "Withdraw" }
                                    p { style: "font-size: 12px; color: #666; margin: 5px 0;", "Available: ${p.cash_balance:.2}" }
                                    input {
                                        r#type: "number",
                                        value: "{withdrawal_amount}",
                                        oninput: move |e| withdrawal_amount.set(e.value().clone()),
                                        style: "width: 100%; padding: 10px; margin: 10px 0; font-size: 16px; border: 1px solid #ddd; border-radius: 4px;",
                                        placeholder: "Amount"
                                    }
                                    button {
                                        onclick: move |_| execute_withdrawal(),
                                        style: "width: 100%; padding: 12px; background: #f44336; color: white; border: none; border-radius: 4px; font-size: 16px; font-weight: bold; cursor: pointer;",
                                        "Withdraw Funds"
                                    }
                                }
                            }

                            // Lifetime stats
                            {
                                let lifetime_deposits: f64 = p.trade_history.iter()
                                    .filter(|t| t.transaction_type == TransactionType::Deposit)
                                    .map(|t| t.quantity)
                                    .sum();
                                let lifetime_withdrawals: f64 = p.trade_history.iter()
                                    .filter(|t| t.transaction_type == TransactionType::Withdrawal)
                                    .map(|t| t.quantity)
                                    .sum();
                                let lifetime_funding = 10000.0 + lifetime_deposits;

                                // Calculate total trade volume in USD (estimated for cross-pairs)
                                let total_trade_volume_usd: f64 = p.trade_history.iter()
                                    .filter(|t| t.transaction_type == TransactionType::Trade)
                                    .filter_map(|t| {
                                        // Calculate USD value: quantity * price * quote_usd_price
                                        t.quote_usd_price.map(|q_usd| t.quantity * t.price * q_usd)
                                    })
                                    .sum();

                                rsx! {
                                    div { style: "margin-top: 20px; padding: 15px; background: white; border-radius: 4px; border: 1px solid #ddd;",
                                        h3 { style: "margin-top: 0;", "Lifetime Statistics" }
                                        div { style: "display: grid; grid-template-columns: 1fr 1fr 1fr 1fr; gap: 15px; text-align: center;",
                                            div {
                                                p { style: "margin: 0; font-size: 12px; color: #666;", "Total Funding" }
                                                p { style: "margin: 5px 0 0 0; font-size: 20px; font-weight: bold; color: #4caf50;", "${lifetime_funding:.2}" }
                                            }
                                            div {
                                                p { style: "margin: 0; font-size: 12px; color: #666;", "Total Deposits" }
                                                p { style: "margin: 5px 0 0 0; font-size: 20px; font-weight: bold;", "${lifetime_deposits:.2}" }
                                            }
                                            div {
                                                p { style: "margin: 0; font-size: 12px; color: #666;", "Total Withdrawals" }
                                                p { style: "margin: 5px 0 0 0; font-size: 20px; font-weight: bold; color: #f44336;", "${lifetime_withdrawals:.2}" }
                                            }
                                            div {
                                                p { style: "margin: 0; font-size: 12px; color: #666;", "Trade Volume (USD)" }
                                                p { style: "margin: 5px 0 0 0; font-size: 20px; font-weight: bold; color: #2196F3;", "${total_trade_volume_usd:.2}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Transaction History
                        div { class: "trade-history",
                            style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                            h2 { "Transaction History" }
                            if p.trade_history.is_empty() {
                                p { style: "color: #666;", "No transactions yet" }
                            } else {
                                div { style: "overflow-x: auto;",
                                    table { style: "width: 100%; border-collapse: collapse;",
                                        thead {
                                            tr { style: "border-bottom: 2px solid #ddd;",
                                                th { style: "padding: 10px; text-align: left;", "Type" }
                                                th { style: "padding: 10px; text-align: left;", "Asset" }
                                                th { style: "padding: 10px; text-align: left;", "Action" }
                                                th { style: "padding: 10px; text-align: right;", "Quantity" }
                                                th { style: "padding: 10px; text-align: right;", "Price" }
                                                th { style: "padding: 10px; text-align: right;", "Total" }
                                                th { style: "padding: 10px; text-align: left;", "Time" }
                                            }
                                        }
                                        tbody {
                                            for trade in p.trade_history.iter().rev().take(10) {
                                                tr { style: "border-bottom: 1px solid #eee;",
                                                    // Transaction Type
                                                    td {
                                                        style: "padding: 10px;",
                                                        {
                                                            match trade.transaction_type {
                                                                TransactionType::Deposit => "💰 Deposit",
                                                                TransactionType::Withdrawal => "💸 Withdraw",
                                                                TransactionType::Trade => "📈 Trade",
                                                            }
                                                        }
                                                    }
                                                    // Asset
                                                    td {
                                                        style: "padding: 10px;",
                                                        {
                                                            match trade.transaction_type {
                                                                TransactionType::Trade => format!("{}/{}", trade.base_asset, trade.quote_asset),
                                                                _ => trade.asset().to_string(),
                                                            }
                                                        }
                                                    }
                                                    // Action
                                                    td {
                                                        style: if matches!(trade.side, TradeSide::Buy) { "padding: 10px; color: #4caf50; font-weight: bold;" } else { "padding: 10px; color: #f44336; font-weight: bold;" },
                                                        {
                                                            match trade.transaction_type {
                                                                TransactionType::Deposit => "+".to_string(),
                                                                TransactionType::Withdrawal => "-".to_string(),
                                                                TransactionType::Trade => format!("{:?}", trade.side),
                                                            }
                                                        }
                                                    }
                                                    td { style: "padding: 10px; text-align: right;", "{trade.quantity:.8}" }
                                                    // Price column - show in quote asset terms
                                                    td {
                                                        style: "padding: 10px; text-align: right;",
                                                        {
                                                            if trade.quote_asset == "USD" {
                                                                format!("${:.2}", trade.price)
                                                            } else {
                                                                format!("{:.4} {}", trade.price, trade.quote_asset)
                                                            }
                                                        }
                                                    }
                                                    // Total column - show in quote asset terms
                                                    td {
                                                        style: "padding: 10px; text-align: right;",
                                                        {
                                                            let total = trade.price * trade.quantity;
                                                            if trade.quote_asset == "USD" {
                                                                format!("${:.2}", total)
                                                            } else {
                                                                format!("{:.4} {}", total, trade.quote_asset)
                                                            }
                                                        }
                                                    }
                                                    td { style: "padding: 10px;", "{format_timestamp(&trade.timestamp)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if p.trade_history.len() > 10 {
                                    p { style: "margin-top: 10px; color: #666; font-size: 14px;",
                                        "Showing last 10 of {p.trade_history.len()} transactions"
                                    }
                                }
                            }
                        }
                    }
                },
                AppView::Markets => rsx! {
                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                        div {
                            h1 { style: "margin: 0;", "Trading Simulator - Markets" }
                            p { style: "color: #666; margin: 5px 0 0 0;", "Logged in as: {username}" }
                        }
                        div { style: "display: flex; gap: 10px;",
                            button {
                                onclick: move |_| current_view.set(AppView::Dashboard),
                                style: "padding: 10px 20px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                "Dashboard"
                            }
                            button {
                                disabled: true,
                                style: "padding: 10px 20px; background: #ccc; color: #666; border: none; border-radius: 4px; cursor: default; font-size: 14px; font-weight: bold;",
                                "Markets"
                            }
                            button {
                                onclick: move |_| handle_logout(),
                                style: "padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                "Logout"
                            }
                        }
                    }

                    h2 { "Available Markets" }
                    p { style: "color: #666; margin-bottom: 30px;", "Click on a market to start trading" }

                    div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 20px;",
                        // BTC/USD Market
                        div {
                            onclick: move |_| current_view.set(AppView::Trading("BTC".to_string())),
                            style: "background: white; padding: 20px; border-radius: 8px; border: 2px solid #ddd; cursor: pointer; transition: all 0.2s; hover: border-color: #2196F3;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                                h3 { style: "margin: 0; font-size: 24px;", "BTC/USD" }
                                p { style: "margin: 0; font-size: 28px; font-weight: bold; color: #2196F3;", "${btc_price():.2}" }
                            }
                            p { style: "color: #666; font-size: 14px; margin-bottom: 15px;", "Bitcoin" }
                            if !btc_history().is_empty() {
                                div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                    svg {
                                        width: "100%",
                                        height: "100",
                                        view_box: "0 0 300 100",
                                        {
                                            let prices = btc_history();
                                            let min = prices.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
                                            let max = prices.iter().map(|p| p.price).fold(f64::NEG_INFINITY, f64::max);
                                            let range = if (max - min).abs() < 0.01 { 1.0 } else { max - min };

                                            let mut path = String::from("M ");
                                            for (i, point) in prices.iter().enumerate() {
                                                let x = (i as f64 / (prices.len() - 1) as f64) * 300.0;
                                                let y = 100.0 - ((point.price - min) / range) * 100.0;
                                                if i == 0 {
                                                    path.push_str(&format!("{} {} ", x, y));
                                                } else {
                                                    path.push_str(&format!("L {} {} ", x, y));
                                                }
                                            }

                                            rsx! {
                                                path {
                                                    d: "{path}",
                                                    fill: "none",
                                                    stroke: "#2196F3",
                                                    stroke_width: "2"
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999;",
                                    "Loading chart..."
                                }
                            }
                        }

                        // ETH/USD Market
                        div {
                            onclick: move |_| current_view.set(AppView::Trading("ETH".to_string())),
                            style: "background: white; padding: 20px; border-radius: 8px; border: 2px solid #ddd; cursor: pointer; transition: all 0.2s;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                                h3 { style: "margin: 0; font-size: 24px;", "ETH/USD" }
                                p { style: "margin: 0; font-size: 28px; font-weight: bold; color: #9c27b0;", "${eth_price():.2}" }
                            }
                            p { style: "color: #666; font-size: 14px; margin-bottom: 15px;", "Ethereum" }
                            if !eth_history().is_empty() {
                                div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                    svg {
                                        width: "100%",
                                        height: "100",
                                        view_box: "0 0 300 100",
                                        {
                                            let prices = eth_history();
                                            let min = prices.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
                                            let max = prices.iter().map(|p| p.price).fold(f64::NEG_INFINITY, f64::max);
                                            let range = if (max - min).abs() < 0.01 { 1.0 } else { max - min };

                                            let mut path = String::from("M ");
                                            for (i, point) in prices.iter().enumerate() {
                                                let x = (i as f64 / (prices.len() - 1) as f64) * 300.0;
                                                let y = 100.0 - ((point.price - min) / range) * 100.0;
                                                if i == 0 {
                                                    path.push_str(&format!("{} {} ", x, y));
                                                } else {
                                                    path.push_str(&format!("L {} {} ", x, y));
                                                }
                                            }

                                            rsx! {
                                                path {
                                                    d: "{path}",
                                                    fill: "none",
                                                    stroke: "#9c27b0",
                                                    stroke_width: "2"
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999;",
                                    "Loading chart..."
                                }
                            }
                        }

                        // BTC/ETH Market (cross-pair)
                        div {
                            onclick: move |_| current_view.set(AppView::Trading("BTC/ETH".to_string())),
                            style: "background: white; padding: 20px; border-radius: 8px; border: 2px solid #ddd; cursor: pointer; transition: all 0.2s;",
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                                h3 { style: "margin: 0; font-size: 24px;", "BTC/ETH" }
                                {
                                    let btc = btc_price();
                                    let eth = eth_price();
                                    let cross_price = if btc > 0.0 && eth > 0.0 {
                                        btc / eth
                                    } else {
                                        0.0
                                    };

                                    rsx! {
                                        p { style: "margin: 0; font-size: 28px; font-weight: bold; color: #ff9800;",
                                            if cross_price > 0.0 {
                                                "{cross_price:.4} ETH"
                                            } else {
                                                "--"
                                            }
                                        }
                                    }
                                }
                            }
                            p { style: "color: #666; font-size: 14px; margin-bottom: 15px;", "Bitcoin per Ethereum" }
                            {
                                // Calculate BTC/ETH historical data
                                let btc_hist = btc_history();
                                let eth_hist = eth_history();
                                let mut cross_history = Vec::new();

                                for btc_point in btc_hist.iter() {
                                    if let Some(eth_point) = eth_hist.iter().find(|e| e.timestamp == btc_point.timestamp) {
                                        if eth_point.price > 0.0 {
                                            cross_history.push(PricePoint {
                                                timestamp: btc_point.timestamp,
                                                price: btc_point.price / eth_point.price,
                                            });
                                        }
                                    }
                                }

                                if !cross_history.is_empty() {
                                    rsx! {
                                        div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center;",
                                            svg {
                                                width: "100%",
                                                height: "100",
                                                view_box: "0 0 300 100",
                                                {
                                                    let prices = &cross_history;
                                                    let min = prices.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
                                                    let max = prices.iter().map(|p| p.price).fold(f64::NEG_INFINITY, f64::max);
                                                    let range = if (max - min).abs() < 0.01 { 1.0 } else { max - min };

                                                    let mut path = String::from("M ");
                                                    for (i, point) in prices.iter().enumerate() {
                                                        let x = (i as f64 / (prices.len() - 1) as f64) * 300.0;
                                                        let y = 100.0 - ((point.price - min) / range) * 100.0;
                                                        if i == 0 {
                                                            path.push_str(&format!("{} {} ", x, y));
                                                        } else {
                                                            path.push_str(&format!("L {} {} ", x, y));
                                                        }
                                                    }

                                                    rsx! {
                                                        path {
                                                            d: "{path}",
                                                            fill: "none",
                                                            stroke: "#ff9800",
                                                            stroke_width: "2"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { style: "height: 120px; background: #f5f5f5; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #999;",
                                            "Loading chart..."
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                AppView::Trading(asset) => rsx! {
                    {
                        let (base_asset, quote_asset, current_price, current_history) = if asset == "BTC/ETH" {
                            // Cross-pair: BTC priced in ETH
                            let btc = btc_price();
                            let eth = eth_price();
                            let cross_price = if btc > 0.0 && eth > 0.0 { btc / eth } else { 0.0 };

                            // Calculate BTC/ETH historical data from BTC-USD and ETH-USD
                            let btc_hist = btc_history();
                            let eth_hist = eth_history();
                            let mut cross_history = Vec::new();

                            // Match timestamps and calculate cross-pair price
                            for btc_point in btc_hist.iter() {
                                if let Some(eth_point) = eth_hist.iter().find(|e| e.timestamp == btc_point.timestamp) {
                                    if eth_point.price > 0.0 {
                                        cross_history.push(PricePoint {
                                            timestamp: btc_point.timestamp,
                                            price: btc_point.price / eth_point.price,
                                        });
                                    }
                                }
                            }

                            ("BTC", "ETH", cross_price, cross_history)
                        } else if asset == "BTC" {
                            ("BTC", "USD", btc_price(), btc_history())
                        } else {
                            ("ETH", "USD", eth_price(), eth_history())
                        };

                        rsx! {
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                                div {
                                    h1 { style: "margin: 0;", "🚀 Trading Simulator - {base_asset}/{quote_asset}" }
                                    p { style: "color: #666; margin: 5px 0 0 0;", "Logged in as: {username}" }
                                }
                                div { style: "display: flex; gap: 10px;",
                                    button {
                                        onclick: move |_| current_view.set(AppView::Dashboard),
                                        style: "padding: 10px 20px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                        "Dashboard"
                                    }
                                    button {
                                        disabled: true,
                                        style: "padding: 10px 20px; background: #ccc; color: #666; border: none; border-radius: 4px; cursor: default; font-size: 14px; font-weight: bold;",
                                        "Markets"
                                    }
                                    button {
                                        onclick: move |_| handle_logout(),
                                        style: "padding: 10px 20px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                        "Logout"
                                    }
                                }
                            }

                            div { class: "price-display",
                                style: "background: #f0f0f0; padding: 20px; border-radius: 8px; margin: 20px 0;",
                                h2 { "{base_asset}/{quote_asset} Price" }
                                p { style: "font-size: 32px; font-weight: bold;",
                                    if quote_asset == "USD" {
                                        "${current_price:.2}"
                                    } else {
                                        "{current_price:.4} {quote_asset}"
                                    }
                                }
                            }

                            // Price Chart (shows base asset USD price history)
                            div { class: "price-chart",
                                style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                                h2 { "{base_asset} Price History (Last Hour)" }
                                if !current_history.is_empty() {
                                    PriceChart { prices: current_history }
                                } else {
                                    p { style: "color: #666;", "Loading price data..." }
                                }
                            }

                            if let Some(p) = portfolio() {
                                div { class: "portfolio",
                                    style: "background: #e8f5e9; padding: 20px; border-radius: 8px; margin: 20px 0;",
                                    h2 { "Portfolio" }
                                    p { style: "font-size: 16px;", "{base_asset}: {p.asset_balances.get(base_asset).unwrap_or(&0.0):.8}" }
                                    p { style: "font-size: 16px;", "{quote_asset}: {p.asset_balances.get(quote_asset).unwrap_or(&0.0):.8}" }
                                }
                            }

                            div { class: "trade-form",
                                style: "background: #fff3e0; padding: 20px; border-radius: 8px;",
                                h2 { "Trade {base_asset}/{quote_asset}" }

                                label { "Quantity ({base_asset}):" }
                                input {
                                    r#type: "number",
                                    step: "0.001",
                                    value: "{quantity}",
                                    oninput: move |e| quantity.set(e.value()),
                                    style: "margin: 10px 0; padding: 8px; width: 100%;",
                                }

                                div { style: "display: flex; gap: 10px; margin-top: 10px;",
                                    button {
                                        onclick: {
                                            let base = base_asset.to_string();
                                            let quote_opt = if quote_asset != "USD" {
                                                Some(quote_asset.to_string())
                                            } else {
                                                None
                                            };
                                            move |_| execute_trade("Buy", &base, quote_opt.clone())
                                        },
                                        style: "flex: 1; padding: 12px; background: #4caf50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                        "Buy {base_asset}"
                                    }
                                    button {
                                        onclick: {
                                            let base = base_asset.to_string();
                                            let quote_opt = if quote_asset != "USD" {
                                                Some(quote_asset.to_string())
                                            } else {
                                                None
                                            };
                                            move |_| execute_trade("Sell", &base, quote_opt.clone())
                                        },
                                        style: "flex: 1; padding: 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                        "Sell {base_asset}"
                                    }
                                }

                                if !status().is_empty() {
                                    p { style: "margin-top: 10px; color: #666;", "{status}" }
                                }
                            }

                            // Trade History filtered by base_asset
                            if let Some(p) = portfolio() {
                                div { class: "trade-history",
                                    style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                                    h2 { "{base_asset} Trade History" }
                                    {
                                        let filtered_trades: Vec<_> = p.trade_history.iter()
                                            .filter(|t| t.asset() == base_asset)
                                            .collect();

                                        if filtered_trades.is_empty() {
                                            rsx! {
                                                p { style: "color: #666;", "No {base_asset} trades yet" }
                                            }
                                        } else {
                                            rsx! {
                                                div { style: "overflow-x: auto;",
                                                    table { style: "width: 100%; border-collapse: collapse;",
                                                        thead {
                                                            tr { style: "border-bottom: 2px solid #ddd;",
                                                                th { style: "padding: 10px; text-align: left;", "Side" }
                                                                th { style: "padding: 10px; text-align: right;", "Quantity" }
                                                                th { style: "padding: 10px; text-align: right;", "Price" }
                                                                th { style: "padding: 10px; text-align: right;", "Total" }
                                                                th { style: "padding: 10px; text-align: left;", "Time" }
                                                            }
                                                        }
                                                        tbody {
                                                            for trade in filtered_trades.iter().rev().take(10) {
                                                                tr { style: "border-bottom: 1px solid #eee;",
                                                                    td {
                                                                        style: if matches!(trade.side, TradeSide::Buy) {
                                                                            "padding: 10px; color: #4caf50; font-weight: bold;"
                                                                        } else {
                                                                            "padding: 10px; color: #f44336; font-weight: bold;"
                                                                        },
                                                                        "{trade.side:?}"
                                                                    }
                                                                    td { style: "padding: 10px; text-align: right;", "{trade.quantity:.8}" }
                                                                    // Price column - show in quote asset terms
                                                                    td {
                                                                        style: "padding: 10px; text-align: right;",
                                                                        {
                                                                            if trade.quote_asset == "USD" {
                                                                                format!("${:.2}", trade.price)
                                                                            } else {
                                                                                format!("{:.4} {}", trade.price, trade.quote_asset)
                                                                            }
                                                                        }
                                                                    }
                                                                    // Total column - show in quote asset terms
                                                                    td {
                                                                        style: "padding: 10px; text-align: right;",
                                                                        {
                                                                            let total = trade.price * trade.quantity;
                                                                            if trade.quote_asset == "USD" {
                                                                                format!("${:.2}", total)
                                                                            } else {
                                                                                format!("{:.4} {}", total, trade.quote_asset)
                                                                            }
                                                                        }
                                                                    }
                                                                    td { style: "padding: 10px;", "{format_timestamp(&trade.timestamp)}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                if filtered_trades.len() > 10 {
                                                    p { style: "margin-top: 10px; color: #666; font-size: 14px;",
                                                        "Showing last 10 of {filtered_trades.len()} {base_asset} trades"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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
