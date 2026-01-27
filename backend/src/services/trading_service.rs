use crate::models::*;
use crate::state::AppState;

#[derive(Debug)]
pub enum TradeError {
    InsufficientFunds,
    InsufficientAssets,
    InvalidQuantity,
    UserNotFound,
    PriceUnavailable,
    DepositTooSmall,
    DepositTooLarge,
    WithdrawalExceedsBalance,
}

/// Execute a trade for manual (UI) trades
pub async fn execute_trade(
    state: &AppState,
    user_id: &UserId,
    base_asset: &str,
    quote_asset: &str,
    side: TradeSide,
    quantity: f64,
) -> Result<Trade, TradeError> {
    if quantity <= 0.0 {
        return Err(TradeError::InvalidQuantity);
    }

    // Get pair price (base in terms of quote)
    let price = state
        .get_pair_price(base_asset, quote_asset)
        .await
        .ok_or(TradeError::PriceUnavailable)?;

    // Capture USD prices at trade time for analytics
    let base_usd_price = if base_asset == "USD" {
        Some(1.0)
    } else {
        state.get_latest_price(base_asset).await
    };

    let quote_usd_price = if quote_asset == "USD" {
        Some(1.0)
    } else {
        state.get_latest_price(quote_asset).await
    };

    execute_trade_internal(
        state,
        user_id,
        base_asset,
        quote_asset,
        side,
        quantity,
        price,
        base_usd_price,
        quote_usd_price,
        None, // No bot name for manual trades
    )
    .await
}

/// Internal trade execution with full control (used by bots)
pub(crate) async fn execute_trade_internal(
    state: &AppState,
    user_id: &UserId,
    base_asset: &str,
    quote_asset: &str,
    side: TradeSide,
    quantity: f64,
    price: f64,
    base_usd_price: Option<f64>,
    quote_usd_price: Option<f64>,
    executed_by_bot: Option<String>,
) -> Result<Trade, TradeError> {
    if quantity <= 0.0 {
        return Err(TradeError::InvalidQuantity);
    }

    let quote_cost = price * quantity;

    // Check balances first before attempting the trade
    let user = state.get_user(user_id).await.ok_or(TradeError::UserNotFound)?;

    match side {
        TradeSide::Buy => {
            let quote_balance = user.get_balance(quote_asset);
            if quote_balance < quote_cost {
                return Err(TradeError::InsufficientFunds);
            }
        }
        TradeSide::Sell => {
            let base_balance = user.get_balance(base_asset);
            if base_balance < quantity {
                return Err(TradeError::InsufficientAssets);
            }
        }
    }

    // Create trade record
    let trade = Trade {
        user_id: user_id.clone(),
        transaction_type: TransactionType::Trade,
        base_asset: base_asset.to_string(),
        quote_asset: quote_asset.to_string(),
        side: side.clone(),
        quantity,
        price,
        timestamp: chrono::Utc::now(),
        base_usd_price,
        quote_usd_price,
        executed_by_bot,
    };

    // Execute the trade and record it in history
    state
        .update_user(user_id, |user| {
            match side {
                TradeSide::Buy => {
                    // Deduct quote asset
                    *user.asset_balances.entry(quote_asset.to_string()).or_insert(0.0) -= quote_cost;
                    // Add base asset
                    *user.asset_balances.entry(base_asset.to_string()).or_insert(0.0) += quantity;
                }
                TradeSide::Sell => {
                    // Deduct base asset
                    *user.asset_balances.entry(base_asset.to_string()).or_insert(0.0) -= quantity;
                    // Add quote asset
                    *user.asset_balances.entry(quote_asset.to_string()).or_insert(0.0) += quote_cost;
                }
            }
            // Add trade to history
            user.trade_history.push(trade.clone());
        })
        .await
        .map_err(|_| TradeError::UserNotFound)?;

    Ok(trade)
}

pub async fn deposit(
    state: &AppState,
    user_id: &UserId,
    amount: f64,
) -> Result<Trade, TradeError> {
    // Validate deposit amount
    if amount < 10.0 {
        return Err(TradeError::DepositTooSmall);
    }
    if amount > 100000.0 {
        return Err(TradeError::DepositTooLarge);
    }

    let transaction = Trade {
        user_id: user_id.clone(),
        transaction_type: TransactionType::Deposit,
        base_asset: "USD".to_string(),
        quote_asset: "USD".to_string(),
        side: TradeSide::Buy,  // Semantically "buying" USD
        quantity: amount,
        price: 1.0,
        timestamp: chrono::Utc::now(),
        base_usd_price: Some(1.0),
        quote_usd_price: Some(1.0),
        executed_by_bot: None,
    };

    // Add USD to balance and record transaction
    state
        .update_user(user_id, |user| {
            *user.asset_balances.entry("USD".to_string()).or_insert(0.0) += amount;
            user.trade_history.push(transaction.clone());
        })
        .await
        .map_err(|_| TradeError::UserNotFound)?;

    Ok(transaction)
}

pub async fn withdraw(
    state: &AppState,
    user_id: &UserId,
    amount: f64,
) -> Result<Trade, TradeError> {
    // Validate withdrawal amount
    if amount <= 0.0 {
        return Err(TradeError::InvalidQuantity);
    }

    // Check sufficient balance
    let user = state.get_user(user_id).await.ok_or(TradeError::UserNotFound)?;
    let usd_balance = user.get_balance("USD");

    if amount > usd_balance {
        return Err(TradeError::WithdrawalExceedsBalance);
    }

    let transaction = Trade {
        user_id: user_id.clone(),
        transaction_type: TransactionType::Withdrawal,
        base_asset: "USD".to_string(),
        quote_asset: "USD".to_string(),
        side: TradeSide::Sell,  // Semantically "selling" USD
        quantity: amount,
        price: 1.0,
        timestamp: chrono::Utc::now(),
        base_usd_price: Some(1.0),
        quote_usd_price: Some(1.0),
        executed_by_bot: None,
    };

    // Deduct USD from balance and record transaction
    state
        .update_user(user_id, |user| {
            *user.asset_balances.entry("USD".to_string()).or_insert(0.0) -= amount;
            user.trade_history.push(transaction.clone());
        })
        .await
        .map_err(|_| TradeError::UserNotFound)?;

    Ok(transaction)
}
