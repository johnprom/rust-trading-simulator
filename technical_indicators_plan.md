# Technical Indicators Implementation Plan

## Overview
This document outlines the implementation strategy for adding technical indicators (Moving Averages, RSI, MACD) to the trading simulator, covering both UX integration and bot framework consumption.

---

## 1. Architecture Analysis

### Current Data Flow
```
Backend (price_service.rs)
  → Fetches prices every 5s
  → Stores in AppState:
    - price_window: 5-second points (last 24h)
    - candle_window: 5-minute candles (last 24h)
    - ohlc_candles_1m: 1-minute OHLC (last 1h)
    - ohlc_candles_5m: 5-minute OHLC (last 24h)

API Endpoints
  → /api/price/history?asset={}&timeframe={} (line chart)
  → /api/price/candles?asset={}&timeframe={} (candlestick chart)

Frontend
  → PriceChart component (line charts)
  → CandlestickChart component (candlestick charts)
  → Both support 1h, 8h, 24h timeframes

Bot Framework (bots/mod.rs)
  → BotContext provides: price_window (5s data), balances, current_price
  → Bots implement tick() method, called every 60s
  → Current bot: NaiveMomentumBot (simple 3-tick trend detection)
```

### Key Design Constraints
1. **Real-time Updates**: Indicators must update as new price data arrives
2. **Multiple Timeframes**: Must support 1h, 8h, 24h views
3. **Performance**: Calculations should be efficient (run on every chart render)
4. **Dual Purpose**: Same indicators for both UI visualization and bot consumption

---

## 2. Technical Indicators to Implement

### 2.1 Simple Moving Average (SMA)
- **Formula**: Average of last N prices
- **Typical Periods**:
  - Short-term: 7, 10, 20 periods
  - Medium-term: 50 periods
  - Long-term: 100, 200 periods
- **Use Cases**: Trend identification, support/resistance levels

### 2.2 Exponential Moving Average (EMA)
- **Formula**: Weighted average giving more weight to recent prices
- **EMA = Price(t) × k + EMA(t-1) × (1-k)**, where k = 2/(N+1)
- **Typical Periods**: 12, 26 (for MACD)
- **Use Cases**: More responsive than SMA, better for fast-moving markets

### 2.3 Relative Strength Index (RSI)
- **Formula**: RSI = 100 - (100 / (1 + RS)), where RS = Avg Gain / Avg Loss
- **Period**: Typically 14
- **Range**: 0-100
- **Signals**:
  - >70: Overbought (potential sell signal)
  - <30: Oversold (potential buy signal)
- **Use Cases**: Identify overbought/oversold conditions

### 2.4 MACD (Moving Average Convergence Divergence)
- **Components**:
  - MACD Line: EMA(12) - EMA(26)
  - Signal Line: EMA(9) of MACD Line
  - Histogram: MACD Line - Signal Line
- **Signals**:
  - MACD crosses above Signal: Bullish (buy)
  - MACD crosses below Signal: Bearish (sell)
- **Use Cases**: Momentum changes, trend reversals

---

## 3. Implementation Strategy

### Phase 1: Backend Indicator Library

**Location**: `backend/src/indicators/mod.rs` (new module)

**Structure**:
```rust
// backend/src/indicators/mod.rs
pub mod moving_averages;
pub mod rsi;
pub mod macd;

// Common trait for all indicators
pub trait Indicator {
    fn calculate(&self, prices: &[f64]) -> Vec<f64>;
    fn name(&self) -> &str;
}

// Helper functions
pub fn gains_and_losses(prices: &[f64]) -> (Vec<f64>, Vec<f64>);
```

**Modules**:

1. **moving_averages.rs**
```rust
pub struct SMA {
    period: usize,
}

impl SMA {
    pub fn new(period: usize) -> Self { ... }
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> { ... }
}

pub struct EMA {
    period: usize,
}

impl EMA {
    pub fn new(period: usize) -> Self { ... }
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> { ... }
    fn smoothing_factor(&self) -> f64 { 2.0 / (self.period as f64 + 1.0) }
}
```

2. **rsi.rs**
```rust
pub struct RSI {
    period: usize, // typically 14
}

impl RSI {
    pub fn new(period: usize) -> Self { ... }
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> { ... }
    // Returns values 0-100, with NaN for insufficient data
}
```

3. **macd.rs**
```rust
pub struct MACD {
    fast_period: usize,   // typically 12
    slow_period: usize,   // typically 26
    signal_period: usize, // typically 9
}

pub struct MACDResult {
    pub macd_line: Vec<f64>,
    pub signal_line: Vec<f64>,
    pub histogram: Vec<f64>,
}

impl MACD {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self { ... }
    pub fn calculate(&self, prices: &[f64]) -> MACDResult { ... }
}
```

**Design Decisions**:
- Pure calculation functions, no state
- Accept `&[f64]` slice for flexibility
- Return `Vec<f64>` aligned with input (pad with NaN for warmup period)
- Efficient algorithms (O(n) where possible)

---

### Phase 2: API Endpoint for Indicators

**Location**: `backend/src/routes/indicators.rs` (new file)

**Endpoints**:

```rust
// GET /api/indicators?asset={BTC}&timeframe={1h}&indicators={sma_20,rsi_14,macd}
#[derive(Deserialize)]
pub struct IndicatorQuery {
    pub asset: String,
    pub timeframe: String,      // "1h", "8h", "24h"
    pub indicators: String,      // comma-separated: "sma_20,ema_50,rsi_14,macd"
}

#[derive(Serialize)]
pub struct IndicatorResponse {
    pub asset: String,
    pub timeframe: String,
    pub timestamps: Vec<i64>,   // Aligned with price data
    pub indicators: HashMap<String, Vec<f64>>, // "sma_20" -> [values...]
}

pub async fn get_indicators(
    State(state): State<AppState>,
    Query(query): Query<IndicatorQuery>,
) -> Json<IndicatorResponse> {
    // 1. Get price data based on timeframe (from price_window or candle_window)
    // 2. Parse indicators parameter
    // 3. Calculate each indicator
    // 4. Return aligned data
}
```

**Data Source Selection**:
- **1h timeframe**: Use `price_window` (5-second data, last ~1-2 hours)
- **8h/24h timeframe**: Use `candle_window` (5-minute candles, last 24h)

**Indicator Parameter Parsing**:
- `sma_20` → SMA with period 20
- `ema_50` → EMA with period 50
- `rsi_14` → RSI with period 14
- `macd` → MACD with default params (12, 26, 9)
- `macd_8_17_9` → MACD with custom params

---

### Phase 3: Frontend UX Integration

**Location**: `frontend/src/main.rs` (extend existing chart components)

#### 3.1 UI Controls

Add indicator selection panel below chart type/timeframe toggles:

```
┌─────────────────────────────────────────────────────────┐
│ [Line] [Candles]   [1H] [8H] [24H]                     │
│                                                          │
│ Indicators:                                              │
│ ☑ SMA(20)  ☑ SMA(50)  ☐ EMA(12)  ☑ RSI(14)  ☐ MACD    │
└─────────────────────────────────────────────────────────┘
```

**State Management**:
```rust
// Add to App() component
let mut selected_indicators = use_signal(|| vec![
    "sma_20".to_string(),
    "sma_50".to_string(),
]);
```

#### 3.2 Chart Rendering

**For Line/Candlestick Charts**:
- Overlay moving averages (SMA/EMA) as additional lines
- Use distinct colors (e.g., SMA20=blue, SMA50=orange, EMA12=purple)
- Add legend showing which indicators are active

**For RSI**:
- Render in separate panel below main chart (150px height)
- Show 0-100 scale with horizontal lines at 30 and 70
- Color zones: <30 green (oversold), >70 red (overbought)

**For MACD**:
- Render in separate panel below main chart (150px height)
- Show MACD line (blue), Signal line (red), Histogram (bars, green/red)
- Zero line reference

**Example Layout**:
```
┌─────────────────────────────────────────────────────────┐
│                                                          │
│              Price Chart (with SMA overlays)            │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                    RSI Panel (0-100)                    │
├─────────────────────────────────────────────────────────┤
│               MACD Panel (histogram + lines)            │
└─────────────────────────────────────────────────────────┘
```

#### 3.3 Data Fetching

```rust
// Fetch indicators when selected indicators or timeframe changes
let fetch_indicators = move |asset: &str| {
    let timeframe = selected_timeframe();
    let indicators_param = selected_indicators().join(",");

    spawn(async move {
        let url = format!(
            "{}/indicators?asset={}&timeframe={}&indicators={}",
            API_BASE, asset, timeframe, indicators_param
        );
        if let Ok(resp) = reqwest::get(&url).await {
            if let Ok(data) = resp.json::<IndicatorResponse>().await {
                indicator_data.set(Some(data));
            }
        }
    });
};

use_effect(move || {
    let _deps = (selected_timeframe(), selected_indicators());
    if let AppView::Trading(asset) = &*current_view.peek() {
        fetch_indicators(asset);
    }
});
```

#### 3.4 Component Updates

**Extend PriceChart** (line charts):
```rust
fn PriceChart(props: PriceChartProps) -> Element {
    // ... existing price line rendering ...

    // Add indicator overlays
    if let Some(indicators) = indicator_data() {
        for (name, values) in indicators.indicators {
            if name.starts_with("sma") || name.starts_with("ema") {
                // Render as overlay line with distinct color
                // Build SVG path for indicator values
            }
        }
    }
}
```

**Extend CandlestickChart** (similar pattern for overlays)

**New Components**:
```rust
fn RSIPanel(props: RSIPanelProps) -> Element {
    // Render RSI chart with 30/70 threshold lines
}

fn MACDPanel(props: MACDPanelProps) -> Element {
    // Render MACD histogram + signal/macd lines
}
```

---

### Phase 4: Bot Framework Integration

**Location**: Extend `backend/src/bots/mod.rs`

#### 4.1 Enhanced BotContext

Add pre-calculated indicators to BotContext:

```rust
#[derive(Debug, Clone)]
pub struct BotContext {
    // Existing fields
    pub price_window: Vec<PricePoint>,
    pub base_balance: f64,
    pub quote_balance: f64,
    pub current_price: f64,
    pub base_asset: String,
    pub quote_asset: String,
    pub tick_count: u64,

    // NEW: Pre-calculated indicators
    pub indicators: IndicatorCache,
}

#[derive(Debug, Clone)]
pub struct IndicatorCache {
    // Moving averages (most recent N values)
    pub sma_20: Option<f64>,
    pub sma_50: Option<f64>,
    pub ema_12: Option<f64>,
    pub ema_26: Option<f64>,

    // RSI (0-100)
    pub rsi_14: Option<f64>,

    // MACD
    pub macd_line: Option<f64>,
    pub macd_signal: Option<f64>,
    pub macd_histogram: Option<f64>,
}
```

**Design Decision**: Pre-calculate indicators in the bot execution loop (in `services/bot_service.rs`) rather than having each bot calculate them. This avoids redundant computation and provides a consistent API.

#### 4.2 Update Bot Execution Loop

**Location**: `backend/src/services/bot_service.rs`

```rust
pub async fn execute_bot_tick(
    state: &AppState,
    bot: &mut Box<dyn TradingBot>,
    // ... other params
) {
    // Get price window
    let price_window = state.get_price_window_for_asset(&base_asset).await;

    // Extract prices for indicator calculations
    let prices: Vec<f64> = price_window.iter().map(|p| p.price).collect();

    // Calculate indicators
    let indicators = IndicatorCache {
        sma_20: SMA::new(20).calculate(&prices).last().copied(),
        sma_50: SMA::new(50).calculate(&prices).last().copied(),
        ema_12: EMA::new(12).calculate(&prices).last().copied(),
        ema_26: EMA::new(26).calculate(&prices).last().copied(),
        rsi_14: RSI::new(14).calculate(&prices).last().copied(),
        macd_line: {
            let macd = MACD::new(12, 26, 9).calculate(&prices);
            macd.macd_line.last().copied()
        },
        macd_signal: {
            let macd = MACD::new(12, 26, 9).calculate(&prices);
            macd.signal_line.last().copied()
        },
        macd_histogram: {
            let macd = MACD::new(12, 26, 9).calculate(&prices);
            macd.histogram.last().copied()
        },
    };

    // Build context with indicators
    let ctx = BotContext {
        price_window,
        base_balance,
        quote_balance,
        current_price,
        base_asset,
        quote_asset,
        tick_count,
        indicators, // NEW
    };

    // Call bot's tick method
    let decision = bot.tick(&ctx);

    // ... execute decision
}
```

#### 4.3 Example Bot Using Indicators

**Location**: `backend/src/bots/ma_crossover.rs` (new bot strategy)

```rust
use super::{BotContext, BotDecision, TradingBot};

/// Moving Average Crossover Bot
/// Buy when fast MA crosses above slow MA
/// Sell when fast MA crosses below slow MA
pub struct MACrossoverBot {
    stepsize_quote: f64,
    cooldown_remaining: u32,

    // Track previous MA values to detect crossover
    prev_fast_ma: Option<f64>,
    prev_slow_ma: Option<f64>,
}

impl MACrossoverBot {
    pub fn new(stoploss_amount: f64) -> Self {
        Self {
            stepsize_quote: stoploss_amount * 0.01,
            cooldown_remaining: 0,
            prev_fast_ma: None,
            prev_slow_ma: None,
        }
    }
}

impl TradingBot for MACrossoverBot {
    fn tick(&mut self, ctx: &BotContext) -> BotDecision {
        // Handle cooldown
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining -= 1;
            return BotDecision::DoNothing;
        }

        // Get current MAs from context
        let fast_ma = ctx.indicators.ema_12;
        let slow_ma = ctx.indicators.ema_26;

        // Need valid current and previous MAs
        if let (Some(fast), Some(slow), Some(prev_fast), Some(prev_slow)) =
            (fast_ma, slow_ma, self.prev_fast_ma, self.prev_slow_ma) {

            // Detect bullish crossover (fast crosses above slow)
            if prev_fast <= prev_slow && fast > slow {
                self.cooldown_remaining = 5;
                self.prev_fast_ma = fast_ma;
                self.prev_slow_ma = slow_ma;
                return BotDecision::Buy {
                    quote_amount: self.stepsize_quote,
                };
            }

            // Detect bearish crossover (fast crosses below slow)
            if prev_fast >= prev_slow && fast < slow {
                self.cooldown_remaining = 5;
                self.prev_fast_ma = fast_ma;
                self.prev_slow_ma = slow_ma;
                return BotDecision::Sell {
                    quote_amount: self.stepsize_quote,
                };
            }
        }

        // Update previous values
        self.prev_fast_ma = fast_ma;
        self.prev_slow_ma = slow_ma;

        BotDecision::DoNothing
    }

    fn name(&self) -> &str {
        "MA Crossover Bot"
    }
}
```

**Another Example**: `backend/src/bots/rsi_bot.rs`

```rust
/// RSI Overbought/Oversold Bot
/// Buy when RSI < 30 (oversold)
/// Sell when RSI > 70 (overbought)
pub struct RSIBot {
    stepsize_quote: f64,
    cooldown_remaining: u32,
}

impl TradingBot for RSIBot {
    fn tick(&mut self, ctx: &BotContext) -> BotDecision {
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining -= 1;
            return BotDecision::DoNothing;
        }

        if let Some(rsi) = ctx.indicators.rsi_14 {
            // Oversold condition
            if rsi < 30.0 {
                self.cooldown_remaining = 10;
                return BotDecision::Buy {
                    quote_amount: self.stepsize_quote,
                };
            }

            // Overbought condition
            if rsi > 70.0 {
                self.cooldown_remaining = 10;
                return BotDecision::Sell {
                    quote_amount: self.stepsize_quote,
                };
            }
        }

        BotDecision::DoNothing
    }

    fn name(&self) -> &str {
        "RSI Overbought/Oversold Bot"
    }
}
```

---

## 4. UX Design Recommendations

### 4.1 Indicator Toggle UI

**Desktop Layout** (below chart controls):
```
┌──────────────────────────────────────────────────────────────┐
│ Chart Type: [Line] [Candles]    Timeframe: [1H] [8H] [24H]  │
├──────────────────────────────────────────────────────────────┤
│ Indicators:                                                   │
│ ☑ SMA(20)  ☑ SMA(50)  ☐ EMA(12)  ☐ EMA(26)                  │
│ ☑ RSI(14)  ☐ MACD                                            │
└──────────────────────────────────────────────────────────────┘
```

**Colors**:
- SMA(20): Blue (#2196F3)
- SMA(50): Orange (#FF9800)
- EMA(12): Purple (#9C27B0)
- EMA(26): Teal (#009688)
- RSI: Green/Red zones
- MACD: Blue (MACD), Red (Signal), Green/Red (Histogram)

### 4.2 Chart Layout Options

**Option A: Stacked Panels** (recommended for better clarity)
```
┌─────────────────────────────────────┐
│     Main Chart (400px height)       │
│  Price + Candlesticks + MA overlays │
├─────────────────────────────────────┤
│     RSI Panel (120px height)        │
│  Shows 0-100 scale with zones       │
├─────────────────────────────────────┤
│     MACD Panel (120px height)       │
│  Shows histogram + signal lines     │
└─────────────────────────────────────┘
```

**Option B: Tabbed Panels** (saves vertical space)
```
┌─────────────────────────────────────┐
│     Main Chart (400px height)       │
│  Price + Candlesticks + MA overlays │
├─────────────────────────────────────┤
│ [RSI] [MACD]                         │
│  Shows selected indicator panel     │
└─────────────────────────────────────┘
```

**Recommendation**: Start with Option A (stacked) for simplicity. Users can toggle panels on/off.

### 4.3 Legend/Labels

Add a legend in the top-right corner of the main chart:
```
┌─────────────────────────────────────┐
│ Price Chart        ┌──────────────┐ │
│                    │ — Price      │ │
│                    │ — SMA(20)    │ │
│                    │ — SMA(50)    │ │
│                    └──────────────┘ │
└─────────────────────────────────────┘
```

### 4.4 Hover Tooltips

Extend existing hover tooltips to show indicator values:
```
┌──────────────────────────────────┐
│ 2026-01-28 14:35                 │
│ Price: $88,742.45                │
│ SMA(20): $88,500.00              │
│ SMA(50): $87,800.00              │
│ RSI(14): 62.5                    │
└──────────────────────────────────┘
```

---

## 5. Bot Strategy Recommendations

### 5.1 New Bot Strategies to Implement

1. **MA Crossover Bot** (high priority)
   - Simple and effective
   - Easy to understand
   - Good for trending markets

2. **RSI Bot** (high priority)
   - Identifies extremes
   - Works well in ranging markets
   - Clear buy/sell signals

3. **MACD Bot** (medium priority)
   - More sophisticated
   - Good for momentum trading
   - Requires more tuning

4. **Combined Strategy Bot** (low priority)
   - Uses multiple indicators
   - More complex logic
   - Example: Buy when (RSI < 30 AND MACD crosses up)

### 5.2 Bot Configuration UI

Add indicator parameters to bot configuration:

```
┌─────────────────────────────────────┐
│ Select Bot Strategy:                │
│ ( ) Naive Momentum                  │
│ (•) MA Crossover                    │
│ ( ) RSI Overbought/Oversold         │
│                                     │
│ MA Crossover Settings:              │
│ Fast Period: [12] (EMA)             │
│ Slow Period: [26] (EMA)             │
│                                     │
│ Stoploss: [$1000]                   │
│ [Start Bot]                         │
└─────────────────────────────────────┘
```

---

## 6. Implementation Phases

### Phase 1: Core Indicator Library (Backend)
**Estimate**: 1-2 days
- [ ] Create `backend/src/indicators/` module
- [ ] Implement SMA, EMA calculations
- [ ] Implement RSI calculation
- [ ] Implement MACD calculation
- [ ] Add unit tests for each indicator
- [ ] Verify calculations against known test cases

### Phase 2: API Endpoint (Backend)
**Estimate**: 1 day
- [ ] Create `backend/src/routes/indicators.rs`
- [ ] Implement `/api/indicators` endpoint
- [ ] Add query parameter parsing
- [ ] Add response serialization
- [ ] Test with curl/Postman

### Phase 3: Frontend UI (Basic Overlays)
**Estimate**: 2-3 days
- [ ] Add indicator selection checkboxes to UI
- [ ] Implement indicator data fetching
- [ ] Add MA overlay rendering to PriceChart
- [ ] Add MA overlay rendering to CandlestickChart
- [ ] Add legend display
- [ ] Update hover tooltips with indicator values

### Phase 4: Frontend UI (Separate Panels)
**Estimate**: 2-3 days
- [ ] Create RSIPanel component
- [ ] Create MACDPanel component
- [ ] Add panel toggle controls
- [ ] Implement stacked layout
- [ ] Add proper scaling and axes
- [ ] Style panels consistently

### Phase 5: Bot Framework Integration
**Estimate**: 1-2 days
- [ ] Extend BotContext with IndicatorCache
- [ ] Update bot execution loop to calculate indicators
- [ ] Implement MA Crossover Bot
- [ ] Implement RSI Bot
- [ ] Add bot selection UI with new strategies
- [ ] Test bot execution with indicators

### Phase 6: Polish & Testing
**Estimate**: 1-2 days
- [ ] Add loading states for indicator data
- [ ] Optimize performance (memoization, caching)
- [ ] Add error handling for failed calculations
- [ ] Write integration tests
- [ ] Update documentation
- [ ] Test with different timeframes and assets

**Total Estimate**: 8-13 days of development

---

## 7. Technical Considerations

### 7.1 Performance
- Indicator calculations are O(n) where n = number of data points
- For 1h view: ~720 points (5s intervals) → ~1ms calculation time
- For 24h view: ~288 points (5m intervals) → <1ms calculation time
- **Optimization**: Cache indicator results on backend, recalculate only when new data arrives

### 7.2 Data Alignment
- Indicators return vectors aligned with input prices
- Warmup period values are NaN (e.g., first 19 values of SMA(20))
- Frontend must handle NaN by not rendering those segments

### 7.3 Timeframe Considerations
- **1h view** (5-second data):
  - SMA(20) = last 100 seconds
  - RSI(14) = last 70 seconds
  - May be too short for meaningful indicators

- **8h/24h view** (5-minute data):
  - SMA(20) = last 100 minutes (~1.7 hours)
  - RSI(14) = last 70 minutes (~1.2 hours)
  - Better for technical analysis

**Recommendation**:
- For bots (60s tick), calculate on 5-second data (price_window)
- For UI display, use appropriate data source per timeframe
- Consider adding "resample to 1-minute" option for better indicator clarity on 1h view

### 7.4 Indicator Parameter Defaults

| Indicator | Default Period | Rationale |
|-----------|---------------|-----------|
| SMA | 20, 50, 200 | Common day trading / swing trading periods |
| EMA | 12, 26 | Standard MACD components |
| RSI | 14 | Wilder's original specification |
| MACD | 12, 26, 9 | Standard settings (fast, slow, signal) |

### 7.5 Bot Design Best Practices

1. **Don't over-optimize**: Simple strategies often outperform complex ones
2. **Use cooldowns**: Prevent rapid-fire trading on noisy signals
3. **Combine signals**: Use 2-3 indicators for confirmation (reduces false signals)
4. **Risk management**: Always respect stoploss, use position sizing
5. **Backtesting**: Test strategies on historical data before deployment

---

## 8. API Specification

### GET /api/indicators

**Query Parameters**:
- `asset` (required): "BTC" | "ETH"
- `timeframe` (required): "1h" | "8h" | "24h"
- `indicators` (required): comma-separated list
  - Examples: "sma_20", "ema_12", "rsi_14", "macd"
  - Custom params: "sma_50", "ema_26", "rsi_21", "macd_8_17_9"

**Response**:
```json
{
  "asset": "BTC",
  "timeframe": "1h",
  "timestamps": [1706400000, 1706400005, ...],
  "indicators": {
    "sma_20": [null, null, ..., 88500.0, 88520.0],
    "sma_50": [null, null, ..., 87800.0, 87850.0],
    "rsi_14": [null, null, ..., 62.5, 63.2],
    "macd_line": [null, null, ..., 150.5, 155.2],
    "macd_signal": [null, null, ..., 148.0, 150.0],
    "macd_histogram": [null, null, ..., 2.5, 5.2]
  }
}
```

**Notes**:
- `null` values represent warmup period (insufficient data)
- All vectors have same length as `timestamps`
- Frontend should skip rendering null values

---

## 9. Questions for User

Before implementing, please provide guidance on:

1. **Priority**: Which indicators should we implement first? (Recommendation: Start with SMA/EMA overlays, then RSI)

2. **UI Layout**: Do you prefer stacked panels (Option A) or tabbed panels (Option B)?

3. **Default Indicators**: Should any indicators be enabled by default when users first view charts?

4. **Bot Priority**: Should bot integration happen in parallel with UI work, or after UI is complete?

5. **Indicator Customization**: Should users be able to customize indicator parameters (e.g., change SMA period from 20 to 30)?

6. **Performance vs Features**: Would you prefer to cache indicator calculations on backend (faster, more memory) or calculate on-demand (slower, less memory)?

---

## 10. Summary

**Technical indicators enhance the trading simulator in two key ways**:

1. **UX Enhancement**: Visual overlays and panels help users understand market conditions, identify trends, and make informed trading decisions

2. **Bot Intelligence**: Pre-calculated indicators in BotContext enable sophisticated trading strategies beyond simple price-based logic

**The implementation is clean and modular**:
- Backend indicator library is reusable
- API endpoint provides flexible indicator access
- Frontend components extend existing chart architecture
- Bot framework gains powerful new capabilities without breaking existing bots

**Recommended approach**: Implement in phases, starting with basic MA overlays and RSI panel, then expand to MACD and bot integration. This allows for iterative testing and user feedback.
