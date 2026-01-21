# Database Persistence Setup

## Overview

The trading simulator now includes SQLite database persistence for user data. This means your balances and portfolio survive container restarts!

## Architecture

### Database Layer
- **Database**: SQLite (locally) → Postgres (AWS deployment)
- **ORM**: `sqlx` with compile-time query verification
- **Migrations**: Located in `backend/migrations/`
- **Location**: `/app/data/trading_sim.db` inside container

### What Gets Persisted
- ✅ User profiles (username, user_id)
- ✅ Cash balances
- ✅ Asset balances (BTC holdings)
- ❌ Price window (remains in-memory only)

### How It Works
1. **On Startup**: Database is initialized, migrations run, users loaded from DB
2. **On Trade**: User changes are saved to in-memory state, then persisted to DB asynchronously
3. **Demo User**: Created automatically if no users exist in database

## Running with Persistence

### Option 1: Named Volume (Recommended)

```powershell
# Windows PowerShell
docker build -t rust-trading-simulator .
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator
```

```bash
# Mac/Linux
docker build -t rust-trading-simulator .
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator
```

**Advantages**:
- Data persists across container restarts
- Docker manages the volume
- Each device has its own independent data

### Option 2: Bind Mount (For Development)

```powershell
# Windows PowerShell
docker run --name sim -p 3000:3000 -v ${PWD}/data:/app/data rust-trading-simulator
```

```bash
# Mac/Linux
docker run --name sim -p 3000:3000 -v $(pwd)/data:/app/data rust-trading-simulator
```

**Advantages**:
- Database file visible in `./data/trading_sim.db`
- Easy to inspect, backup, or delete
- Can use SQLite browser tools

## Volume Management

```powershell
# List all volumes
docker volume ls

# Inspect volume details
docker volume inspect trading_data

# Remove volume (deletes all data!)
docker volume rm trading_data

# Backup database (if using bind mount)
Copy-Item ./data/trading_sim.db ./backups/trading_sim_backup.db
```

## Container Management

```powershell
# Stop container (data persists)
docker stop sim

# Remove container (data persists in volume)
docker rm sim

# Restart with same data
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator

# Start fresh (delete volume first)
docker volume rm trading_data
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator
```

## Multi-Device Setup

Each device maintains its own database:

**Windows Desktop**:
```powershell
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator
# Data stored in: Docker Desktop's Windows storage
```

**Mac Laptop**:
```bash
docker run --name sim -p 3000:3000 -v trading_data:/app/data rust-trading-simulator
# Data stored in: Docker's Mac storage (completely separate)
```

Both use the same volume name (`trading_data`) but data is device-specific.

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    user_id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    cash_balance REAL NOT NULL DEFAULT 10000.0,
    asset_balances TEXT NOT NULL DEFAULT '{}',  -- JSON: {"BTC": 0.5, ...}
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Future: AWS Deployment

When deploying to AWS, migration is simple:

1. **Change environment variable**:
   ```bash
   DATABASE_URL=postgresql://user:pass@rds-endpoint.aws.com/trading_sim
   ```

2. **Code stays the same** - `sqlx` handles both SQLite and Postgres

3. **No volumes needed** - RDS provides centralized persistence

## Troubleshooting

### Database locked error
- Only one container can access SQLite at a time
- Stop other containers: `docker ps` → `docker stop <container>`

### Lost data after restart
- Ensure you're using the `-v` flag with volume name
- Check volume exists: `docker volume ls`

### Migration errors
- Delete volume and restart: `docker volume rm trading_data`
- Check migrations folder is copied: `docker exec sim ls /app/migrations`

### Can't see database file
- If using named volume, it's in Docker's internal storage
- Use bind mount instead: `-v ${PWD}/data:/app/data`
