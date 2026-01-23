use crate::models::*;
use crate::state::AppState;

#[derive(Debug)]
pub enum TradeError {
    InsufficientFunds,
    InsufficientAssets,
    InvalidQuantity,
    UserNotFound,
    PriceUnavailable,
}

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

    let quote_cost = price * quantity;

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
        base_asset: base_asset.to_string(),
        quote_asset: quote_asset.to_string(),
        side: side.clone(),
        quantity,
        price,
        timestamp: chrono::Utc::now(),
        base_usd_price,
        quote_usd_price,
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
