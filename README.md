# Rust Trading Simulator – Project Definition

## Overview

A full-stack cryptocurrency trading simulator that enables users to practice trading strategies in a risk-free environment with real market data. The platform consists of two primary components: a mock trading platform that simulates multi-asset trading with live price feeds from Coinbase, and a modular bot framework that allows users to deploy automated trading strategies with configurable risk parameters. Built with Rust (Axum backend, Dioxus frontend), the simulator maintains persistent user portfolios, tracks comprehensive transaction history, and provides real-time price visualization while supporting both manual trading and algorithmic bot execution on a 60-second decision cadence.

### Getting Started

```bash
# 1 - Pull in repo and navigate to project directory

# 2 - Stop the container (ignore if first time running)
docker stop sim

# 3 - Remove the container (ignore if first time running)
docker rm sim

# 4 - Build the Docker image
docker build -t rust-trading-simulator .

# 5 - Run the container (with persistent volume on your machine)
docker run -d --name sim -p 3000:3000 -v trading-sim-data:/app/data rust-trading-simulator

# 6 - View application in browser
http://localhost:3000

# View verbose logs for debugging 
docker logs sim -f
```

**Tips** : To enter the simulator you may continue as a guest or create a new profile. When using the demo (guest profile) note that user data does not survive application restarts. To have a long-lived account which  persists your account data, you must create a profile. A new profile can be created simply by providing a username and password into the standard login form and pressing "sign-up". 

## Mock Trading Platform High-Level Design

The mock trading platform simulates a real cryptocurrency exchange environment by polling live market data from Coinbase every 5 seconds and maintaining an in-memory sliding window of price history. Users can trade three asset pairs (BTC/USD, ETH/USD, BTC/ETH) with full support for cross-pair pricing calculations, manage their portfolios through deposits and withdrawals, and view comprehensive transaction history with lifetime statistics. The platform supports both authenticated users with persistent SQLite storage and guest users with session-only data, providing a multi-tab interface for dashboard overview, market exploration, and active trading.

The trading interface includes both line and candlestick chart views with technical indicators (SMA, EMA, RSI) that can be toggled on demand. Indicators are calculated server-side and overlaid on price charts, with RSI displayed in a separate panel below the main chart. These same indicators are pre-calculated and provided to trading bots through the BotContext for strategy implementation.

**Key Design Points:**

- **Resilient Price Data Architecture**: Maintains a 24-hour sliding window of 5-second price data in memory, with historical backfill from Coinbase's 1-minute candles (linearly interpolated). Continues operation during temporary API failures, ensuring bots and charts always have access to price data.

- **Trading Pair Model**: Implements standard financial pair semantics with base_asset, quote_asset, and pricing in quote terms. Cross-pair pricing (e.g., BTC/ETH) is computed dynamically from USD pairs. USD snapshots captured at trade time enable accurate portfolio analytics across all trading pairs.

- **Multi-User Support**: Thread-safe state management using `Arc<RwLock<AppState>>` supports concurrent users with isolated portfolios. SQLite persistence for authenticated users, in-memory-only for guest accounts that reset on restart.

- **Transaction Model**: Unified transaction history tracking trades, deposits, and withdrawals with a single Trade struct using a TransactionType enum. Enables comprehensive lifetime statistics (total funding, trade volume, withdrawals) calculated on-demand from transaction history.

- **Account Funding**: Users can deposit ($10 min, $100K max) and withdraw USD to simulate realistic portfolio management and enable testing of capital allocation strategies across multiple assets.


## Modular Trading Bot Framework High-Level Design

Bots are trait-based modules that own their internal state and execute trading decisions based on raw market data. The framework provides immutable context each tick while bots maintain mutable state across cycles.

**Bot State Management**: Each bot instance owns its state entirely in memory - accumulators, custom data structures, flags, and bot-specific trade history are all managed within the bot struct itself. This state exists only during the bot's lifetime and does not persist across app restarts. When a bot starts, it begins with a clean slate. When stopped or the app restarts, all bot state is discarded.

**Decision Model - Quote Asset Terms**: All trading decisions are expressed in quote asset terms (e.g., USD for BTC/USD pairs). The user provides a stoploss amount in quote asset (e.g., $10,000), and bots dispatch decisions like "Buy $100 worth of BTC" or "Sell $100 worth of BTC". This creates an intuitive mental model where stoploss, step-size, and decisions all operate in the same currency unit. The framework converts quote amounts to base quantities during execution using current market price.

**Data Access**: Bots receive raw, uninterpolated price data from the 5-second polling window (not the interpolated historical backfill). The BotContext includes the raw price_window Vec<PricePoint>, current balances, current market price, trading pair metadata, and pre-calculated technical indicators (SMA, EMA, RSI, MACD). These indicators are computed by the framework before each tick to avoid redundant calculation across bots. Bots do NOT see global trade history - they only see their own trades, which they can track as part of their internal state if needed.

**Stoploss Enforcement**: The framework (not the bot) is responsible for stoploss checking. Stoploss is evaluated against total portfolio value (all assets converted to USD equivalent) since bots impose a full trading lock across all markets. The reference point is the portfolio value when the bot started. After each tick, before executing any trade decision, the framework calculates current portfolio value and terminates the bot if losses exceed the stoploss threshold.

**Framework vs Bot Responsibilities**: The framework handles validation (sufficient balance, valid quantities), execution (converting quote amounts to base quantities, executing trades at market price), stoploss monitoring, and bot lifecycle (start/stop/error handling). The bot only needs to implement the `tick()` method which examines context and returns a BotDecision. Bots can maintain arbitrary state between ticks using standard Rust fields in their struct - counters, moving averages, custom indicators, or any algorithm-specific data.

**Asynchronous Execution with Tokio**: Each active bot runs as an independent Tokio task spawned via `tokio::spawn()`, enabling concurrent execution of multiple bots without blocking the main API server or each other. The task maintains a 60-second interval timer using Tokio's async primitives, yielding control between ticks to allow efficient resource sharing. Each bot task holds a `JoinHandle` stored in `AppState` for lifecycle management - graceful shutdown is signaled by removing the bot from the active_bots map, while forceful termination uses `.abort()` on the handle. This architecture provides lightweight concurrency, allowing hundreds of bot instances to run simultaneously with minimal overhead.

**Example Flow**: User starts a bot with $10,000 stoploss on BTC/USD market. Bot struct initializes with empty state. A Tokio task spawns and every 60 seconds: (1) Framework assembles BotContext with latest price window and balances, (2) Calls bot's `tick()` method which updates internal state and returns decision, (3) Framework validates decision won't breach stoploss or balances, (4) Executes trade if valid, marking it as bot-executed in transaction history, (5) Repeats until user stops, stoploss hit, insufficient funds, or task error.

## Data Model Design

The application uses a hybrid data model combining in-memory state for real-time operations and SQLite persistence for user data. In-memory structures (AppState, PricePoint, BotInstance) are shared across threads using `Arc<RwLock<>>` for thread-safe concurrent access, while the database stores only essential user information with JSON serialization for complex fields. Bot state exists entirely in memory and is not persisted - each bot maintains its own internal state during execution and discards it upon termination. The price window operates as a fixed-size circular buffer storing 24 hours of 5-second data points (17,280 entries), providing resilient data access for both charts and bot algorithms.

### In-Memory Data Structures

**AppState** (shared via `Arc<RwLock<AppStateInner>>`)
- `users: HashMap<UserId, UserData>` - All user portfolios in memory
- `price_window: Vec<PricePoint>` - 24-hour sliding window (5s granularity, capacity: 17,280 points)
- `active_bots: HashMap<UserId, BotInstance>` - Currently running bots (one per user maximum)

**UserData**
- `username: String`
- `cash_balance: f64` - Legacy field (kept for backward compatibility)
- `asset_balances: HashMap<Asset, f64>` - Current holdings (USD, BTC, ETH, etc.)
- `trade_history: Vec<Trade>` - Complete transaction history (trades, deposits, withdrawals)

**PricePoint**
- `timestamp: DateTime<Utc>` - When the price was captured
- `asset: String` - Asset symbol (BTC, ETH)
- `price: f64` - USD price at timestamp

**Trade** (unified transaction model)
- `user_id: UserId`
- `transaction_type: TransactionType` - Enum: Trade, Deposit, or Withdrawal
- `base_asset: Asset` - Base asset in trading pair (e.g., BTC in BTC/USD)
- `quote_asset: Asset` - Quote asset in trading pair (e.g., USD in BTC/USD)
- `side: TradeSide` - Enum: Buy or Sell
- `quantity: f64` - Amount of base asset
- `price: f64` - Price in quote asset terms
- `timestamp: DateTime<Utc>`
- `base_usd_price: Option<f64>` - USD snapshot of base asset at trade time (for analytics)
- `quote_usd_price: Option<f64>` - USD snapshot of quote asset at trade time (for analytics)
- `executed_by_bot: Option<String>` - Bot name if trade was automated, None if manual

**BotInstance** (bot runtime tracking)
- `bot_name: String` - Display name of the bot strategy
- `trading_pair: (String, String)` - Tuple of (base_asset, quote_asset)
- `stoploss_amount: f64` - Loss threshold in quote asset terms
- `initial_portfolio_value_usd: f64` - Portfolio value when bot started (for stoploss calculation)
- `task_handle: JoinHandle<()>` - Tokio task handle for lifecycle management

### Database Schema (SQLite)

**users table**
- `user_id TEXT PRIMARY KEY` - UUID string
- `username TEXT NOT NULL` - Display name
- `password_hash TEXT` - Bcrypt hash (nullable for guest users)
- `cash_balance REAL DEFAULT 10000.0` - Legacy field (migrated to asset_balances)
- `asset_balances TEXT DEFAULT '{}'` - JSON serialized HashMap<Asset, f64>
- `trade_history TEXT` - JSON serialized Vec<Trade>
- `created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP`
- `updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP`

**Key Database Characteristics:**
- **No bot persistence**: Bot state is never written to database - bots start fresh on user request
- **JSON serialization**: Complex structures (asset_balances, trade_history) stored as JSON text fields
- **Backward compatibility**: Migration logic handles old cash_balance field by copying to asset_balances["USD"]
- **Guest user behavior**: demo_user is deleted from DB on startup and exists only in memory

### Bot Framework Data Structures

**BotContext** (immutable context passed to bot each tick)
- `price_window: Vec<PricePoint>` - Raw 5s price data (not interpolated), typically last 720 points (1 hour)
- `base_balance: f64` - Current holdings of base asset
- `quote_balance: f64` - Current holdings of quote asset
- `current_price: f64` - Most recent market price
- `base_asset: String` - Trading pair base (e.g., "BTC")
- `quote_asset: String` - Trading pair quote (e.g., "USD")
- `tick_count: u64` - Number of ticks since bot started (0-indexed)
- `indicator_data: Option<IndicatorData>` - Pre-calculated technical indicators (SMA, EMA, RSI values) when available

**BotDecision** (bot's output each tick)
- `DoNothing` - Skip this cycle
- `Buy { quote_amount: f64 }` - Buy worth X in quote asset (e.g., "buy $100 of BTC")
- `Sell { quote_amount: f64 }` - Sell worth X in quote asset (e.g., "sell $100 of BTC")

**Note**: Individual bot implementations (e.g., NaiveMomentumBot) define their own internal state structures which are not standardized - they can include any fields needed for their strategy (counters, moving averages, flags, price history, etc.).
