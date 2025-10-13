use crate::{models::*, services::trading_service, state::AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TradeRequest {
    pub asset: String,
    pub side: TradeSide,
    pub quantity: f64,
}

pub async fn post_trade(
    State(state): State<AppState>,
    Json(req): Json<TradeRequest>,
) -> Result<Json<Trade>, StatusCode> {
    match trading_service::execute_trade(
        &state,
        &"demo_user".to_string(),
        &req.asset,
        req.side,
        req.quantity,
    )
    .await
    {
        Ok(trade) => Ok(Json(trade)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
