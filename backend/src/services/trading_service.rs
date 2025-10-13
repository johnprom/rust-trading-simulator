use crate::models::*;
use crate::state::AppState;

#[derive(Debug)]
pub enum TradeError {
    InsufficientFunds,
    InsufficientAssets,
    InvalidQuantity,
    UserNotFound,
}

pub async fn execute_trade(
    state: &AppState,
    user_id: &UserId,
    asset: &str,
    side: TradeSide,
    quantity: f64,
) -> Result<Trade, TradeError> {
    if quantity <= 0.0 {
        return Err(TradeError::InvalidQuantity);
    }

    let price = state
        .get_latest_price(asset)
        .await
        .ok_or(TradeError::UserNotFound)?;

    let total_cost = price * quantity;

    state
        .update_user(user_id, |user| {
            match side {
                TradeSide::Buy => {
                    if user.cash_balance < total_cost {
                        return;
                    }
                    user.cash_balance -= total_cost;
                    *user.asset_balances.entry(asset.to_string()).or_insert(0.0) += quantity;
                }
                TradeSide::Sell => {
                    let balance = user.asset_balances.get(asset).copied().unwrap_or(0.0);
                    if balance < quantity {
                        return;
                    }
                    *user.asset_balances.get_mut(asset).unwrap() -= quantity;
                    user.cash_balance += total_cost;
                }
            }
        })
        .await
        .map_err(|_| TradeError::UserNotFound)?;

    Ok(Trade {
        user_id: user_id.clone(),
        asset: asset.to_string(),
        side,
        quantity,
        price,
        timestamp: chrono::Utc::now(),
    })
}
