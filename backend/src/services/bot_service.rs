use crate::bots::{BotContext, BotDecision, TradingBot};
use crate::models::*;
use crate::state::AppState;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Spawn a bot execution task for a user
/// Returns JoinHandle for the spawned task
pub fn spawn_bot_task(
    state: AppState,
    user_id: UserId,
    bot: Box<dyn TradingBot>,
    base_asset: String,
    quote_asset: String,
    stoploss_amount: f64,
    initial_portfolio_value: f64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut bot = bot;
        let mut tick_count = 0u64;
        let mut interval = interval(Duration::from_secs(60)); // 60-second cadence

        tracing::info!(
            "Bot '{}' started for user {} on {}/{} (stoploss: ${:.2})",
            bot.name(),
            user_id,
            base_asset,
            quote_asset,
            stoploss_amount
        );

        loop {
            interval.tick().await;

            // Check if bot was stopped by user
            let bot_exists = {
                let state_lock = state.inner.read().await;
                state_lock.active_bots.contains_key(&user_id)
            };

            if !bot_exists {
                tracing::info!("Bot stopped by user for {}", user_id);
                break;
            }

            // Assemble bot context
            let ctx = match assemble_bot_context(
                &state,
                &user_id,
                &base_asset,
                &quote_asset,
                tick_count,
            )
            .await
            {
                Ok(ctx) => ctx,
                Err(e) => {
                    tracing::error!("Failed to assemble bot context: {}", e);
                    stop_bot(&state, &user_id, "context assembly failed").await;
                    break;
                }
            };

            // Call bot's tick method
            let decision = bot.tick(&ctx);

            // Log every tick decision at INFO level for visibility
            tracing::info!(
                "Bot '{}' tick {} @ ${:.2}: {:?}",
                bot.name(),
                tick_count,
                ctx.current_price,
                decision
            );

            // Validate and execute decision
            match execute_bot_decision(
                &state,
                &user_id,
                &decision,
                &base_asset,
                &quote_asset,
                ctx.current_price,
                bot.name(),
            )
            .await
            {
                Ok(ExecutionResult::TradeExecuted) => {
                    tracing::info!(
                        "Bot '{}' executed trade: {:?}",
                        bot.name(),
                        decision
                    );
                }
                Ok(ExecutionResult::NoAction) => {
                    // DoNothing decision, continue
                }
                Ok(ExecutionResult::InsufficientFunds(msg)) => {
                    tracing::warn!("Bot stopped due to insufficient funds: {}", msg);
                    stop_bot(&state, &user_id, "insufficient funds").await;
                    break;
                }
                Err(e) => {
                    tracing::error!("Bot execution error: {}", e);
                    stop_bot(&state, &user_id, &format!("execution error: {}", e)).await;
                    break;
                }
            }

            // Check stoploss after trade execution
            if let Err(reason) = check_stoploss(
                &state,
                &user_id,
                initial_portfolio_value,
                stoploss_amount,
            )
            .await
            {
                tracing::warn!("Bot stopped: {}", reason);
                stop_bot(&state, &user_id, &reason).await;
                break;
            }

            tick_count += 1;
        }

        tracing::info!("Bot '{}' terminated for user {}", bot.name(), user_id);
    })
}

/// Assemble BotContext from current state
async fn assemble_bot_context(
    state: &AppState,
    user_id: &UserId,
    base_asset: &str,
    quote_asset: &str,
    tick_count: u64,
) -> Result<BotContext, String> {
    // Get price window (raw 5s data, last 720 points = 1 hour)
    let price_window = state.get_price_window(base_asset, 720).await;

    if price_window.is_empty() {
        return Err(format!("No price data available for {}", base_asset));
    }

    // Get current price for the trading pair
    let current_price = state
        .get_pair_price(base_asset, quote_asset)
        .await
        .ok_or_else(|| format!("Could not get price for {}/{}", base_asset, quote_asset))?;

    // Get user balances
    let user = state
        .get_user(user_id)
        .await
        .ok_or_else(|| "User not found".to_string())?;

    let base_balance = user.get_balance(base_asset);
    let quote_balance = user.get_balance(quote_asset);

    Ok(BotContext {
        price_window,
        base_balance,
        quote_balance,
        current_price,
        base_asset: base_asset.to_string(),
        quote_asset: quote_asset.to_string(),
        tick_count,
    })
}

enum ExecutionResult {
    TradeExecuted,
    NoAction,
    InsufficientFunds(String),
}

/// Execute bot decision with validation
async fn execute_bot_decision(
    state: &AppState,
    user_id: &UserId,
    decision: &BotDecision,
    base_asset: &str,
    quote_asset: &str,
    current_price: f64,
    bot_name: &str,
) -> Result<ExecutionResult, String> {
    match decision {
        BotDecision::DoNothing => Ok(ExecutionResult::NoAction),

        BotDecision::Buy { quote_amount } => {
            // Convert quote amount to base quantity
            let base_quantity = quote_amount / current_price;

            // Validate sufficient quote balance
            let user = state
                .get_user(user_id)
                .await
                .ok_or_else(|| "User not found".to_string())?;

            let quote_balance = user.get_balance(quote_asset);

            if quote_balance < *quote_amount {
                return Ok(ExecutionResult::InsufficientFunds(format!(
                    "Cannot buy: need ${:.2} but only have ${:.2}",
                    quote_amount, quote_balance
                )));
            }

            // Execute buy trade
            execute_bot_trade(
                state,
                user_id,
                base_asset,
                quote_asset,
                TradeSide::Buy,
                base_quantity,
                current_price,
                bot_name,
            )
            .await?;

            Ok(ExecutionResult::TradeExecuted)
        }

        BotDecision::Sell { quote_amount } => {
            // Convert quote amount to base quantity
            let base_quantity = quote_amount / current_price;

            // Validate sufficient base balance
            let user = state
                .get_user(user_id)
                .await
                .ok_or_else(|| "User not found".to_string())?;

            let base_balance = user.get_balance(base_asset);

            if base_balance < base_quantity {
                // Bot tried to sell more than available - not a hard error, just skip
                // This is expected behavior (e.g., bot starting with 0 BTC in your example)
                tracing::debug!(
                    "Bot tried to sell {:.8} {} but only has {:.8}, skipping",
                    base_quantity,
                    base_asset,
                    base_balance
                );
                return Ok(ExecutionResult::NoAction);
            }

            // Execute sell trade
            execute_bot_trade(
                state,
                user_id,
                base_asset,
                quote_asset,
                TradeSide::Sell,
                base_quantity,
                current_price,
                bot_name,
            )
            .await?;

            Ok(ExecutionResult::TradeExecuted)
        }
    }
}

/// Execute a trade for the bot
async fn execute_bot_trade(
    state: &AppState,
    user_id: &UserId,
    base_asset: &str,
    quote_asset: &str,
    side: TradeSide,
    quantity: f64,
    price: f64,
    bot_name: &str,
) -> Result<(), String> {
    // Get USD snapshots for analytics
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

    // Execute trade via trading service
    crate::services::trading_service::execute_trade_internal(
        state,
        user_id,
        base_asset,
        quote_asset,
        side,
        quantity,
        price,
        base_usd_price,
        quote_usd_price,
        Some(bot_name.to_string()), // Mark as bot-executed
    )
    .await
    .map(|_| ())
    .map_err(|e| format!("{:?}", e))
}

/// Check if stoploss has been breached
async fn check_stoploss(
    state: &AppState,
    user_id: &UserId,
    initial_portfolio_value: f64,
    stoploss_amount: f64,
) -> Result<(), String> {
    let current_portfolio_value = calculate_portfolio_value_usd(state, user_id).await?;
    let loss = initial_portfolio_value - current_portfolio_value;

    if loss >= stoploss_amount {
        Err(format!(
            "Stoploss breached: lost ${:.2} (limit: ${:.2})",
            loss, stoploss_amount
        ))
    } else {
        Ok(())
    }
}

/// Calculate total portfolio value in USD
pub async fn calculate_portfolio_value_usd(
    state: &AppState,
    user_id: &UserId,
) -> Result<f64, String> {
    let user = state
        .get_user(user_id)
        .await
        .ok_or_else(|| "User not found".to_string())?;

    let mut total_usd = 0.0;

    for (asset, balance) in &user.asset_balances {
        if *balance <= 0.0 {
            continue;
        }

        if asset == "USD" {
            total_usd += balance;
        } else {
            // Get USD price for asset
            if let Some(price) = state.get_latest_price(asset).await {
                total_usd += balance * price;
            } else {
                tracing::warn!("Could not get price for {} when calculating portfolio value", asset);
            }
        }
    }

    Ok(total_usd)
}

/// Stop a bot (remove from active_bots map)
async fn stop_bot(state: &AppState, user_id: &UserId, reason: &str) {
    let mut state_lock = state.inner.write().await;
    if let Some(bot_instance) = state_lock.active_bots.remove(user_id) {
        bot_instance.task_handle.abort(); // Abort the task
        tracing::info!(
            "Bot '{}' stopped for user {}: {}",
            bot_instance.bot_name,
            user_id,
            reason
        );
    }
}
