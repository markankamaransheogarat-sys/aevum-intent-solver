use super::pool::{AmmError, AmmPool, PoolType};

#[derive(Debug, Clone)]
pub struct UniswapV2Pool {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: f64,
    pub reserve1: f64,
    pub fee: f64, // e.g. 0.003 for 30 bps
}

impl UniswapV2Pool {
    pub fn new(id: &str, token0: &str, token1: &str, reserve0: f64, reserve1: f64, fee: f64) -> Self {
        Self {
            id: id.to_string(),
            token0: token0.to_string(),
            token1: token1.to_string(),
            reserve0,
            reserve1,
            fee,
        }
    }
}

impl AmmPool for UniswapV2Pool {
    fn pool_id(&self) -> &str {
        &self.id
    }

    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV2
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
        if token_in == self.token0 {
            if self.reserve0 <= 0.0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(self.reserve1 / self.reserve0)
        } else if token_in == self.token1 {
            if self.reserve1 <= 0.0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(self.reserve0 / self.reserve1)
        } else {
            Err(AmmError::InvalidToken(token_in.to_string()))
        }
    }

    fn get_amount_out(&self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        if amount_in <= 0.0 {
            return Err(AmmError::ZeroAmount);
        }

        let (reserve_in, reserve_out) = if token_in == self.token0 {
            (self.reserve0, self.reserve1)
        } else if token_in == self.token1 {
            (self.reserve1, self.reserve0)
        } else {
            return Err(AmmError::InvalidToken(token_in.to_string()));
        };

        if reserve_in <= 0.0 || reserve_out <= 0.0 {
            return Err(AmmError::InsufficientLiquidity);
        }

        let amount_in_with_fee = amount_in * (1.0 - self.fee);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;

        let amount_out = numerator / denominator;
        if amount_out >= reserve_out {
            return Err(AmmError::InsufficientLiquidity);
        }

        Ok(amount_out)
    }

    fn execute_swap(&mut self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        let amount_out = self.get_amount_out(token_in, amount_in)?;
        if token_in == self.token0 {
            self.reserve0 += amount_in;
            self.reserve1 -= amount_out;
        } else {
            self.reserve1 += amount_in;
            self.reserve0 -= amount_out;
        }
        Ok(amount_out)
    }

    fn clone_box(&self) -> Box<dyn AmmPool> {
        Box::new(self.clone())
    }
}
