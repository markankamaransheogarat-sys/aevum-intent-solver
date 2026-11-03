use super::pool::{AmmError, AmmPool, PoolType};

/// Curve Stableswap 2-token AMM Invariant Pool
/// Formula: A * n^n * sum(x) + D = A * D * n^n + D^(n+1) / (n^n * prod(x))
#[derive(Debug, Clone)]
pub struct CurvePool {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub balance0: f64,
    pub balance1: f64,
    pub amplification_a: f64, // Amplification coefficient A (e.g. 100-2000)
    pub fee: f64,             // e.g. 0.0004 (4 bps)
}

impl CurvePool {
    pub fn new(id: &str, token0: &str, token1: &str, balance0: f64, balance1: f64, a: f64, fee: f64) -> Self {
        Self {
            id: id.to_string(),
            token0: token0.to_string(),
            token1: token1.to_string(),
            balance0,
            balance1,
            amplification_a: a,
            fee,
        }
    }

    /// Calculate invariant D using Newton-Raphson approximation
    pub fn calculate_invariant_d(&self) -> Result<f64, AmmError> {
        let x = self.balance0;
        let y = self.balance1;
        let s = x + y;
        if s <= 0.0 {
            return Ok(0.0);
        }

        let mut d = s;
        let ann = self.amplification_a * 4.0; // A * n^n for n=2

        for _ in 0..64 {
            // D_P = D^3 / (4 * x * y)
            let d_p = (d * d * d) / (4.0 * x * y);
            let prev_d = d;

            // d = (Ann * S + 2 * D_P) * d / ((Ann - 1) * d + 3 * D_P)
            let num = (ann * s + 2.0 * d_p) * d;
            let den = (ann - 1.0) * d + 3.0 * d_p;
            d = num / den;

            if (d - prev_d).abs() <= 1e-6 {
                return Ok(d);
            }
        }

        Ok(d)
    }

    /// Calculate y given new x and invariant D
    pub fn calculate_y(&self, new_x: f64, d: f64) -> Result<f64, AmmError> {
        let ann = self.amplification_a * 4.0;
        let c = (d * d * d) / (4.0 * new_x * ann);
        let b = new_x + (d / ann);

        let mut y = d;
        for _ in 0..64 {
            let prev_y = y;
            // y = (y^2 + c) / (2y + b - d)
            y = (y * y + c) / (2.0 * y + b - d);
            if (y - prev_y).abs() <= 1e-6 {
                return Ok(y);
            }
        }

        Ok(y)
    }
}

impl AmmPool for CurvePool {
    fn pool_id(&self) -> &str {
        &self.id
    }

    fn pool_type(&self) -> PoolType {
        PoolType::CurveStableswap
    }

    fn token0(&self) -> &str {
        &self.token0
    }

    fn token1(&self) -> &str {
        &self.token1
    }

    fn fee_rate(&self) -> f64 {
        self.fee
    }

    fn spot_price(&self, token_in: &str) -> Result<f64, AmmError> {
        // Curve near-peg spot price approximation
        if token_in == self.token0 {
            let small_in = 1.0;
            let out = self.get_amount_out(token_in, small_in)?;
            Ok(out / (small_in * (1.0 - self.fee)))
        } else if token_in == self.token1 {
            let small_in = 1.0;
            let out = self.get_amount_out(token_in, small_in)?;
            Ok(out / (small_in * (1.0 - self.fee)))
        } else {
            Err(AmmError::InvalidToken(token_in.to_string()))
        }
    }

    fn get_amount_out(&self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        if amount_in <= 0.0 {
            return Err(AmmError::ZeroAmount);
        }

        let d = self.calculate_invariant_d()?;
        let amount_in_net = amount_in * (1.0 - self.fee);

        if token_in == self.token0 {
            let new_x = self.balance0 + amount_in_net;
            let new_y = self.calculate_y(new_x, d)?;
            let dy = self.balance1 - new_y;
            if dy <= 0.0 || dy >= self.balance1 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(dy)
        } else if token_in == self.token1 {
            let new_y = self.balance1 + amount_in_net;
            let new_x = self.calculate_y(new_y, d)?;
            let dx = self.balance0 - new_x;
            if dx <= 0.0 || dx >= self.balance0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(dx)
        } else {
            Err(AmmError::InvalidToken(token_in.to_string()))
        }
    }

    fn execute_swap(&mut self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        let amount_out = self.get_amount_out(token_in, amount_in)?;
        if token_in == self.token0 {
            self.balance0 += amount_in;
            self.balance1 -= amount_out;
        } else {
            self.balance1 += amount_in;
            self.balance0 -= amount_out;
        }
        Ok(amount_out)
    }

    fn clone_box(&self) -> Box<dyn AmmPool> {
        Box::new(self.clone())
    }
}
