use super::pool::{AmmError, AmmPool, PoolType};

/// Simplified Concentrated Liquidity Pool based on Uniswap v3 Tick & SqrtPrice Math
#[derive(Debug, Clone)]
pub struct UniswapV3Pool {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub sqrt_price_x96: f64, // Normalized sqrt(price of token0 in token1)
    pub liquidity: f64,      // Active virtual liquidity L
    pub fee: f64,            // e.g. 0.0005 (5 bps), 0.003 (30 bps), 0.01 (100 bps)
    pub tick_lower_price: f64,
    pub tick_upper_price: f64,
}

impl UniswapV3Pool {
    pub fn new(
        id: &str,
        token0: &str,
        token1: &str,
        current_price: f64,
        liquidity: f64,
        fee: f64,
        tick_lower_price: f64,
        tick_upper_price: f64,
    ) -> Self {
        Self {
            id: id.to_string(),
            token0: token0.to_string(),
            token1: token1.to_string(),
            sqrt_price_x96: current_price.sqrt(),
            liquidity,
            fee,
            tick_lower_price,
            tick_upper_price,
        }
    }

    pub fn current_price(&self) -> f64 {
        self.sqrt_price_x96 * self.sqrt_price_x96
    }
}

impl AmmPool for UniswapV3Pool {
    fn pool_id(&self) -> &str {
        &self.id
    }

    fn pool_type(&self) -> PoolType {
        PoolType::UniswapV3
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
        let p = self.current_price();
        if p <= 0.0 {
            return Err(AmmError::InsufficientLiquidity);
        }
        if token_in == self.token0 {
            Ok(p)
        } else if token_in == self.token1 {
            Ok(1.0 / p)
        } else {
            Err(AmmError::InvalidToken(token_in.to_string()))
        }
    }

    fn get_amount_out(&self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        if amount_in <= 0.0 {
            return Err(AmmError::ZeroAmount);
        }
        if self.liquidity <= 0.0 {
            return Err(AmmError::InsufficientLiquidity);
        }

        let amount_in_net = amount_in * (1.0 - self.fee);

        if token_in == self.token0 {
            // Selling token0 to buy token1: price decreases (sqrt_price drops)
            // Delta(1/sqrt_p) = Delta x / L
            let sqrt_p_cur = self.sqrt_price_x96;
            let inv_sqrt_p_next = (1.0 / sqrt_p_cur) + (amount_in_net / self.liquidity);
            let sqrt_p_next = 1.0 / inv_sqrt_p_next;

            // Delta y = L * (sqrt_p_cur - sqrt_p_next)
            let delta_y = self.liquidity * (sqrt_p_cur - sqrt_p_next);
            if delta_y <= 0.0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(delta_y)
        } else if token_in == self.token1 {
            // Selling token1 to buy token0: price increases (sqrt_price rises)
            // Delta(sqrt_p) = Delta y / L
            let sqrt_p_cur = self.sqrt_price_x96;
            let sqrt_p_next = sqrt_p_cur + (amount_in_net / self.liquidity);

            // Delta x = L * (1/sqrt_p_cur - 1/sqrt_p_next)
            let delta_x = self.liquidity * ((1.0 / sqrt_p_cur) - (1.0 / sqrt_p_next));
            if delta_x <= 0.0 {
                return Err(AmmError::InsufficientLiquidity);
            }
            Ok(delta_x)
        } else {
            Err(AmmError::InvalidToken(token_in.to_string()))
        }
    }

    fn execute_swap(&mut self, token_in: &str, amount_in: f64) -> Result<f64, AmmError> {
        let amount_out = self.get_amount_out(token_in, amount_in)?;
        let amount_in_net = amount_in * (1.0 - self.fee);

        if token_in == self.token0 {
            let inv_sqrt_p_next = (1.0 / self.sqrt_price_x96) + (amount_in_net / self.liquidity);
            self.sqrt_price_x96 = 1.0 / inv_sqrt_p_next;
        } else {
            self.sqrt_price_x96 += amount_in_net / self.liquidity;
        }

        Ok(amount_out)
    }

    fn clone_box(&self) -> Box<dyn AmmPool> {
        Box::new(self.clone())
    }
}
