# Rust Trading Simulator – Project Definition

## Objective
Full-stack local trading simulator with a Rust backend (Axum) and Dioxus frontend, enabling users to simulate trading strategies, visualize market data, and interact with a portfolio in real-time.

## Current Status: Phase 2 Complete + Trading Pairs ✅

**Live Features:**
- 🔐 User authentication (login/signup/guest mode)
- 💰 Multi-asset trading (BTC/USD, ETH/USD, BTC/ETH)
- 🔄 Full trading pair support with cross-pair pricing
- 💵 Deposit/Withdrawal system with lifetime tracking
- 📊 Real-time price charts (1-hour history, 5s intervals)
- 📈 Live price polling from Coinbase API
- 💼 Portfolio management with persistent storage
- 📜 Complete transaction history (trades, deposits, withdrawals)
- 🗂️ Multi-tab UI (Dashboard, Markets, Trading)
- 💾 SQLite database with user persistence
- 🐳 Docker deployment ready

**Next Phase:** Bot framework and automated trading strategies

## Core Principles
- **Resiliency**: Maintain in-memory sliding windows of price data to ensure continuous availability for bots and charts, even if the API fails temporarily.
- **Granularity**: Track fine-grained data (5s intervals) in memory; aggregate for charts or coarser intervals as needed.
- **Modularity**: Trading bots are asynchronous tasks tied to a user profile and can be enabled/disabled dynamically.

## Data Structures (In-Memory)

### AppState (shared via Arc<RwLock<...>>)
- `users: HashMap<UserId, UserData>`
- `price_window: Vec<PricePoint>` (5s granularity, sliding 24h, capacity: 17280 points)
- `bots: HashMap<UserId, BotTaskHandle>` (Phase 4+)

### UserData
- `username: String`
- `cash_balance: f64` (starts at $10,000)
- `asset_balances: HashMap<Asset, f64>`
- `trade_history: Vec<Trade>` (Phase 2+)

### PricePoint
- `timestamp: DateTime<Utc>`
- `asset: String`
- `price: f64`

### Trade (Transaction)
- `user_id: UserId`
- `transaction_type: TransactionType` (Trade/Deposit/Withdrawal enum)
- `base_asset: Asset` (e.g., BTC in BTC/USD)
- `quote_asset: Asset` (e.g., USD in BTC/USD)
- `side: TradeSide` (Buy/Sell enum)
- `quantity: f64`
- `price: f64` (in quote asset terms)
- `timestamp: DateTime<Utc>`
- `base_usd_price: Option<f64>` (USD snapshot for analytics)
- `quote_usd_price: Option<f64>` (USD snapshot for analytics)



## Phases & Tasks

### Phase 1 – MVP ✅ (COMPLETED)

**Frontend**
- ✅ Single-page app showing user data:
  - ✅ Cash balance, BTC balance
  - ✅ BTC current price (updates every 5s)
  - ✅ Buy/Sell form with quantity input
  - ✅ Trade execution feedback/status display

**Backend**
- ✅ Axum API:
  - ✅ `GET /api/price` → latest price
  - ✅ `GET /api/portfolio` → user balances
  - ✅ `POST /api/trade` → execute buy/sell
- ✅ Trading logic:
  - ✅ Simple order execution handlers (in-memory)
  - ✅ Validation (insufficient funds, invalid quantity)
- ✅ Sliding window:
  - ✅ Poll Coinbase API every 5s
  - ✅ Append to in-memory 5s window (24h capacity)
  - ✅ Resilient to API failures (continues polling)
- ✅ State management:
  - ✅ Thread-safe `Arc<RwLock<AppState>>`
  - ✅ Demo user with $10,000 starting balance

**Docker**
- ✅ Multi-stage Dockerfile
- ✅ Frontend built with Dioxus CLI (`dx build`)
- ✅ Backend serves static frontend files
- ✅ Single container deployment on port 3000



### Phase 2 – Persistence & Historical Data ✅ (COMPLETED)

**Persistence** ✅ (COMPLETED)
- ✅ Database setup (SQLite)
  - ✅ User profiles table with username, cash_balance, asset_balances, password_hash
  - ✅ Migration system using sqlx migrations
  - ✅ Named Docker volume (`trading-sim-data`) for data persistence across restarts
  - ✅ Database queries for CRUD operations on users
- ✅ Demo user behavior:
  - ✅ Resets to $10,000 on every app restart (memory-only, not persisted)
  - ✅ Deleted from DB on startup to ensure fresh state
  - ✅ Only authenticated users persist to database

**Authentication** ✅ (COMPLETED)
- ✅ User registration endpoint (`POST /api/signup`)
- ✅ Login endpoint (`POST /api/login`)
- ✅ Logout functionality (frontend clears session)
- ✅ Password hashing with bcrypt (DEFAULT_COST)
- ✅ UUID-based user IDs
- ✅ Frontend authentication UI:
  - ✅ Login form with username/password
  - ✅ Signup form with validation (min 6 characters)
  - ✅ "Continue as Guest" option for demo profile
  - ✅ Logout button in trading view
  - ✅ Input validation and error messages
- ✅ Session approach: User ID stored client-side, sent with API requests (MVP approach)
- ✅ Routes updated to accept `user_id` query parameter

**Design Decision: Authentication Approach**
- Chose simple MVP approach: user_id stored client-side and passed with requests
- No session cookies or JWT tokens for initial version (keeps implementation simple)
- Sufficient for local single-machine deployment
- Can upgrade to proper session management later for hosted deployment

**Charts** ✅ (COMPLETED)
- ✅ 1-hour price graph component (SVG-based)
- ✅ Real-time data aggregation from 5s price window (720 points)
- ✅ Backfill historical data from Coinbase API on startup
  - ✅ Fetches 1-minute candles from Coinbase Exchange API
  - ✅ Linear interpolation to 5-second intervals for smooth charts
- ✅ Chart features:
  - ✅ Grid lines (5 horizontal, 6 vertical)
  - ✅ Axis labels (price in USD, time in minutes ago)
  - ✅ Auto-scaling based on min/max prices
  - ✅ Updates every 30 seconds
- ✅ Custom SVG rendering (no external chart library needed)

**Design Decision: Historical Data Strategy**
- Coinbase Exchange API provides 1-minute granularity (not 5-second)
- Implemented linear interpolation between 1-minute candles to create smooth 5s data
- Falls back to simulated data if API fails
- User-Agent header required for Coinbase API requests

**Trading History** ✅ (COMPLETED)
- ✅ Trade history storage in UserData (Vec<Trade>)
- ✅ Persist trades to database (JSON serialization in SQLite)
- ✅ Display last 10 trades in Dashboard (with count message)
- ✅ Filter trades by asset in trading view
- ✅ Show all trades (unfiltered) in dashboard
- ✅ Timestamp formatting for trade display

**Multi-Asset Support** ✅ (COMPLETED)
- ✅ Support 3 active markets: BTC/USD, ETH/USD, BTC/ETH
- ✅ Full trading pair model (base_asset/quote_asset)
- ✅ Cross-pair pricing (BTC/ETH calculated from BTC-USD / ETH-USD)
- ✅ USD snapshot tracking for portfolio analytics
- ✅ Multiple price polling services for each asset (separate tokio tasks)
- ✅ Tabular navigation structure:
  - ✅ Dashboard tab (all balances, name, transaction history)
  - ✅ Markets tab (preview of all 3 markets with live graphs and prices)
  - ✅ Trading view (per-pair trading interface with pair-specific history)
- ✅ Asset-specific price windows (720 points per asset)
- ✅ Trade form with dynamic pair context
- ✅ Active tab highlighting in navigation
- ✅ Multi-asset portfolio display

**Design Decision: Trading Pairs Implementation**
- Chose standard financial model: base_asset, quote_asset, price (in quote terms), quantity (of base)
- USD snapshots captured at trade time for portfolio analytics and P&L calculations
- Cross-pair pricing computed on-demand (no separate API calls needed)
- Backward compatible via serde defaults (old trades automatically migrated)
- Trading service handles all pairs uniformly (USD, non-USD, and cross-pairs)

**Deposit/Withdrawal System** ✅ (COMPLETED)
- ✅ Deposit endpoint with validation ($10 min, $100K max)
- ✅ Withdrawal endpoint with balance checking
- ✅ Unified transaction history (trades, deposits, withdrawals)
- ✅ Lifetime statistics tracking:
  - ✅ Lifetime funding (seed + deposits)
  - ✅ Lifetime deposits
  - ✅ Lifetime withdrawals
- ✅ Dashboard UI with funding controls
- ✅ Transaction type indicators in history table

**Design Decision: Transaction Model**
- Extended Trade struct with TransactionType enum instead of separate models
- Deposits/withdrawals appear in unified transaction history
- Lifetime stats calculated from transaction history (no separate fields)
- All transactions tracked in same Vec<Trade> for chronological ordering



### Phase 3 – Bot Framework & Advanced Features

**Bot Framework** (HIGH PRIORITY)
- [ ] Bot framework that can read price window
- [ ] Placeholder bot strategies (no execution yet)
- [ ] Bot state tracking structure
- [ ] Bot configuration interface

**User Operations** ✅ (COMPLETED)
- ✅ Deposit/withdraw USD endpoints
- ✅ Frontend forms for deposits/withdrawals
- ✅ Balance validation and transaction history
- ✅ Lifetime funding statistics

**Charts** (OPTIONAL ENHANCEMENT)
- [ ] Interactive charts with variable time windows
- [ ] Zoom/pan functionality
- [ ] Multiple timeframe selection (1h, 24h, 7d)
- [ ] Candlestick charts



### Phase 4 – UX & Automation

**Frontend**
- [ ] Landing page
- [ ] Login/signup pages
- [ ] User profile page
- [ ] Improved trading dashboard
- [ ] Better UI/UX design and styling

**Trading Bots**
- [ ] Bot enable/disable per user profile
- [ ] Spawn Tokio tasks for asynchronous bot execution
- [ ] Bot reads price window and executes trades autonomously
- [ ] Bot configuration interface
- [ ] Bot performance tracking

**Advanced Features**
- [ ] WebSocket support for real-time updates
- [ ] Live price streaming to frontend
- [ ] Multiple bot strategies per user
- [ ] Strategy selection/configuration UI
- [ ] Bot marketplace or strategy library



---

## API Cadence and Data Handling

- **Polling interval**: 5s for fine-grained real-time price window
- **Backfill**: 24h of 1-minute candles from API on startup (Phase 2+, interpolated if necessary)
- **Aggregation**: 1-hour or 1-minute graphs derived from 5s sliding window

## Key Considerations

### Bots as Tokio Tasks
- Each active bot runs independently per user profile
- Reads shared price window
- Executes trades on in-memory or persisted user portfolio
- Enable/disable dynamically

### Resiliency
- Sliding window ensures minimal API dependency for charts and bots
- Temporary API failures do not break simulation
- Continues polling and operation during network issues

### Scalability
- Single-machine first; can extend to multi-user hosted deployment later
- Data structures designed to support multiple users/bots
- Thread-safe state management with `Arc<RwLock>`

## Task Summary Table

| Phase | Task | Category | Status |
|-------|------|----------|--------|
| 1 | Single-page frontend | Frontend | ✅ Complete |
| 1 | In-memory user data | Backend | ✅ Complete |
| 1 | Price polling (5s) | Backend | ✅ Complete |
| 1 | Trading logic | Backend | ✅ Complete |
| 1 | Axum API endpoints | Backend | ✅ Complete |
| 1 | Docker | DevOps | ✅ Complete |
| 2 | Persistent storage | Backend | ✅ Complete |
| 2 | Login/signup | Backend/Frontend | ✅ Complete |
| 2 | 1-hour price graph | Frontend | ✅ Complete |
| 2 | Historical data backfill | Backend | ✅ Complete |
| 2 | Trading history | Backend/Frontend | ✅ Complete |
| 2 | Multi-asset support (BTC/ETH) | Backend/Frontend | ✅ Complete |
| 2 | Multi-tab navigation | Frontend | ✅ Complete |
| 2 | Trading pairs (BTC/ETH cross-pair) | Backend/Frontend | ✅ Complete |
| 2 | Deposit/withdraw system | Frontend/Backend | ✅ Complete |
| 3 | Bot framework | Backend | Pending |
| 3 | Interactive graphs | Frontend | Pending |
| 4 | UX improvements | Frontend | Pending |
| 4 | Async trading bots | Backend | Pending |
| 4 | WebSockets | Backend/Frontend | Pending |
| 4 | Multiple bot strategies | Backend | Pending |

## Current Implementation Status

### Files Implemented

**Backend**
- [backend/src/main.rs](backend/src/main.rs) - Axum server setup, API routes, static file serving, database initialization
- [backend/src/state.rs](backend/src/state.rs) - AppState with Arc<RwLock>, demo user reset behavior, database integration
- [backend/src/models.rs](backend/src/models.rs) - Data structures (PricePoint, UserData, Trade, TradeSide)
- [backend/src/api_client.rs](backend/src/api_client.rs) - Coinbase API client with historical data fetching and interpolation
- [backend/src/db/mod.rs](backend/src/db/mod.rs) - Database module with migration support
- [backend/src/db/queries.rs](backend/src/db/queries.rs) - User CRUD operations, authentication queries
- [backend/src/services/price_service.rs](backend/src/services/price_service.rs) - Price polling, historical data backfill
- [backend/src/services/trading_service.rs](backend/src/services/trading_service.rs) - Trade execution logic
- [backend/src/services/auth_service.rs](backend/src/services/auth_service.rs) - Password hashing, user ID generation
- [backend/src/routes/price.rs](backend/src/routes/price.rs) - GET /api/price, GET /api/price/history endpoints
- [backend/src/routes/portfolio.rs](backend/src/routes/portfolio.rs) - GET /api/portfolio endpoint (with user_id)
- [backend/src/routes/trade.rs](backend/src/routes/trade.rs) - POST /api/trade endpoint (with user_id)
- [backend/src/routes/auth.rs](backend/src/routes/auth.rs) - POST /api/signup, POST /api/login endpoints
- [backend/migrations/](backend/migrations/) - SQLite migrations for users table and password field

**Frontend**
- [frontend/src/main.rs](frontend/src/main.rs) - Dioxus app with:
  - Authentication UI (login/signup/guest)
  - SVG-based price chart
  - Portfolio display
  - Trade form
  - Logout functionality

**DevOps**
- [Dockerfile](Dockerfile) - Multi-stage build for frontend and backend
- Docker volume: `trading-sim-data` for SQLite persistence

### Running the Project

```bash
# Build the Docker image
docker build -t rust-trading-simulator .

# Run the container (with persistent volume)
docker run -d --name sim -p 3000:3000 -v trading-sim-data:/app/data rust-trading-simulator

# Access in browser
http://localhost:3000

# Stop the container
docker stop sim

# Remove the container
docker rm sim

# View logs
docker logs sim -f
```

### Current Status & Next Steps

**Phase 1 (MVP)** ✅ Complete
**Phase 2 (Persistence & Historical Data)** ✅ Complete

**Completed in Phase 2:**
- ✅ SQLite database with full CRUD operations
- ✅ User authentication (login/signup/guest)
- ✅ Password hashing with bcrypt
- ✅ Trade history tracking (Vec<Trade> in UserData)
- ✅ Multi-asset support (BTC/USD, ETH/USD)
- ✅ Multi-tab navigation (Dashboard, Markets, Trading)
- ✅ Asset-specific price charts and trade history
- ✅ Live price polling for multiple assets (separate tasks per asset)
- ✅ Active tab highlighting in navigation

**Recommended Next Steps (Phase 3):**

1. **Bot Framework** (High Priority - PRIMARY FOCUS)
   - Create bot trait/interface that reads price window
   - Implement placeholder bot strategies (no execution yet)
   - Add bot state tracking to AppState
   - Design bot configuration and storage
   - Bot enable/disable per user

2. **Chart Enhancements** (Optional)
   - Interactive charts with zoom/pan
   - Multiple timeframe selection (1h, 24h, 7d)
   - Candlestick charts
   - Technical indicators (MA, RSI, MACD)

3. **Additional Assets** (Optional)
   - Add more USD pairs (SOL, DOGE, ADA, etc.)
   - Additional cross-pairs (ETH/BTC, etc.)

**Phase 4 Focus:**
- Bot execution with async tasks
- WebSocket support for real-time updates
- Multiple bot strategies per user
- Enhanced UX and styling

**Key Achievements:**
- Real historical data integration with interpolation
- Secure authentication with bcrypt
- SVG-based charting without external libraries
- Database persistence with demo user reset behavior
- Clean separation between guest and authenticated user experiences
- Multi-asset trading with BTC, ETH, and cross-pairs
- Full trading pair model with USD snapshots for analytics
- Cross-pair pricing calculation (BTC/ETH)
- Comprehensive transaction history (trades, deposits, withdrawals)
- Deposit/withdrawal system with lifetime statistics
- Professional multi-tab navigation UI
- Backward-compatible schema migrations
