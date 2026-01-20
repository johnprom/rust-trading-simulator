# Rust Trading Simulator – Project Definition

## Objective
Full-stack local trading simulator with a Rust backend (Axum) and Dioxus frontend, enabling users to simulate trading strategies, visualize market data, and interact with a portfolio in real-time.

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

### Trade
- `user_id: UserId`
- `asset: Asset`
- `side: TradeSide` (Buy/Sell enum)
- `quantity: f64`
- `price: f64`
- `timestamp: DateTime<Utc>`



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



### Phase 2 – Persistence & Historical Data

**Persistence**
- [ ] Database setup (SQLite or Postgres)
  - [ ] User profiles table
  - [ ] Cash/asset balances
  - [ ] Migration system

**Authentication**
- [ ] User registration endpoint
- [ ] Login/logout endpoints
- [ ] Session management or JWT tokens
- [ ] Password hashing (bcrypt/argon2)
- [ ] Frontend login/signup forms

**Charts**
- [ ] 1-hour price graph component
- [ ] Aggregate data from 5s price window
- [ ] Optional: Backfill historical data from API on startup
- [ ] Chart library integration (e.g., plotters, charming)

**Bots**
- [ ] Bot framework that can read price window
- [ ] Placeholder bot strategies (no execution yet)
- [ ] Bot state tracking structure



### Phase 3 – Expanded Functionality

**User Operations**
- [ ] Mock deposit/withdraw USD endpoints
- [ ] Frontend forms for deposits/withdrawals
- [ ] Balance validation and updates

**Trading History**
- [ ] Persist trade history to database
- [ ] Trades table schema
- [ ] Frontend component to display trade history
- [ ] Filtering/pagination for trade list

**Multi-Asset Support**
- [ ] Support multiple trading pairs (ETH, SOL, etc.)
- [ ] Multiple price polling services
- [ ] Asset selection in trade form
- [ ] Portfolio display for multiple assets

**Charts**
- [ ] Interactive charts with variable time windows
- [ ] Zoom/pan functionality
- [ ] Dynamic updates (live chart)
- [ ] Multiple timeframe selection (1h, 24h, 7d)



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
| 2 | Persistent storage | Backend | Pending |
| 2 | Login/signup | Backend/Frontend | Pending |
| 2 | 1-hour price graph | Frontend | Pending |
| 2 | Bot integration | Backend | Pending |
| 3 | Mock deposit/withdraw | Frontend/Backend | Pending |
| 3 | Trading history | Backend/Frontend | Pending |
| 3 | Multi-asset support | Backend/Frontend | Pending |
| 3 | Interactive graphs | Frontend | Pending |
| 4 | UX improvements | Frontend | Pending |
| 4 | Async trading bots | Backend | Pending |
| 4 | WebSockets | Backend/Frontend | Pending |
| 4 | Multiple bot strategies | Backend | Pending |

## Current Implementation Status

### Files Implemented

**Backend**
- [backend/src/main.rs](backend/src/main.rs) - Axum server setup, API routes, static file serving
- [backend/src/state.rs](backend/src/state.rs) - AppState with Arc<RwLock>, demo user initialization
- [backend/src/models.rs](backend/src/models.rs) - Data structures (PricePoint, UserData, Trade, TradeSide)
- [backend/src/api_client.rs](backend/src/api_client.rs) - Coinbase API client
- [backend/src/services/price_service.rs](backend/src/services/price_service.rs) - Price polling every 5s
- [backend/src/services/trading_service.rs](backend/src/services/trading_service.rs) - Trade execution logic
- [backend/src/routes/price.rs](backend/src/routes/price.rs) - GET /api/price endpoint
- [backend/src/routes/portfolio.rs](backend/src/routes/portfolio.rs) - GET /api/portfolio endpoint
- [backend/src/routes/trade.rs](backend/src/routes/trade.rs) - POST /api/trade endpoint

**Frontend**
- [frontend/src/main.rs](frontend/src/main.rs) - Dioxus single-page app with price display, portfolio, and trade form

**DevOps**
- [Dockerfile](Dockerfile) - Multi-stage build for frontend and backend

### Running the Project

```bash
# Build the Docker image
docker build -t rust-trading-simulator .

# Run the container
docker run --name sim -p 3000:3000 rust-trading-simulator

# Access in browser
http://localhost:3000
```

### Next Steps

The MVP (Phase 1) is complete. Ready to begin Phase 2 work on persistence and authentication.
