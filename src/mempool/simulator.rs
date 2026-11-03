use super::bundle::FlashbotsBundle;
use super::intent::SwapIntent;
use crate::graph::{CycleEnumerator, LiquidityGraph};
use crate::solver::{ArbitrageOptimizer, RouteExecutionResult};

pub struct MempoolSimulator;

impl MempoolSimulator {
    /// Ingest a pending user swap intent, simulate its impact on the virtual liquidity graph,
    /// and immediately solve for a backrunning cyclic arbitrage bundle.
    pub fn simulate_backrun(
        graph: &LiquidityGraph,
        intent: &SwapIntent,
        target_block: u64,
        base_token: &str,
        gas_cost_token: f64,
        bribe_ratio: f64,
    ) -> Option<(RouteExecutionResult, FlashbotsBundle)> {
        // Create an isolated virtual clone of the liquidity graph to mutate
        let mut sim_graph = LiquidityGraph::new();

        for (_, pool_arc) in &graph.pools {
            let mut pool_clone = pool_arc.as_ref().clone();

            // If this is the victim pool being traded against, mutate its virtual reserves
            if pool_clone.pool_id() == intent.pool_id {
                let _ = pool_clone.execute_swap(&intent.token_in, intent.amount_in);
            }

            sim_graph.add_pool(pool_clone);
        }

        // Find arbitrage cycles originating from base_token
        let cycles = CycleEnumerator::find_cycles(&sim_graph, base_token, 2, 4);

        let mut best_execution: Option<RouteExecutionResult> = None;

        for cycle in cycles {
            if let Some(exec) = ArbitrageOptimizer::optimize_input_amount(
                &cycle,
                0.01,
                100.0,
                gas_cost_token,
                1e-4,
                32,
            ) {
                if exec.net_profit > 0.0 {
                    match &best_execution {
                        Some(current_best) if exec.net_profit > current_best.net_profit => {
                            best_execution = Some(exec);
                        }
                        None => {
                            best_execution = Some(exec);
                        }
                        _ => {}
                    }
                }
            }
        }

        let execution = best_execution?;
        let bundle = FlashbotsBundle::build(
            target_block,
            Some(&intent.tx_hash),
            &execution,
            bribe_ratio,
        );

        Some((execution, bundle))
    }
}
