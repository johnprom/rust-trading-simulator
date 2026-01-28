use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{self, Timelike};

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
    timestamp: i64, // Unix timestamp in seconds
    price: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Candle {
    timestamp: i64, // Unix timestamp in seconds
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Clone, PartialEq, Props)]
struct PriceChartProps {
    prices: Vec<PricePoint>,
    quote_asset: String,
    timeframe: String, // "1h", "8h", or "24h"
    #[props(optional)]
    indicator_data: Option<IndicatorResponse>,
}

#[derive(Clone, PartialEq, Props)]
struct CandlestickChartProps {
    candles: Vec<Candle>,
    quote_asset: String,
    timeframe: String,
    #[props(optional)]
    indicator_data: Option<IndicatorResponse>,
}

#[derive(Clone, Debug, Deserialize)]
struct PriceHistoryResponse {
    asset: String,
    prices: Vec<PricePoint>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct IndicatorResponse {
    asset: String,
    timeframe: String,
    timestamps: Vec<i64>,
    prices: Vec<f64>,
    indicators: HashMap<String, Vec<Option<f64>>>,
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
    #[serde(default)]
    executed_by_bot: Option<String>,
}

fn default_quote_usd() -> String {
    "USD".to_string()
}

impl Trade {
    fn asset(&self) -> &str {
        &self.base_asset
    }
}

#[derive(Clone, Debug, Serialize)]
struct StartBotRequest {
    user_id: String,
    bot_name: String,
    base_asset: String,
    quote_asset: String,
    stoploss_amount: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct BotResponse {
    success: bool,
    message: String,
}

#[derive(Clone, Debug, Deserialize)]
struct BotStatusResponse {
    is_active: bool,
    bot_name: Option<String>,
    trading_pair: Option<String>,
    stoploss_amount: Option<f64>,
    initial_portfolio_value: Option<f64>,
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
fn PriceChart(props: PriceChartProps) -> Element {
    // Clone props data to satisfy lifetime requirements for event handlers
    let prices = props.prices.clone();
    let quote_asset = props.quote_asset.clone();

    // Debug: Log if we have indicator data
    if let Some(ref ind_data) = props.indicator_data {
        web_sys::console::log_1(&format!("PriceChart received indicators: {:?}", ind_data.indicators.keys().collect::<Vec<_>>()).into());
    } else {
        web_sys::console::log_1(&"PriceChart: No indicator data".into());
    }

    if prices.is_empty() {
        return rsx! { p { "No data available" } };
    }

    // Hover state for crosshair and tooltip
    let mut hover_x = use_signal(|| None::<f64>);
    let mut hover_y = use_signal(|| None::<f64>);
    let mut hover_price = use_signal(|| None::<f64>);
    let mut hover_time = use_signal(|| None::<i64>);

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

    // Generate vertical grid lines and time labels (6 marks with real timestamps)
    let mut v_grid_lines = Vec::new();
    let time_span = prices.last().unwrap().timestamp - prices.first().unwrap().timestamp;
    for i in 0..6 {
        let x = padding_left + (i as f64 / 5.0) * (width - padding_left - padding_right);
        let timestamp = prices.first().unwrap().timestamp + ((time_span as f64 * i as f64 / 5.0) as i64);
        v_grid_lines.push((x, timestamp));
    }

    // Precompute fixed coordinates
    let chart_top = padding_top;
    let chart_bottom = height - padding_bottom;
    let chart_left = padding_left;
    let chart_right = width - padding_right;

    // Price label (show currency symbol for USD, otherwise show asset name)
    let price_label = if quote_asset == "USD" {
        "Price ($)".to_string()
    } else {
        format!("Price ({})", quote_asset)
    };

    rsx! {
        div {
            style: "position: relative;",
            svg {
                width: "{width}",
                height: "{height}",
                view_box: "0 0 {width} {height}",
                style: "display: block; margin: 0 auto; background: white; cursor: crosshair;",
                onmousemove: move |evt| {
                    let rect_x = evt.data().element_coordinates().x;
                    let rect_y = evt.data().element_coordinates().y;

                    // Check if within chart bounds
                    if rect_x >= chart_left && rect_x <= chart_right && rect_y >= chart_top && rect_y <= chart_bottom {
                        hover_x.set(Some(rect_x));
                        hover_y.set(Some(rect_y));

                        // Calculate price from y position
                        let price = max_price - ((rect_y - chart_top) / (chart_bottom - chart_top)) * price_range;
                        hover_price.set(Some(price));

                        // Calculate time from x position
                        let time_idx = ((rect_x - chart_left) / (chart_right - chart_left) * (prices.len() - 1) as f64) as usize;
                        if time_idx < prices.len() {
                            hover_time.set(Some(prices[time_idx].timestamp));
                        }
                    } else {
                        hover_x.set(None);
                        hover_y.set(None);
                        hover_price.set(None);
                        hover_time.set(None);
                    }
                },
                onmouseleave: move |_| {
                    hover_x.set(None);
                    hover_y.set(None);
                    hover_price.set(None);
                    hover_time.set(None);
                },

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
                        {
                            if quote_asset == "USD" {
                                format!("${:.2}", price)
                            } else {
                                format!("{:.4}", price)
                            }
                        }
                    }
                }

                // Vertical grid lines with time labels
                for (x, timestamp) in v_grid_lines.iter() {
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
                        {
                            // Format timestamp as HH:MM
                            let dt = chrono::DateTime::from_timestamp(*timestamp, 0).unwrap();
                            format!("{:02}:{:02}", dt.hour(), dt.minute())
                        }
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

                // Indicator overlays (SMA/EMA)
                if let Some(ref indicators) = props.indicator_data {
                    // SMA(20) overlay - Orange
                    if let Some(sma_20) = indicators.indicators.get("sma_20") {
                        {
                            let mut sma_path = String::from("M ");
                            let mut first_valid = true;
                            for (i, value_opt) in sma_20.iter().enumerate() {
                                if let Some(value) = value_opt {
                                    let x = padding_left + (i as f64 / (sma_20.len() - 1) as f64) * (width - padding_left - padding_right);
                                    let y = height - padding_bottom - ((value - min_price) / price_range) * (height - padding_top - padding_bottom);
                                    if first_valid {
                                        sma_path.push_str(&format!("{} {} ", x, y));
                                        first_valid = false;
                                    } else {
                                        sma_path.push_str(&format!("L {} {} ", x, y));
                                    }
                                }
                            }
                            rsx! {
                                path {
                                    d: "{sma_path}",
                                    fill: "none",
                                    stroke: "#FF9800",
                                    stroke_width: "2",
                                    opacity: "0.8"
                                }
                            }
                        }
                    }

                    // SMA(50) overlay - Purple
                    if let Some(sma_50) = indicators.indicators.get("sma_50") {
                        {
                            let mut sma_path = String::from("M ");
                            let mut first_valid = true;
                            for (i, value_opt) in sma_50.iter().enumerate() {
                                if let Some(value) = value_opt {
                                    let x = padding_left + (i as f64 / (sma_50.len() - 1) as f64) * (width - padding_left - padding_right);
                                    let y = height - padding_bottom - ((value - min_price) / price_range) * (height - padding_top - padding_bottom);
                                    if first_valid {
                                        sma_path.push_str(&format!("{} {} ", x, y));
                                        first_valid = false;
                                    } else {
                                        sma_path.push_str(&format!("L {} {} ", x, y));
                                    }
                                }
                            }
                            rsx! {
                                path {
                                    d: "{sma_path}",
                                    fill: "none",
                                    stroke: "#9C27B0",
                                    stroke_width: "2",
                                    opacity: "0.8"
                                }
                            }
                        }
                    }

                    // EMA(12) overlay - Teal
                    if let Some(ema_12) = indicators.indicators.get("ema_12") {
                        {
                            let mut ema_path = String::from("M ");
                            let mut first_valid = true;
                            for (i, value_opt) in ema_12.iter().enumerate() {
                                if let Some(value) = value_opt {
                                    let x = padding_left + (i as f64 / (ema_12.len() - 1) as f64) * (width - padding_left - padding_right);
                                    let y = height - padding_bottom - ((value - min_price) / price_range) * (height - padding_top - padding_bottom);
                                    if first_valid {
                                        ema_path.push_str(&format!("{} {} ", x, y));
                                        first_valid = false;
                                    } else {
                                        ema_path.push_str(&format!("L {} {} ", x, y));
                                    }
                                }
                            }
                            rsx! {
                                path {
                                    d: "{ema_path}",
                                    fill: "none",
                                    stroke: "#009688",
                                    stroke_width: "2",
                                    opacity: "0.8"
                                }
                            }
                        }
                    }

                    // EMA(26) overlay - Deep Orange
                    if let Some(ema_26) = indicators.indicators.get("ema_26") {
                        {
                            let mut ema_path = String::from("M ");
                            let mut first_valid = true;
                            for (i, value_opt) in ema_26.iter().enumerate() {
                                if let Some(value) = value_opt {
                                    let x = padding_left + (i as f64 / (ema_26.len() - 1) as f64) * (width - padding_left - padding_right);
                                    let y = height - padding_bottom - ((value - min_price) / price_range) * (height - padding_top - padding_bottom);
                                    if first_valid {
                                        ema_path.push_str(&format!("{} {} ", x, y));
                                        first_valid = false;
                                    } else {
                                        ema_path.push_str(&format!("L {} {} ", x, y));
                                    }
                                }
                            }
                            rsx! {
                                path {
                                    d: "{ema_path}",
                                    fill: "none",
                                    stroke: "#FF5722",
                                    stroke_width: "2",
                                    opacity: "0.8"
                                }
                            }
                        }
                    }
                }

                // Crosshair lines
                if let Some(x) = hover_x() {
                    line {
                        x1: "{x}",
                        y1: "{chart_top}",
                        x2: "{x}",
                        y2: "{chart_bottom}",
                        stroke: "#666",
                        stroke_width: "1",
                        stroke_dasharray: "4,4",
                        pointer_events: "none"
                    }
                }
                if let Some(y) = hover_y() {
                    line {
                        x1: "{chart_left}",
                        y1: "{y}",
                        x2: "{chart_right}",
                        y2: "{y}",
                        stroke: "#666",
                        stroke_width: "1",
                        stroke_dasharray: "4,4",
                        pointer_events: "none"
                    }
                }

                // Axis labels
                text {
                    x: "{chart_left - 60.0}",
                    y: "{(chart_top + chart_bottom) / 2.0}",
                    font_size: "14",
                    fill: "#333",
                    text_anchor: "middle",
                    transform: "rotate(-90 {chart_left - 60.0} {(chart_top + chart_bottom) / 2.0})",
                    "{price_label}"
                }
                text {
                    x: "{(chart_left + chart_right) / 2.0}",
                    y: "{height - 10.0}",
                    font_size: "14",
                    fill: "#333",
                    text_anchor: "middle",
                    "Time"
                }
            }

            // Tooltip
            if let Some(price) = hover_price() {
                if let Some(time) = hover_time() {
                    if let (Some(x), Some(y)) = (hover_x(), hover_y()) {
                        div {
                            style: "position: absolute; left: {x + 10.0}px; top: {y - 40.0}px; background: rgba(0,0,0,0.8); color: white; padding: 8px 12px; border-radius: 4px; font-size: 12px; pointer-events: none; white-space: nowrap;",
                            div {
                                {
                                    let dt = chrono::DateTime::from_timestamp(time, 0).unwrap();
                                    format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second())
                                }
                            }
                            div {
                                {
                                    if quote_asset == "USD" {
                                        format!("${:.2}", price)
                                    } else {
                                        format!("{:.4} {}", price, quote_asset)
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

fn CandlestickChart(props: CandlestickChartProps) -> Element {
    let candles = props.candles.clone();
    let quote_asset = props.quote_asset.clone();

    if candles.is_empty() {
        return rsx! { p { "No candlestick data available" } };
    }

    // Hover state
    let mut hover_candle_idx = use_signal(|| None::<usize>);

    let width = 1000.0;
    let height = 300.0;
    let padding_left = 80.0;
    let padding_right = 40.0;
    let padding_top = 40.0;
    let padding_bottom = 60.0;

    // Find min/max prices
    let mut min_price = f64::INFINITY;
    let mut max_price = f64::NEG_INFINITY;
    for candle in &candles {
        min_price = min_price.min(candle.low);
        max_price = max_price.max(candle.high);
    }
    let price_range = if (max_price - min_price).abs() < 0.01 { 1.0 } else { max_price - min_price };

    let chart_width = width - padding_left - padding_right;
    let candle_spacing = chart_width / candles.len() as f64;
    let candle_width = (candle_spacing * 0.7).max(2.0);

    let price_label = if quote_asset == "USD" {
        "Price ($)".to_string()
    } else {
        format!("Price ({})", quote_asset)
    };

    // Build SVG elements as strings
    let mut svg_elements = String::new();

    // Grid lines and labels
    for i in 0..5 {
        let y = padding_top + (i as f64 / 4.0) * (height - padding_top - padding_bottom);
        let price = max_price - (i as f64 / 4.0) * price_range;
        svg_elements.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#e0e0e0\" stroke-width=\"1\"/>",
            padding_left, y, width - padding_right, y
        ));
        svg_elements.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"end\" font-size=\"12\" fill=\"#666\">{:.2}</text>",
            padding_left - 10.0, y + 5.0, price
        ));
    }

    let time_span = candles.last().unwrap().timestamp - candles.first().unwrap().timestamp;
    for i in 0..6 {
        let x = padding_left + (i as f64 / 5.0) * chart_width;
        let timestamp = candles.first().unwrap().timestamp + ((time_span as f64 * i as f64 / 5.0) as i64);
        let dt = chrono::DateTime::from_timestamp(timestamp, 0).unwrap();
        svg_elements.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#e0e0e0\" stroke-width=\"1\"/>",
            x, padding_top, x, height - padding_bottom
        ));
        svg_elements.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#666\">{:02}:{:02}</text>",
            x, height - padding_bottom + 20.0, dt.hour(), dt.minute()
        ));
    }

    // Draw candlesticks
    for (i, candle) in candles.iter().enumerate() {
        let x_center = padding_left + (i as f64 + 0.5) * candle_spacing;
        let open_y = height - padding_bottom - ((candle.open - min_price) / price_range) * (height - padding_top - padding_bottom);
        let close_y = height - padding_bottom - ((candle.close - min_price) / price_range) * (height - padding_top - padding_bottom);
        let high_y = height - padding_bottom - ((candle.high - min_price) / price_range) * (height - padding_top - padding_bottom);
        let low_y = height - padding_bottom - ((candle.low - min_price) / price_range) * (height - padding_top - padding_bottom);

        let is_bullish = candle.close >= candle.open;
        let color = if is_bullish { "#26a69a" } else { "#ef5350" };
        let body_height = (open_y - close_y).abs().max(1.0);
        let body_y = open_y.min(close_y);

        // High-low wick
        svg_elements.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>",
            x_center, high_y, x_center, low_y, color
        ));
        // Body
        svg_elements.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\"/>",
            x_center - candle_width / 2.0, body_y, candle_width, body_height, color, color
        ));
    }

    // Axis labels
    svg_elements.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"bold\" fill=\"#333\" transform=\"rotate(-90 {} {})\">{}</text>",
        padding_left / 2.0, height / 2.0, padding_left / 2.0, height / 2.0, price_label
    ));
    svg_elements.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-size=\"14\" font-weight=\"bold\" fill=\"#333\">Time</text>",
        width / 2.0, height - 10.0
    ));

    // Clone values needed for closures
    let padding_left_clone = padding_left;
    let chart_width_clone = chart_width;
    let candles_len = candles.len();

    // Build tooltip HTML if hovering
    let tooltip_html = if let Some(idx) = hover_candle_idx() {
        if idx < candles.len() {
            let candle = &candles[idx];
            let dt = chrono::DateTime::from_timestamp(candle.timestamp, 0).unwrap();
            let x_pos = padding_left_clone + (idx as f64 + 0.5) * candle_spacing;
            Some(format!(
                "<div style=\"position: absolute; left: {}px; top: {}px; background: rgba(0,0,0,0.85); color: white; padding: 8px; border-radius: 4px; pointer-events: none; font-size: 12px; white-space: nowrap; z-index: 1000; transform: translateX(-50%);\"><div>{}</div><div>Open: ${:.2}</div><div>High: ${:.2}</div><div>Low: ${:.2}</div><div>Close: ${:.2}</div></div>",
                x_pos, 10.0,
                dt.format("%Y-%m-%d %H:%M"),
                candle.open, candle.high, candle.low, candle.close
            ))
        } else {
            None
        }
    } else {
        None
    };

    rsx! {
        div {
            style: "position: relative;",
            div {
                dangerous_inner_html: format!(
                    "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" style=\"display: block; margin: 0 auto; background: white;\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#fafafa\"/>{}</svg>",
                    width, height, width, height,
                    padding_left, padding_top,
                    width - padding_left - padding_right,
                    height - padding_top - padding_bottom,
                    svg_elements
                )
            }

            // Transparent overlay for mouse events
            div {
                style: format!(
                    "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; cursor: crosshair;",
                    padding_left, padding_top, chart_width, height - padding_top - padding_bottom
                ),
                onmousemove: move |evt| {
                    let rect_x = evt.data.element_coordinates().x;
                    let candle_idx = (rect_x / (chart_width_clone / candles_len as f64)).floor() as usize;
                    if candle_idx < candles_len {
                        hover_candle_idx.set(Some(candle_idx));
                    }
                },
                onmouseleave: move |_| {
                    hover_candle_idx.set(None);
                }
            }

            // Tooltip
            if let Some(html) = tooltip_html {
                div {
                    dangerous_inner_html: html
                }
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

    // Bot state
    let mut bot_status = use_signal(|| None::<BotStatusResponse>);
    let mut bot_stoploss = use_signal(|| String::from("1000"));
    let mut selected_bot = use_signal(|| String::from("naive_momentum"));

    // Chart state
    let mut selected_timeframe = use_signal(|| String::from("1h"));
    let mut chart_type = use_signal(|| String::from("line")); // "line" or "candlestick"
    let mut candle_history = use_signal(|| Vec::<Candle>::new());

    // Indicator state
    let mut indicator_data = use_signal(|| None::<IndicatorResponse>);
    let mut show_sma_20 = use_signal(|| false);
    let mut show_sma_50 = use_signal(|| false);
    let mut show_ema_12 = use_signal(|| false);
    let mut show_ema_26 = use_signal(|| false);

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

    // Fetch BTC price history when timeframe changes
    let fetch_btc_history = move || {
        let timeframe = selected_timeframe();
        web_sys::console::log_1(&format!("Fetching BTC history with timeframe: {}", timeframe).into());
        spawn(async move {
            let url = format!("{}/price/history?asset=BTC&timeframe={}", API_BASE, timeframe);
            web_sys::console::log_1(&format!("BTC URL: {}", url).into());
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(data) = resp.json::<PriceHistoryResponse>().await {
                    web_sys::console::log_1(&format!("BTC history received: {} points", data.prices.len()).into());
                    btc_history.set(data.prices);
                }
            }
        });
    };

    // Re-fetch BTC history when timeframe changes
    use_effect(move || {
        selected_timeframe();  // Track dependency
        fetch_btc_history();
    });

    // Periodic BTC history refresh (every 30 seconds)
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(30_000).await;
                fetch_btc_history();
            }
        });
    });

    // Fetch ETH price history when timeframe changes
    let fetch_eth_history = move || {
        let timeframe = selected_timeframe();
        web_sys::console::log_1(&format!("Fetching ETH history with timeframe: {}", timeframe).into());
        spawn(async move {
            let url = format!("{}/price/history?asset=ETH&timeframe={}", API_BASE, timeframe);
            web_sys::console::log_1(&format!("ETH URL: {}", url).into());
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(data) = resp.json::<PriceHistoryResponse>().await {
                    web_sys::console::log_1(&format!("ETH history received: {} points", data.prices.len()).into());
                    eth_history.set(data.prices);
                }
            }
        });
    };

    // Re-fetch ETH history when timeframe changes
    use_effect(move || {
        selected_timeframe();  // Track dependency
        fetch_eth_history();
    });

    // Periodic ETH history refresh (every 30 seconds)
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(30_000).await;
                fetch_eth_history();
            }
        });
    });

    // Fetch candlestick data for the selected market (base/quote)
    let fetch_candle_history = move |asset: &str| {
        let timeframe = selected_timeframe();
        let asset = asset.to_string();
        spawn(async move {
            let url = format!("{}/price/candles?asset={}&timeframe={}", API_BASE, asset, timeframe);
            if let Ok(resp) = reqwest::get(&url).await {
                #[derive(Deserialize)]
                struct CandleHistoryResponse {
                    candles: Vec<Candle>,
                }
                if let Ok(data) = resp.json::<CandleHistoryResponse>().await {
                    candle_history.set(data.candles);
                }
            }
        });
    };

    // Fetch indicator data for the selected market
    let mut fetch_indicators = move |asset: &str| {
        let timeframe = selected_timeframe();

        // Only fetch if timeframe is 1h (indicators only supported for 1h)
        if timeframe != "1h" {
            indicator_data.set(None);
            return;
        }

        // Build indicators list based on toggles
        let mut indicators = Vec::new();
        if show_sma_20() {
            indicators.push("sma_20");
        }
        if show_sma_50() {
            indicators.push("sma_50");
        }
        if show_ema_12() {
            indicators.push("ema_12");
        }
        if show_ema_26() {
            indicators.push("ema_26");
        }

        // If no indicators selected, clear data
        if indicators.is_empty() {
            indicator_data.set(None);
            return;
        }

        let indicators_param = indicators.join(",");
        let asset = asset.to_string();

        spawn(async move {
            let url = format!(
                "{}/indicators?asset={}&timeframe={}&indicators={}",
                API_BASE, asset, timeframe, indicators_param
            );
            web_sys::console::log_1(&format!("Fetching indicators: {}", url).into());
            match reqwest::get(&url).await {
                Ok(resp) => {
                    web_sys::console::log_1(&format!("Response status: {}", resp.status()).into());
                    match resp.json::<IndicatorResponse>().await {
                        Ok(data) => {
                            web_sys::console::log_1(&format!("Received indicator data with {} indicators", data.indicators.len()).into());
                            indicator_data.set(Some(data));
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("Failed to parse indicator response: {:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("Failed to fetch indicators: {:?}", e).into());
                }
            }
        });
    };

    // Re-fetch candle data when timeframe changes (only when in candlestick mode)
    use_effect(move || {
        let timeframe = selected_timeframe();
        let current_chart_type = chart_type();

        // Fetch immediately if we're in candlestick mode and in Trading view
        if current_chart_type == "candlestick" {
            if let AppView::Trading(asset) = &*current_view.peek() {
                // Extract base asset from trading pair
                let base_asset = if asset.contains('/') {
                    asset.split('/').next().unwrap_or("BTC")
                } else {
                    asset.as_str()
                };
                fetch_candle_history(base_asset);
            }
        }
    });

    // Periodic candle history refresh based on timeframe
    // 1h view: refresh every 60 seconds (new 1-minute candle)
    // 8h/24h views: refresh every 5 minutes (new 5-minute candle)
    use_effect(move || {
        spawn(async move {
            loop {
                let timeframe = selected_timeframe();
                let current_chart_type = chart_type();

                // Only poll if in candlestick mode
                if current_chart_type == "candlestick" {
                    // Determine refresh interval based on timeframe
                    let interval_ms = match timeframe.as_str() {
                        "1h" => 60_000,  // 1 minute for 1h view
                        "8h" | "24h" => 300_000,  // 5 minutes for 8h/24h views
                        _ => 60_000,  // Default to 1 minute
                    };

                    gloo_timers::future::TimeoutFuture::new(interval_ms).await;

                    // Fetch candles for the current trading pair
                    if let AppView::Trading(asset) = &*current_view.peek() {
                        // Extract base asset from trading pair
                        let base_asset = if asset.contains('/') {
                            asset.split('/').next().unwrap_or("BTC")
                        } else {
                            asset.as_str()
                        };
                        fetch_candle_history(base_asset);
                    }
                } else {
                    // If not in candlestick mode, just wait a bit before checking again
                    gloo_timers::future::TimeoutFuture::new(5_000).await;
                }
            }
        });
    });

    // Fetch indicators when toggles or timeframe changes
    use_effect(move || {
        let (_tf, _sma20, _sma50, _ema12, _ema26) = (
            selected_timeframe(),
            show_sma_20(),
            show_sma_50(),
            show_ema_12(),
            show_ema_26()
        );

        if let AppView::Trading(asset) = &*current_view.peek() {
            let base_asset = if asset.contains('/') {
                asset.split('/').next().unwrap_or("BTC")
            } else {
                asset.as_str()
            };
            fetch_indicators(base_asset);
        }
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

    // Poll portfolio every 10 seconds when bot is active
    use_effect(move || {
        spawn(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(10_000).await;

                // Only poll if in trading view and bot is active
                if matches!(current_view(), AppView::Trading(_)) {
                    if let Some(status) = bot_status() {
                        if status.is_active {
                            fetch_portfolio();
                        }
                    }
                }
            }
        });
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

    // Fetch bot status when in Trading view
    let fetch_bot_status = move || {
        let uid = user_id();
        spawn(async move {
            if let Ok(resp) = reqwest::get(format!("{}/bot/status?user_id={}", API_BASE, uid)).await {
                if let Ok(data) = resp.json::<BotStatusResponse>().await {
                    bot_status.set(Some(data));
                }
            }
        });
    };

    use_effect(move || {
        // Poll bot status every 5 seconds when in Trading view
        match current_view() {
            AppView::Trading(_) => {
                fetch_bot_status();
                spawn(async move {
                    loop {
                        gloo_timers::future::TimeoutFuture::new(5_000).await;
                        if matches!(current_view(), AppView::Trading(_)) {
                            fetch_bot_status();
                        } else {
                            break;
                        }
                    }
                });
            }
            _ => {}
        }
    });

    let start_bot = move |base_asset: String, quote_asset: String| {
        let stoploss = bot_stoploss().parse::<f64>().unwrap_or(1000.0);
        let bot_name = selected_bot();
        let uid = user_id();

        spawn(async move {
            let request = StartBotRequest {
                user_id: uid.clone(),
                bot_name,
                base_asset,
                quote_asset,
                stoploss_amount: stoploss,
            };

            let client = reqwest::Client::new();
            match client
                .post(format!("{}/bot/start", API_BASE))
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(bot_resp) = response.json::<BotResponse>().await {
                            status.set(bot_resp.message);
                            // Immediately fetch updated bot status
                            if let Ok(resp) = reqwest::get(format!("{}/bot/status?user_id={}", API_BASE, uid)).await {
                                if let Ok(data) = resp.json::<BotStatusResponse>().await {
                                    bot_status.set(Some(data));
                                }
                            }
                        }
                    } else {
                        if let Ok(error) = response.text().await {
                            status.set(format!("Bot start failed: {}", error));
                        }
                    }
                }
                Err(e) => status.set(format!("Error: {}", e)),
            }
        });
    };

    let stop_bot = move || {
        let uid = user_id();

        spawn(async move {
            let client = reqwest::Client::new();
            match client
                .post(format!("{}/bot/stop?user_id={}", API_BASE, uid.clone()))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(bot_resp) = response.json::<BotResponse>().await {
                            status.set(bot_resp.message);
                            // Immediately fetch updated bot status
                            if let Ok(resp) = reqwest::get(format!("{}/bot/status?user_id={}", API_BASE, uid)).await {
                                if let Ok(data) = resp.json::<BotStatusResponse>().await {
                                    bot_status.set(Some(data));
                                }
                            }
                        }
                    } else {
                        if let Ok(error) = response.text().await {
                            status.set(format!("Bot stop failed: {}", error));
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
                                // Calculate total portfolio value in USD
                                let mut total_value_usd = 0.0;
                                for (asset, balance) in p.asset_balances.iter() {
                                    if asset == "USD" {
                                        total_value_usd += balance;
                                    } else if asset == "BTC" {
                                        total_value_usd += balance * btc_price();
                                    } else if asset == "ETH" {
                                        total_value_usd += balance * eth_price();
                                    }
                                }

                                rsx! {
                                    p { style: "font-size: 18px; font-weight: bold; margin-bottom: 15px;",
                                        "Estimated Total Value: ${total_value_usd:.2}"
                                    }
                                }
                            }

                            h3 { style: "margin-top: 20px; margin-bottom: 10px;", "Assets" }
                            for (asset, balance) in p.asset_balances.iter() {
                                if *balance > 0.0 || asset == "USD" {
                                    if asset == "USD" {
                                        p { style: "font-size: 16px; margin: 5px 0;", "USD: ${balance:.2}" }
                                    } else {
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
                                                th { style: "padding: 10px; text-align: center;", "Source" }
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
                                                    // Source column - show bot icon if executed by bot
                                                    td {
                                                        style: "padding: 10px; text-align: center;",
                                                        {
                                                            if let Some(bot_name) = &trade.executed_by_bot {
                                                                format!("🤖 {}", bot_name)
                                                            } else {
                                                                "Manual".to_string()
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
                                        onclick: move |_| current_view.set(AppView::Markets),
                                        style: "padding: 10px 20px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                        "← Markets"
                                    }
                                    button {
                                        onclick: move |_| current_view.set(AppView::Dashboard),
                                        style: "padding: 10px 20px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;",
                                        "Dashboard"
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

                            // Price Chart (shows base asset price history)
                            div { class: "price-chart",
                                style: "background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border: 1px solid #ddd;",
                                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                                    h2 { style: "margin: 0;", "{base_asset} Price History" }
                                    div { style: "display: flex; gap: 15px; align-items: center;",
                                        // Chart type toggle
                                        div { style: "display: flex; gap: 4px; border: 1px solid #ddd; border-radius: 4px; overflow: hidden;",
                                            button {
                                                onclick: move |_| chart_type.set("line".to_string()),
                                                style: if chart_type() == "line" {
                                                    "padding: 6px 12px; background: #2196F3; color: white; border: none; cursor: pointer; font-size: 12px;"
                                                } else {
                                                    "padding: 6px 12px; background: white; color: #333; border: none; cursor: pointer; font-size: 12px;"
                                                },
                                                "Line"
                                            }
                                            button {
                                                onclick: move |_| {
                                                    chart_type.set("candlestick".to_string());
                                                    // Trigger candle fetch
                                                    fetch_candle_history(&base_asset);
                                                },
                                                style: if chart_type() == "candlestick" {
                                                    "padding: 6px 12px; background: #2196F3; color: white; border: none; cursor: pointer; font-size: 12px;"
                                                } else {
                                                    "padding: 6px 12px; background: white; color: #333; border: none; cursor: pointer; font-size: 12px;"
                                                },
                                                "Candles"
                                            }
                                        }
                                        // Timeframe selection
                                        div { style: "display: flex; gap: 8px;",
                                            button {
                                                onclick: move |_| {
                                                    selected_timeframe.set("1h".to_string());
                                                    if chart_type() == "candlestick" {
                                                        fetch_candle_history(&base_asset);
                                                    }
                                                },
                                                style: if selected_timeframe() == "1h" {
                                                    "padding: 8px 16px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: bold;"
                                                } else {
                                                    "padding: 8px 16px; background: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                                },
                                                "1H"
                                            }
                                            button {
                                                onclick: move |_| {
                                                    selected_timeframe.set("8h".to_string());
                                                    if chart_type() == "candlestick" {
                                                        fetch_candle_history(&base_asset);
                                                    }
                                                },
                                                style: if selected_timeframe() == "8h" {
                                                    "padding: 8px 16px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: bold;"
                                                } else {
                                                    "padding: 8px 16px; background: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                                },
                                                "8H"
                                            }
                                            button {
                                                onclick: move |_| {
                                                    selected_timeframe.set("24h".to_string());
                                                    if chart_type() == "candlestick" {
                                                        fetch_candle_history(&base_asset);
                                                    }
                                                },
                                                style: if selected_timeframe() == "24h" {
                                                    "padding: 8px 16px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 13px; font-weight: bold;"
                                                } else {
                                                    "padding: 8px 16px; background: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 4px; cursor: pointer; font-size: 13px;"
                                                },
                                                "24H"
                                            }
                                        }
                                    }
                                }

                                // Render chart based on chart type
                                if chart_type() == "candlestick" {
                                    if !candle_history().is_empty() {
                                        CandlestickChart {
                                            candles: candle_history(),
                                            quote_asset: quote_asset.to_string(),
                                            timeframe: selected_timeframe(),
                                            indicator_data: indicator_data()
                                        }
                                    } else {
                                        p { style: "color: #666;", "Loading candlestick data..." }
                                    }
                                } else {
                                    if !current_history.is_empty() {
                                        PriceChart {
                                            prices: current_history,
                                            quote_asset: quote_asset.to_string(),
                                            timeframe: selected_timeframe(),
                                            indicator_data: indicator_data()
                                        }
                                    } else {
                                        p { style: "color: #666;", "Loading price data..." }
                                    }
                                }

                                // Indicator toggles (only for 1h view) - Below chart
                                if selected_timeframe() == "1h" {
                                    div { style: "display: flex; gap: 10px; align-items: center; margin-top: 15px; padding: 10px; background: #f9f9f9; border-radius: 4px;",
                                        span { style: "font-size: 13px; color: #666; font-weight: bold;", "Indicators:" }
                                        label { style: "display: flex; align-items: center; gap: 5px; cursor: pointer; font-size: 13px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: show_sma_20(),
                                                onchange: move |_| show_sma_20.set(!show_sma_20())
                                            }
                                            "SMA(20)"
                                        }
                                        label { style: "display: flex; align-items: center; gap: 5px; cursor: pointer; font-size: 13px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: show_sma_50(),
                                                onchange: move |_| show_sma_50.set(!show_sma_50())
                                            }
                                            "SMA(50)"
                                        }
                                        label { style: "display: flex; align-items: center; gap: 5px; cursor: pointer; font-size: 13px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: show_ema_12(),
                                                onchange: move |_| show_ema_12.set(!show_ema_12())
                                            }
                                            "EMA(12)"
                                        }
                                        label { style: "display: flex; align-items: center; gap: 5px; cursor: pointer; font-size: 13px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: show_ema_26(),
                                                onchange: move |_| show_ema_26.set(!show_ema_26())
                                            }
                                            "EMA(26)"
                                        }
                                    }
                                }
                            }

                            if let Some(p) = portfolio() {
                                div { class: "portfolio",
                                    style: "background: #e8f5e9; padding: 20px; border-radius: 8px; margin: 20px 0;",
                                    h2 { "Portfolio" }
                                    {
                                        // Calculate total portfolio value in USD
                                        let mut total_value_usd = 0.0;
                                        for (asset, balance) in p.asset_balances.iter() {
                                            if asset == "USD" {
                                                total_value_usd += balance;
                                            } else if asset == "BTC" {
                                                total_value_usd += balance * btc_price();
                                            } else if asset == "ETH" {
                                                total_value_usd += balance * eth_price();
                                            }
                                        }

                                        let base_balance = p.asset_balances.get(base_asset).copied().unwrap_or(0.0);
                                        let quote_balance = p.asset_balances.get(quote_asset).copied().unwrap_or(0.0);

                                        rsx! {
                                            p { style: "font-size: 18px; font-weight: bold; margin-bottom: 15px;",
                                                "Estimated Total Value: ${total_value_usd:.2}"
                                            }
                                            {
                                                if quote_asset == "USD" {
                                                    rsx! {
                                                        p { style: "font-size: 16px; margin: 5px 0;", "USD: ${quote_balance:.2}" }
                                                        p { style: "font-size: 16px; margin: 5px 0;", "{base_asset}: {base_balance:.8}" }
                                                    }
                                                } else {
                                                    rsx! {
                                                        p { style: "font-size: 16px; margin: 5px 0;", "{base_asset}: {base_balance:.8}" }
                                                        p { style: "font-size: 16px; margin: 5px 0;", "{quote_asset}: {quote_balance:.8}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
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

                            // Bot Controls
                            div { class: "bot-controls",
                                style: "background: #e3f2fd; padding: 20px; border-radius: 8px; margin: 20px 0; border: 2px solid #2196F3;",
                                h2 { style: "margin-bottom: 15px;", "Trading Bot" }

                                // Bot Status Display
                                if let Some(status) = bot_status() {
                                    if status.is_active {
                                        div { style: "background: #c8e6c9; padding: 15px; border-radius: 6px; margin-bottom: 15px; border-left: 4px solid #4caf50;",
                                            p { style: "margin: 0; font-weight: bold; color: #2e7d32;", "🤖 Bot Active" }
                                            if let Some(bot_name) = &status.bot_name {
                                                p { style: "margin: 5px 0 0 0; font-size: 14px;", "Bot: {bot_name}" }
                                            }
                                            if let Some(pair) = &status.trading_pair {
                                                p { style: "margin: 5px 0 0 0; font-size: 14px;", "Pair: {pair}" }
                                            }
                                            if let Some(stoploss) = status.stoploss_amount {
                                                p { style: "margin: 5px 0 0 0; font-size: 14px;", "Stoploss: ${stoploss:.2}" }
                                            }
                                            if let Some(initial_value) = status.initial_portfolio_value {
                                                p { style: "margin: 5px 0 0 0; font-size: 14px;", "Started at: ${initial_value:.2}" }
                                            }
                                        }

                                        button {
                                            onclick: move |_| stop_bot(),
                                            style: "width: 100%; padding: 12px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; font-weight: bold;",
                                            "Stop Bot"
                                        }
                                    } else {
                                        div { style: "background: #fff3cd; padding: 15px; border-radius: 6px; margin-bottom: 15px; border-left: 4px solid #ff9800;",
                                            p { style: "margin: 0; font-weight: bold; color: #e65100;", "⏸️ No Bot Running" }
                                            p { style: "margin: 5px 0 0 0; font-size: 13px; color: #666;", "Configure and start a bot to trade automatically" }
                                        }

                                        div { style: "margin-bottom: 15px;",
                                            label { style: "display: block; margin-bottom: 5px; font-weight: bold;", "Bot Strategy:" }
                                            select {
                                                value: "{selected_bot}",
                                                onchange: move |e| selected_bot.set(e.value()),
                                                style: "width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                                                option { value: "naive_momentum", "Naive Momentum (Buy on 3↑, Sell on 3↓)" }
                                            }
                                        }

                                        div { style: "margin-bottom: 15px;",
                                            label { style: "display: block; margin-bottom: 5px; font-weight: bold;", "Stoploss ({quote_asset}):" }
                                            input {
                                                r#type: "number",
                                                step: "100",
                                                value: "{bot_stoploss}",
                                                oninput: move |e| bot_stoploss.set(e.value()),
                                                style: "width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px;",
                                            }
                                            p { style: "margin: 5px 0 0 0; font-size: 12px; color: #666;", "Maximum loss before bot stops (step size will be 1% of this)" }
                                        }

                                        button {
                                            onclick: {
                                                let base = base_asset.to_string();
                                                let quote = quote_asset.to_string();
                                                move |_| start_bot(base.clone(), quote.clone())
                                            },
                                            style: "width: 100%; padding: 12px; background: #2196F3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; font-weight: bold;",
                                            "Start Bot"
                                        }
                                    }
                                } else {
                                    p { style: "color: #666;", "Loading bot status..." }
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
                                                                th { style: "padding: 10px; text-align: center;", "Source" }
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
                                                                    // Source column - show bot icon if executed by bot
                                                                    td {
                                                                        style: "padding: 10px; text-align: center;",
                                                                        {
                                                                            if let Some(bot_name) = &trade.executed_by_bot {
                                                                                format!("🤖 {}", bot_name)
                                                                            } else {
                                                                                "Manual".to_string()
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
