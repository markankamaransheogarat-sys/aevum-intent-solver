use serde::{Deserialize, Serialize};
use crate::solver::RouteExecutionResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashbotsBundle {
    pub bundle_id: String,
    pub target_block: u64,
    pub transaction_hashes: Vec<String>,
    pub miner_bribe_eth: f64,
    pub searcher_profit_eth: f64,
    pub bribe_ratio_percent: f64,
    pub is_backrun: bool,
}

impl FlashbotsBundle {
    pub fn build(
        target_block: u64,
        victim_tx_hash: Option<&str>,
        execution: &RouteExecutionResult,
        bribe_ratio: f64, // e.g. 0.85 (85% of net profit goes to builder/validator)
    ) -> Self {
        let net_profit = execution.net_profit.max(0.0);
        let miner_bribe = net_profit * bribe_ratio;
        let searcher_profit = net_profit - miner_bribe;

        let mut tx_hashes = Vec::new();
        let is_backrun = victim_tx_hash.is_some();
        if let Some(vh) = victim_tx_hash {
            tx_hashes.push(vh.to_string());
        }
        let pool_suffix = if execution.hops[0].pool_id.len() >= 6 {
            &execution.hops[0].pool_id[..6]
        } else {
            &execution.hops[0].pool_id
        };
        tx_hashes.push(format!("0xsearcher_arb_{}", pool_suffix));

        Self {
            bundle_id: format!("bundle_{}_{}", target_block, tx_hashes.len()),
            target_block,
            transaction_hashes: tx_hashes,
            miner_bribe_eth: miner_bribe,
            searcher_profit_eth: searcher_profit,
            bribe_ratio_percent: bribe_ratio * 100.0,
            is_backrun,
        }
    }
}
