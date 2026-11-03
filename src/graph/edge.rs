use std::sync::Arc;
use crate::amm::AmmPool;

#[derive(Clone, Debug)]
pub struct PoolEdge {
    pub pool_id: String,
    pub from_token: String,
    pub to_token: String,
    pub pool: Arc<Box<dyn AmmPool>>,
    pub spot_price: f64, // Price of 1 unit of from_token in to_token
    pub fee: f64,
    pub weight: f64,     // -ln(spot_price * (1.0 - fee))
}

impl PoolEdge {
    pub fn new(from_token: &str, to_token: &str, pool: Arc<Box<dyn AmmPool>>) -> Option<Self> {
        let spot = pool.spot_price(from_token).ok()?;
        let fee = pool.fee_rate();
        let net_rate = spot * (1.0 - fee);
        if net_rate <= 0.0 {
            return None;
        }

        let weight = -net_rate.ln();

        Some(Self {
            pool_id: pool.pool_id().to_string(),
            from_token: from_token.to_string(),
            to_token: to_token.to_string(),
            pool,
            spot_price: spot,
            fee,
            weight,
        })
    }
}
