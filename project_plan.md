# Project Plan - Task Tracking

## Task Summary

| Phase | Task | Status |
|-------|------|--------|
| 1 | MVP (frontend, backend, Docker) | ✅ Complete |
| 2 | Persistence & authentication | ✅ Complete |
| 2 | Charts & historical data | ✅ Complete |
| 2 | Multi-asset support & trading pairs | ✅ Complete |
| 2 | Deposit/withdrawal system | ✅ Complete |
| 3 | Bot framework infrastructure | ✅ Complete |
| 3 | Bot execution & lifecycle | ✅ Complete |
| 3 | Bot UI & API | ✅ Complete |
| 3 | Naive Momentum Bot strategy | ✅ Complete |
| 4 | Chart fixes & enhancements | Pending |
| 4 | UX improvements | Pending |
| 4 | Additional bot strategies | Pending |
| 5 | Real-time features (WebSockets) | Pending |
| 5 | Advanced features | Pending |

---

## Implementation Files

**Backend**
- `backend/src/main.rs` - Server, routes, DB initialization
- `backend/src/state.rs` - AppState, BotInstance tracking
- `backend/src/models.rs` - Data structures (Trade, UserData, PricePoint)
- `backend/src/bots/` - Bot framework (mod.rs, naive_momentum.rs)
- `backend/src/services/` - Business logic (price, trading, auth, bot)
- `backend/src/routes/` - API endpoints (price, portfolio, trade, auth, bot)
- `backend/src/db/` - Database layer (queries, migrations)
- `backend/migrations/` - SQLite schema migrations

**Frontend**
- `frontend/src/main.rs` - Dioxus app with all UI components

**DevOps**
- `Dockerfile` - Multi-stage build with automated testing
- `trading-sim-data` volume - SQLite persistence

---

## Phase 1 – MVP ✅

**Frontend**
- ✅ Single-page app with trading interface
- ✅ Cash balance, BTC balance display
- ✅ BTC current price (updates every 5s)
- ✅ Buy/Sell form with quantity input
- ✅ Trade execution feedback

**Backend**
- ✅ Axum API endpoints (price, portfolio, trade)
- ✅ Trading logic with validation
- ✅ Sliding window (5s polling, 24h capacity)
- ✅ Thread-safe state management (`Arc<RwLock<AppState>>`)

**DevOps**
- ✅ Multi-stage Dockerfile
- ✅ Single container deployment on port 3000

---

## Phase 2 – Persistence & Multi-Asset ✅

**Database & Authentication**
- ✅ SQLite with sqlx migrations
- ✅ User authentication (login/signup/guest)
- ✅ Password hashing with bcrypt
- ✅ Demo user memory-only behavior

**Multi-Asset Trading**
- ✅ Three trading pairs (BTC/USD, ETH/USD, BTC/ETH)
- ✅ Cross-pair pricing calculations
- ✅ USD snapshots for analytics
- ✅ Deposit/withdrawal system
- ✅ Lifetime statistics tracking

**Charts & UI**
- ✅ Multi-tab navigation (Dashboard, Markets, Trading)
- ✅ SVG-based 1-hour price charts
- ✅ Historical backfill with interpolation
- ✅ Transaction history with filtering

---

## Phase 3 – Bot Framework ✅

**Bot Infrastructure**
- ✅ TradingBot trait interface
- ✅ BotContext (price window, balances, current price)
- ✅ BotDecision enum (DoNothing, Buy, Sell in quote terms)
- ✅ BotInstance tracking in AppState
- ✅ PriceHistory template helper

**Bot Execution**
- ✅ Tokio task-based async execution
- ✅ 60-second tick cadence
- ✅ Stoploss monitoring (total portfolio value)
- ✅ Trading lock (one bot per user)
- ✅ Graceful shutdown (user stop, stoploss, errors)
- ✅ Bot-executed trades marked in history

**Bot API & UI**
- ✅ POST /api/bot/start endpoint
- ✅ POST /api/bot/stop endpoint
- ✅ GET /api/bot/status endpoint
- ✅ Frontend bot controls (strategy selection, stoploss)
- ✅ Real-time bot status display
- ✅ Bot trade indicators in history tables
- ✅ Automatic portfolio polling when bot active

**Bot Strategies**
- ✅ Naive Momentum Bot (3-tick trend, 3-tick cooldown)

---

## Phase 4 – Chart Enhancements & UX (NEXT)

**Current Chart Fixes**
- ✅ Fix y-axis formatting for cross-pairs (e.g., BTC/ETH should show ETH not $)
- ✅ Add back button from Trading view to Markets tab
- ✅ Update x-axis to show real time instead of "minutes ago"

**Interactive Chart Features**
- ✅ Visually appealing charts with hover tooltip with crosshair for precise values
- ✅ Multiple timeframe selection (1h, 8h, 24h)
- ✅ Candlestick charts with toggleable view
  - ✅ Backend OHLC data model (Candle struct)
  - ✅ Dual-window architecture (1m candles for 1h, 5m candles for 8h/24h)
  - ✅ API endpoint `/api/price/candles` with timeframe support
  - ✅ Frontend candlestick component with SVG rendering
  - ✅ Toggle button between line and candlestick views
  - ✅ Hover tooltip showing OHLC data

**Advanced Chart Enhancements**
- ✅ Technical indicators (SMA_20, SMA_50) - feature only enabled for 1h (5-second) window
  - ✅ Backend SMA calculation with unit tests
  - ✅ API endpoint `/api/indicators` with SMA support
  - ✅ Frontend toggles and overlay rendering
  - ✅ Orange (SMA20) and Purple (SMA50) color scheme
- ✅ Technical indicators (EMA_12, EMA_26) - feature only enabled for 1h (5-second) window
  - ✅ Backend EMA calculation with unit tests
  - ✅ API endpoint extended for EMA
  - ✅ Frontend toggles and overlay rendering
  - ✅ Teal (EMA12) and Deep Orange (EMA26) color scheme
- ✅ Technical indicators (RSI_14) and Panel - feature only enabled for 1h (5-second) window
  - ✅ Backend RSI calculation with unit tests (Wilder's smoothing method)
  - ✅ API endpoint extended for RSI
  - ✅ Frontend RSI checkbox toggle
  - ✅ Separate RSI panel below main chart (120px height)
  - ✅ 0-100 scale with reference lines at 30 (oversold) and 70 (overbought)
  - ✅ Purple (#9C27B0) color scheme for RSI line

**Frontend Improvements**
- ✅ Improved styling and responsive design
  - ✅ Financial color scheme: #e9eeff page backing, #fefefe content backing, #424242 dark grey, #1a237e navy, #757575 light grey
  - ✅ Typography: Inter font for headers, system fonts for body text
  - ✅ Landing page: Navy background with two-column responsive layout (branding left, login right)
  - ✅ Header component: Navy background with "Trading Simulator" title and dropdown navigation (Dashboard/Markets/About/Logout)
  - ✅ Status bar: Fixed bottom bar showing logged-in user and bot status across all views
- ✅ Enhanced dashboard design
  - ✅ 3-column portfolio grid: (1) Total value & available cash (2) Asset balances list (3) Pie chart composition
  - ✅ Expandable deposit/withdrawal forms below lifetime statistics
  - ✅ Consistent neutral color scheme with box shadows and rounded corners
- ✅ Markets view refinement
  - ✅ Updated to use new color scheme
  - ✅ Removed old navigation (moved to header)
  - ✅ Market cards with neutral colors and consistent styling
- ✅ Trading view refinement
  - ✅ Horizontal market/price display (left/right layout)
  - ✅ Consolidated chart section (time controls, graph, indicators in one white div)
  - ✅ Blue-tinted page background (#e9eeff)
  - ✅ Neutral colors for all forms (portfolio, trade, bot controls)
  - ✅ Reordered: Trade form before portfolio section
  - ✅ Updated table formatting with consistent colors
- ✅ About page
  - ✅ Professional layout with project description, features, and tech stack
---

## Phase 5 – Implement New Bot Strategies

**Bot Strategies**
- [ ] Additional bot strategies (MA crossover, RSI-based, etc.)
- [ ] Bot performance metrics and comparison
- [ ] Real-time bot status updates

## Backlog - Bugs and UI improvements
- [ ] Update off white-color to F2F6FF
- [ ] RSA plot should have brighter white background (like the other graphs) instead of off-white
- [ ] Consolidate trading form and portfolio section in the trading view to be side-by-side, neither need that much horizontal space
- [ ] Remove emojis, except for ones being used as icons such as pause/play
- [ ] Either hide indicator overlay selection for candlestick graphs or fix it (does nothing on the candlestick graphs)
- [ ] Momentum bot is named "Naive Momentum Bot" resulting in this "Status: Naive Momentum Bot Bot running in BTC/USD", should rename
- [ ] Indicator div on trading view has a visible border, we should remove this

## Backlog - Features
- [ ] (lower priority) Technical indicators (MACD) - feature only enabled for 1h (5-second) window
- [ ] Additional trading pairs (SOL, DOGE, etc.)
- [ ] Export trade history (CSV, JSON)
- [ ] Advanced portfolio analytics (P&L, Sharpe ratio)
- [ ] WebSocket support for live updates
- [ ] Live price streaming
