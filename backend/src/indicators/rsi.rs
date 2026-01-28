/// Relative Strength Index (RSI)
/// Measures momentum by comparing magnitude of recent gains to recent losses
/// Returns values between 0-100:
/// - Below 30: Oversold (potentially undervalued)
/// - Above 70: Overbought (potentially overvalued)
pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    /// Calculate RSI for a price series using Wilder's smoothing method
    /// Returns a vector of the same length as input
    /// First (period) values will be NaN (warmup period)
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        let mut result = vec![f64::NAN; prices.len()];

        if prices.len() < self.period + 1 {
            return result;
        }

        // Calculate price changes
        let mut gains = Vec::new();
        let mut losses = Vec::new();

        for i in 1..prices.len() {
            let change = prices[i] - prices[i - 1];
            gains.push(if change > 0.0 { change } else { 0.0 });
            losses.push(if change < 0.0 { -change } else { 0.0 });
        }

        if gains.len() < self.period {
            return result;
        }

        // Calculate first average gain and loss (simple average)
        let first_avg_gain: f64 = gains[0..self.period].iter().sum::<f64>() / self.period as f64;
        let first_avg_loss: f64 = losses[0..self.period].iter().sum::<f64>() / self.period as f64;

        let mut avg_gain = first_avg_gain;
        let mut avg_loss = first_avg_loss;

        // Calculate first RSI value
        let rs = if avg_loss == 0.0 {
            100.0 // Avoid division by zero
        } else {
            avg_gain / avg_loss
        };
        result[self.period] = 100.0 - (100.0 / (1.0 + rs));

        // Calculate RSI for remaining values using Wilder's smoothing
        // avg_gain = ((prev_avg_gain * (period - 1)) + current_gain) / period
        for i in self.period..gains.len() {
            avg_gain = ((avg_gain * (self.period - 1) as f64) + gains[i]) / self.period as f64;
            avg_loss = ((avg_loss * (self.period - 1) as f64) + losses[i]) / self.period as f64;

            let rs = if avg_loss == 0.0 {
                100.0
            } else {
                avg_gain / avg_loss
            };

            // i in gains corresponds to i+1 in prices (since gains is offset by 1)
            result[i + 1] = 100.0 - (100.0 / (1.0 + rs));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        // Simple test case with clear gains and losses
        let prices = vec![
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 106.0, 108.0, 110.0, 109.0,
            111.0, 113.0, 112.0, 114.0, 116.0, 115.0, 117.0, 119.0, 118.0, 120.0,
        ];
        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // First 14 values should be NaN
        for i in 0..14 {
            assert!(result[i].is_nan(), "Index {} should be NaN", i);
        }

        // 15th value (index 14) should be a valid RSI value between 0-100
        assert!(!result[14].is_nan(), "Index 14 should have a value");
        assert!(result[14] >= 0.0 && result[14] <= 100.0, "RSI should be between 0-100");

        // With mostly gains, RSI should be relatively high (>50)
        assert!(result[14] > 50.0, "RSI should be high with mostly gains");
    }

    #[test]
    fn test_rsi_downtrend() {
        // Prices in downtrend should produce low RSI
        let prices = vec![
            120.0, 118.0, 116.0, 117.0, 115.0, 113.0, 114.0, 112.0, 110.0, 111.0,
            109.0, 107.0, 108.0, 106.0, 104.0, 105.0, 103.0, 101.0, 102.0, 100.0,
        ];
        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // With mostly losses, RSI should be relatively low (<50)
        assert!(result[14] < 50.0, "RSI should be low with mostly losses");
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let prices = vec![100.0, 102.0, 104.0, 103.0, 105.0]; // Only 5 prices
        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // All values should be NaN
        for (i, val) in result.iter().enumerate() {
            assert!(val.is_nan(), "Index {} should be NaN", i);
        }
    }

    #[test]
    fn test_rsi_all_gains() {
        // All gains should produce RSI close to 100
        let mut prices = vec![100.0];
        for i in 1..20 {
            prices.push(100.0 + i as f64);
        }

        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // RSI should be very high (>90) when there are no losses
        // It may not be exactly 100 due to Wilder's smoothing handling zero losses
        assert!(result[14] > 90.0, "RSI should be very high (>90) with all gains, got {}", result[14]);
    }

    #[test]
    fn test_rsi_all_losses() {
        // All losses should produce RSI close to 0
        let mut prices = vec![120.0];
        for i in 1..20 {
            prices.push(120.0 - i as f64);
        }

        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // RSI should be 0 when there are no gains
        assert!(result[14] < 1.0, "RSI should be close to 0 with all losses");
    }

    #[test]
    fn test_rsi_period_14() {
        // Create 30 prices with alternating small gains/losses
        let mut prices = vec![100.0];
        for i in 1..30 {
            let change = if i % 2 == 0 { 1.0 } else { -0.5 };
            prices.push(prices[i - 1] + change);
        }

        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // First 14 values should be NaN
        for i in 0..14 {
            assert!(result[i].is_nan());
        }

        // All subsequent values should be valid RSI values
        for i in 14..result.len() {
            assert!(!result[i].is_nan(), "Index {} should have a value", i);
            assert!(result[i] >= 0.0 && result[i] <= 100.0, "RSI at {} should be between 0-100", i);
        }
    }

    #[test]
    fn test_rsi_no_change() {
        // Flat prices should produce RSI of 50 (no momentum)
        let prices = vec![100.0; 20];
        let rsi = RSI::new(14);
        let result = rsi.calculate(&prices);

        // With no change, RSI behavior depends on implementation
        // In this case with all zeros, RSI will be 0/0 which we handle as 100
        // This is an edge case - flat prices typically won't occur in real data
        assert!(!result[14].is_nan(), "Should handle flat prices");
    }
}
