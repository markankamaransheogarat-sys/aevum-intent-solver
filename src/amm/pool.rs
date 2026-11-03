use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PoolType {
    UniswapV2,
    UniswapV3,
    CurveStableswap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AmmError {
    InvalidToken(String),
    InsufficientLiquidity,
    ZeroAmount,
    MathOverflow,
    SlippageExceeded,
}

impl std::fmt::Display for AmmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmmError::InvalidToken(tok) => write!(f, "Invalid token for pool: {}", tok),
            AmmError::InsufficientLiquidity => write!(f, "Insufficient liquidity in AMM pool"),
            AmmError::ZeroAmount => write!(f, "Swap input amount must be greater than zero"),
            AmmError::MathOverflow => write!(f, "Math overflow in AMM invariant calculation"),
            AmmError::SlippageExceeded => write!(f, "Execution slippage tolerance exceeded"),
        }
    }
}

impl std::error::Error for AmmError {}

pub trait AmmPool: Send + Sync + Debug {
    fn pool_id(&self) -> &str;
    fn pool_type(&self) -> PoolType;
    fn token0(&self) -> &str;
    fn token1(&self) -> &str;
    fn fee_rate(&self) -> f64; // e.g. 0.003 for 0.3%

    /// Spot price of token_in expressed in token_out
    fn spot_price(&self, token_in: &str) -> Result<f64, AmmError>;

    /// Calculate exact amount out given amount in (including fee and slippage)
    fn get_amount_out(&self, token_in: &str, amount_in: f64) -> Result<f64, AmmError>;

    /// Mutate state by executing a swap (virtual reserve update)
    fn execute_swap(&mut self, token_in: &str, amount_in: f64) -> Result<f64, AmmError>;

    /// Clone the pool into a boxed trait object
    fn clone_box(&self) -> Box<dyn AmmPool>;
}

impl Clone for Box<dyn AmmPool> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
