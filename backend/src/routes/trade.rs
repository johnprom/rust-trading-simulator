use crate::{models::*, services::trading_service::{self, TradeError}, state::AppState};
use axum::{extract::{State, Query}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TradeRequest {
    pub asset: String,
    pub side: TradeSide,
    pub quantity: f64,
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
    match trading_service::execute_trade(
        &state,
        &query.user_id,
        &req.asset,
        req.side,
        req.quantity,
    )
    .await
    {
        Ok(trade) => Ok(Json(trade)),
        Err(err) => {
            let error_msg = match err {
                TradeError::InsufficientFunds => "Insufficient funds to complete this purchase".to_string(),
                TradeError::InsufficientAssets => "Insufficient BTC to complete this sale".to_string(),
                TradeError::InvalidQuantity => "Invalid quantity specified".to_string(),
                TradeError::UserNotFound => "User not found".to_string(),
            };
            Err((
                StatusCode::BAD_REQUEST,
                Json(TradeErrorResponse { error: error_msg }),
            ))
        }
    }
}
