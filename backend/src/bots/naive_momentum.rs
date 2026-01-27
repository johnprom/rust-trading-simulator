use super::{BotContext, BotDecision, PriceHistory, TradingBot};

/// Naive momentum bot: Buys on 3 consecutive price increases, sells on 3 consecutive decreases
/// Uses 1% of stoploss as step size, enforces 3-tick cooldown after each trade
pub struct NaiveMomentumBot {
    // Configuration (set at initialization)
    stepsize_quote: f64, // 1% of stoploss amount

    // Internal state (tracked across ticks)
    price_history: PriceHistory,  // Template helper for tracking prices
    cooldown_remaining: u32,      // Cycles to skip after a trade (0 = not in cooldown)

    // Statistics (optional, for debugging/visibility)
    total_buys: u32,
    total_sells: u32,
    last_action: String,
}

impl NaiveMomentumBot {
    /// Create new bot with given stoploss amount
    /// Stepsize is automatically set to 1% of stoploss
    pub fn new(stoploss_amount: f64) -> Self {
        Self {
            stepsize_quote: stoploss_amount * 0.01, // 1% of stoploss
            price_history: PriceHistory::new(10),    // Track last 10 prices (more than we need)
            cooldown_remaining: 0,
            total_buys: 0,
            total_sells: 0,
            last_action: "initialized".to_string(),
        }
    }

    /// Check if last 3 prices show consecutive increases
    fn is_uptrend(&self) -> bool {
        if !self.price_history.has_at_least(3) {
            return false;
        }

        let recent = self.price_history.last_n(3);
        recent[1] > recent[0] && recent[2] > recent[1]
    }

    /// Check if last 3 prices show consecutive decreases
    fn is_downtrend(&self) -> bool {
        if !self.price_history.has_at_least(3) {
            return false;
        }

        let recent = self.price_history.last_n(3);
        recent[1] < recent[0] && recent[2] < recent[1]
    }
}

impl TradingBot for NaiveMomentumBot {
    fn tick(&mut self, ctx: &BotContext) -> BotDecision {
        // Update price history (tick happens every 60s, matches minutely cadence)
        self.price_history.push(ctx.current_price);

        // Handle cooldown period
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining -= 1;
            self.last_action = format!("cooldown ({})", self.cooldown_remaining);
            return BotDecision::DoNothing;
        }

        // Need at least 3 prices to detect trend
        if !self.price_history.has_at_least(3) {
            self.last_action = "warming up".to_string();
            return BotDecision::DoNothing;
        }

        // Check for uptrend -> Buy
        if self.is_uptrend() {
            self.cooldown_remaining = 3;
            self.total_buys += 1;
            self.last_action = format!("buy ${:.2}", self.stepsize_quote);
            return BotDecision::Buy {
                quote_amount: self.stepsize_quote,
            };
        }

        // Check for downtrend -> Sell
        if self.is_downtrend() {
            self.cooldown_remaining = 3;
            self.total_sells += 1;
            self.last_action = format!("sell ${:.2}", self.stepsize_quote);
            return BotDecision::Sell {
                quote_amount: self.stepsize_quote,
            };
        }

        // No clear trend
        self.last_action = "no trend".to_string();
        BotDecision::DoNothing
    }

    fn name(&self) -> &str {
        "Naive Momentum Bot"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PricePoint;
    use chrono::Utc;

    fn create_test_context(prices: Vec<f64>, current_price: f64) -> BotContext {
        let price_window = prices
            .iter()
            .map(|&p| PricePoint {
                timestamp: Utc::now(),
                asset: "BTC".to_string(),
                price: p,
            })
            .collect();

        BotContext {
            price_window,
            base_balance: 0.0,
            quote_balance: 10000.0,
            current_price,
            base_asset: "BTC".to_string(),
            quote_asset: "USD".to_string(),
            tick_count: 0,
        }
    }

    #[test]
    fn test_uptrend_detection() {
        let mut bot = NaiveMomentumBot::new(10000.0); // $10k stoploss, $100 stepsize

        // Feed prices showing uptrend
        let ctx1 = create_test_context(vec![], 100.0);
        bot.tick(&ctx1); // Not enough data

        let ctx2 = create_test_context(vec![], 105.0);
        bot.tick(&ctx2); // Not enough data

        let ctx3 = create_test_context(vec![], 110.0);
        let decision = bot.tick(&ctx3); // Should trigger buy

        assert_eq!(decision, BotDecision::Buy { quote_amount: 100.0 });
        assert_eq!(bot.cooldown_remaining, 3);
    }

    #[test]
    fn test_downtrend_detection() {
        let mut bot = NaiveMomentumBot::new(10000.0);

        // Feed prices showing downtrend
        bot.tick(&create_test_context(vec![], 110.0));
        bot.tick(&create_test_context(vec![], 105.0));
        let decision = bot.tick(&create_test_context(vec![], 100.0));

        assert_eq!(decision, BotDecision::Sell { quote_amount: 100.0 });
        assert_eq!(bot.cooldown_remaining, 3);
    }

    #[test]
    fn test_cooldown_behavior() {
        let mut bot = NaiveMomentumBot::new(10000.0);

        // Trigger buy (uptrend)
        bot.tick(&create_test_context(vec![], 100.0));
        bot.tick(&create_test_context(vec![], 105.0));
        bot.tick(&create_test_context(vec![], 110.0)); // Buy

        // Next 3 ticks should do nothing (cooldown)
        assert_eq!(bot.tick(&create_test_context(vec![], 115.0)), BotDecision::DoNothing);
        assert_eq!(bot.tick(&create_test_context(vec![], 120.0)), BotDecision::DoNothing);
        assert_eq!(bot.tick(&create_test_context(vec![], 125.0)), BotDecision::DoNothing);

        // After cooldown, should be able to trade again
        assert_eq!(bot.cooldown_remaining, 0);
    }

    #[test]
    fn test_no_trend() {
        let mut bot = NaiveMomentumBot::new(10000.0);

        // Feed prices with no clear trend
        bot.tick(&create_test_context(vec![], 100.0));
        bot.tick(&create_test_context(vec![], 105.0));
        let decision = bot.tick(&create_test_context(vec![], 103.0)); // Mixed signal

        assert_eq!(decision, BotDecision::DoNothing);
    }
}
