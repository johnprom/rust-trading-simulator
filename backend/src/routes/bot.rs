use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::bots::naive_momentum::NaiveMomentumBot;
use crate::models::UserId;
use crate::services::bot_service::{calculate_portfolio_value_usd, spawn_bot_task};
use crate::state::{AppState, BotInstance};

#[derive(Debug, Deserialize)]
pub struct StartBotRequest {
    pub user_id: UserId,
    pub bot_name: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub stoploss_amount: f64,
}

#[derive(Debug, Serialize)]
pub struct StartBotResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct BotStatusResponse {
    pub is_active: bool,
    pub bot_name: Option<String>,
    pub trading_pair: Option<String>,
    pub stoploss_amount: Option<f64>,
    pub initial_portfolio_value: Option<f64>,
}

/// Start a bot for a user
pub async fn start_bot(
    State(state): State<AppState>,
    Json(req): Json<StartBotRequest>,
) -> Result<Json<StartBotResponse>, (StatusCode, String)> {
    // Validate stoploss amount
    if req.stoploss_amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Stoploss amount must be positive".to_string(),
        ));
    }

    // Check if user already has an active bot
    {
        let state_lock = state.inner.read().await;
        if state_lock.active_bots.contains_key(&req.user_id) {
            return Err((
                StatusCode::CONFLICT,
                "User already has an active bot running".to_string(),
            ));
        }
    }

    // Verify user exists
    if state.get_user(&req.user_id).await.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    // Calculate initial portfolio value for stoploss tracking
    let initial_portfolio_value = calculate_portfolio_value_usd(&state, &req.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create bot instance based on bot_name
    let bot: Box<dyn crate::bots::TradingBot> = match req.bot_name.as_str() {
        "naive_momentum" => Box::new(NaiveMomentumBot::new(req.stoploss_amount)),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown bot: {}", req.bot_name),
            ))
        }
    };

    let bot_display_name = bot.name().to_string();

    // Spawn bot task
    let task_handle = spawn_bot_task(
        state.clone(),
        req.user_id.clone(),
        bot,
        req.base_asset.clone(),
        req.quote_asset.clone(),
        req.stoploss_amount,
        initial_portfolio_value,
    );

    // Store bot instance in state
    {
        let mut state_lock = state.inner.write().await;
        state_lock.active_bots.insert(
            req.user_id.clone(),
            BotInstance {
                bot_name: bot_display_name.clone(),
                trading_pair: (req.base_asset.clone(), req.quote_asset.clone()),
                stoploss_amount: req.stoploss_amount,
                initial_portfolio_value_usd: initial_portfolio_value,
                task_handle,
            },
        );
    }

    Ok(Json(StartBotResponse {
        success: true,
        message: format!(
            "Bot '{}' started on {}/{} with ${:.2} stoploss",
            bot_display_name, req.base_asset, req.quote_asset, req.stoploss_amount
        ),
    }))
}

/// Stop a bot for a user
pub async fn stop_bot(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<StartBotResponse>, (StatusCode, String)> {
    let user_id = params
        .get("user_id")
        .ok_or((StatusCode::BAD_REQUEST, "Missing user_id parameter".to_string()))?;

    // Remove bot from active_bots (this signals the task to stop)
    let bot_instance = {
        let mut state_lock = state.inner.write().await;
        state_lock.active_bots.remove(user_id)
    };

    match bot_instance {
        Some(instance) => {
            instance.task_handle.abort(); // Force abort the task
            Ok(Json(StartBotResponse {
                success: true,
                message: format!("Bot '{}' stopped", instance.bot_name),
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            "No active bot for this user".to_string(),
        )),
    }
}

/// Get bot status for a user
pub async fn bot_status(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<BotStatusResponse>, (StatusCode, String)> {
    let user_id = params
        .get("user_id")
        .ok_or((StatusCode::BAD_REQUEST, "Missing user_id parameter".to_string()))?;

    let state_lock = state.inner.read().await;

    match state_lock.active_bots.get(user_id) {
        Some(instance) => Ok(Json(BotStatusResponse {
            is_active: true,
            bot_name: Some(instance.bot_name.clone()),
            trading_pair: Some(format!(
                "{}/{}",
                instance.trading_pair.0, instance.trading_pair.1
            )),
            stoploss_amount: Some(instance.stoploss_amount),
            initial_portfolio_value: Some(instance.initial_portfolio_value_usd),
        })),
        None => Ok(Json(BotStatusResponse {
            is_active: false,
            bot_name: None,
            trading_pair: None,
            stoploss_amount: None,
            initial_portfolio_value: None,
        })),
    }
}
