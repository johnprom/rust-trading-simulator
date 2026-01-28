/// Simple Moving Average (SMA)
/// Calculates the arithmetic mean of the last N prices
pub struct SMA {
    period: usize,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    /// Calculate SMA for a price series
    /// Returns a vector of the same length as input
    /// First (period - 1) values will be NaN (warmup period)
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        let mut result = vec![f64::NAN; prices.len()];

        if prices.len() < self.period {
            return result;
        }

        // Calculate SMA for each valid window
        for i in (self.period - 1)..prices.len() {
            let window_start = i + 1 - self.period;
            let window = &prices[window_start..=i];
            let sum: f64 = window.iter().sum();
            result[i] = sum / self.period as f64;
        }

        result
    }
}

/// Exponential Moving Average (EMA)
/// Gives more weight to recent prices using exponential smoothing
pub struct EMA {
    period: usize,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        Self { period }
    }

    /// Smoothing factor (k) for EMA calculation
    /// k = 2 / (period + 1)
    fn smoothing_factor(&self) -> f64 {
        2.0 / (self.period as f64 + 1.0)
    }

    /// Calculate EMA for a price series
    /// Returns a vector of the same length as input
    /// First (period - 1) values will be NaN (warmup period)
    /// First EMA value uses SMA as seed
    pub fn calculate(&self, prices: &[f64]) -> Vec<f64> {
        let mut result = vec![f64::NAN; prices.len()];

        if prices.len() < self.period {
            return result;
        }

        let k = self.smoothing_factor();

        // First EMA value is the SMA of the first 'period' prices
        let first_window = &prices[0..self.period];
        let first_sma: f64 = first_window.iter().sum::<f64>() / self.period as f64;
        result[self.period - 1] = first_sma;

        // Calculate EMA for remaining values
        // EMA(t) = Price(t) * k + EMA(t-1) * (1 - k)
        for i in self.period..prices.len() {
            let price = prices[i];
            let prev_ema = result[i - 1];
            result[i] = price * k + prev_ema * (1.0 - k);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basic() {
        let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];
        let sma = SMA::new(3);
        let result = sma.calculate(&prices);

        // First 2 values should be NaN
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());

        // Third value should be average of first 3: (100 + 102 + 101) / 3 = 101.0
        assert!((result[2] - 101.0).abs() < 0.001);

        // Fourth value: (102 + 101 + 103) / 3 = 102.0
        assert!((result[3] - 102.0).abs() < 0.001);

        // Fifth value: (101 + 103 + 105) / 3 = 103.0
        assert!((result[4] - 103.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_insufficient_data() {
        let prices = vec![100.0, 102.0];
        let sma = SMA::new(3);
        let result = sma.calculate(&prices);

        // All values should be NaN
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());
    }

    #[test]
    fn test_sma_period_20() {
        // Create 25 prices
        let mut prices = Vec::new();
        for i in 0..25 {
            prices.push(100.0 + i as f64);
        }

        let sma = SMA::new(20);
        let result = sma.calculate(&prices);

        // First 19 values should be NaN
        for i in 0..19 {
            assert!(result[i].is_nan());
        }

        // 20th value (index 19) should be average of first 20 prices
        // (100 + 101 + ... + 119) / 20 = 109.5
        assert!((result[19] - 109.5).abs() < 0.001);

        // 21st value (index 20) should be average of prices 1-20
        // (101 + 102 + ... + 120) / 20 = 110.5
        assert!((result[20] - 110.5).abs() < 0.001);
    }

    #[test]
    fn test_ema_basic() {
        let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0, 106.0];
        let ema = EMA::new(3);
        let result = ema.calculate(&prices);

        // First 2 values should be NaN
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());

        // Third value should be SMA of first 3: (100 + 102 + 101) / 3 = 101.0
        assert!((result[2] - 101.0).abs() < 0.001);

        // k = 2 / (3 + 1) = 0.5
        // Fourth value: 103 * 0.5 + 101.0 * 0.5 = 102.0
        assert!((result[3] - 102.0).abs() < 0.001);

        // Fifth value: 105 * 0.5 + 102.0 * 0.5 = 103.5
        assert!((result[4] - 103.5).abs() < 0.001);
    }

    #[test]
    fn test_ema_period_12() {
        // Create 20 prices
        let mut prices = Vec::new();
        for i in 0..20 {
            prices.push(100.0 + i as f64);
        }

        let ema = EMA::new(12);
        let result = ema.calculate(&prices);

        // First 11 values should be NaN
        for i in 0..11 {
            assert!(result[i].is_nan());
        }

        // 12th value (index 11) should be SMA of first 12
        // (100 + 101 + ... + 111) / 12 = 105.5
        assert!((result[11] - 105.5).abs() < 0.001);

        // k = 2 / (12 + 1) = 0.1538...
        // 13th value: 112 * k + 105.5 * (1 - k)
        let k = 2.0 / 13.0;
        let expected = 112.0 * k + 105.5 * (1.0 - k);
        assert!((result[12] - expected).abs() < 0.001);
    }

    #[test]
    fn test_ema_smoothing_factor() {
        let ema = EMA::new(12);
        let k = ema.smoothing_factor();
        assert!((k - 2.0 / 13.0).abs() < 0.0001);

        let ema26 = EMA::new(26);
        let k26 = ema26.smoothing_factor();
        assert!((k26 - 2.0 / 27.0).abs() < 0.0001);
    }
}
