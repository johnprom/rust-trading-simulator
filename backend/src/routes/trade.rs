use crate::{models::*, services::trading_service::{self, TradeError}, state::AppState};
use axum::{extract::{State, Query}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TradeRequest {
    pub asset: String,           // base_asset for backward compatibility
    #[serde(default)]
    pub quote_asset: Option<String>,  // Optional, defaults to "USD"
    pub side: TradeSide,
    pub quantity: f64,
}

#[derive(Deserialize)]
pub struct DepositRequest {
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct WithdrawalRequest {
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct TradeQuery {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct TradeErrorResponse {
    pub error: String,
}

pub async fn post_trade(
    State(state): State<AppState>,
    Query(query): Query<TradeQuery>,
    Json(req): Json<TradeRequest>,
) -> Result<Json<Trade>, (StatusCode, Json<TradeErrorResponse>)> {
    let base_asset = &req.asset;
    let quote_asset = req.quote_asset.as_deref().unwrap_or("USD");

    match trading_service::execute_trade(
        &state,
        &query.user_id,
        base_asset,
        quote_asset,
        req.side,
        req.quantity,
    )
    .await
    {
        Ok(trade) => Ok(Json(trade)),
        Err(err) => {
            let error_msg = match err {
                TradeError::InsufficientFunds => format!("Insufficient {} to complete this purchase", quote_asset),
                TradeError::InsufficientAssets => format!("Insufficient {} to complete this sale", base_asset),
                TradeError::InvalidQuantity => "Invalid quantity specified".to_string(),
                TradeError::UserNotFound => "User not found".to_string(),
                TradeError::PriceUnavailable => "Price unavailable for this trading pair".to_string(),
                TradeError::DepositTooSmall => "Deposit must be at least $10".to_string(),
                TradeError::DepositTooLarge => "Deposit cannot exceed $100,000".to_string(),
                TradeError::WithdrawalExceedsBalance => "Insufficient balance for withdrawal".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(TradeErrorResponse { error: error_msg }),
            ))
        }
    }
}

pub async fn post_deposit(
    State(state): State<AppState>,
    Query(query): Query<TradeQuery>,
    Json(req): Json<DepositRequest>,
) -> Result<Json<Trade>, (StatusCode, Json<TradeErrorResponse>)> {
    match trading_service::deposit(&state, &query.user_id, req.amount).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(err) => {
            let error_msg = match err {
                TradeError::DepositTooSmall => "Deposit must be at least $10".to_string(),
                TradeError::DepositTooLarge => "Deposit cannot exceed $100,000".to_string(),
                TradeError::UserNotFound => "User not found".to_string(),
                _ => "Deposit failed".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(TradeErrorResponse { error: error_msg }),
            ))
        }
    }
}

pub async fn post_withdrawal(
    State(state): State<AppState>,
    Query(query): Query<TradeQuery>,
    Json(req): Json<WithdrawalRequest>,
) -> Result<Json<Trade>, (StatusCode, Json<TradeErrorResponse>)> {
    match trading_service::withdraw(&state, &query.user_id, req.amount).await {
        Ok(transaction) => Ok(Json(transaction)),
        Err(err) => {
            let error_msg = match err {
                TradeError::WithdrawalExceedsBalance => "Insufficient balance for withdrawal".to_string(),
                TradeError::InvalidQuantity => "Invalid withdrawal amount".to_string(),
                TradeError::UserNotFound => "User not found".to_string(),
                _ => "Withdrawal failed".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(TradeErrorResponse { error: error_msg }),
            ))
        }
    }
}
