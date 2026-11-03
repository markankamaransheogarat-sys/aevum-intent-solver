use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapIntent {
    pub tx_hash: String,
    pub pool_id: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: f64,
    pub min_amount_out: f64,
    pub max_priority_fee_per_gas: f64, // Gwei
    pub gas_limit: u64,
}

impl SwapIntent {
    pub fn new(
        tx_hash: &str,
        pool_id: &str,
        token_in: &str,
        token_out: &str,
        amount_in: f64,
        min_amount_out: f64,
    ) -> Self {
        Self {
            tx_hash: tx_hash.to_string(),
            pool_id: pool_id.to_string(),
            token_in: token_in.to_string(),
            token_out: token_out.to_string(),
            amount_in,
            min_amount_out,
            max_priority_fee_per_gas: 2.0,
            gas_limit: 150_000,
        }
    }
}
