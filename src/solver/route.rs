use crate::graph::ArbitrageCycle;

#[derive(Debug, Clone)]
pub struct HopExecution {
    pub pool_id: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub effective_price: f64,
}

#[derive(Debug, Clone)]
pub struct RouteExecutionResult {
    pub hops: Vec<HopExecution>,
    pub input_amount: f64,
    pub output_amount: f64,
    pub gross_profit: f64,
    pub gas_cost_token: f64,
    pub net_profit: f64,
    pub roi_percent: f64,
}

pub struct RouteSimulator;

impl RouteSimulator {
    /// Simulate exact trade execution through every pool in the cycle
    pub fn simulate_cycle(
        cycle: &ArbitrageCycle,
        initial_amount_in: f64,
        gas_cost_token: f64,
    ) -> Option<RouteExecutionResult> {
        if initial_amount_in <= 0.0 || cycle.hops.is_empty() {
            return None;
        }

        let mut current_amount = initial_amount_in;
        let mut hops_exec = Vec::new();

        for edge in &cycle.hops {
            let pool = edge.pool.as_ref();
            let amount_out = pool.get_amount_out(&edge.from_token, current_amount).ok()?;
            if amount_out <= 0.0 {
                return None;
            }

            hops_exec.push(HopExecution {
                pool_id: edge.pool_id.clone(),
                token_in: edge.from_token.clone(),
                token_out: edge.to_token.clone(),
                amount_in: current_amount,
                amount_out,
                effective_price: amount_out / current_amount,
            });

            current_amount = amount_out;
        }

        let gross_profit = current_amount - initial_amount_in;
        let net_profit = gross_profit - gas_cost_token;
        let roi_percent = (gross_profit / initial_amount_in) * 100.0;

        Some(RouteExecutionResult {
            hops: hops_exec,
            input_amount: initial_amount_in,
            output_amount: current_amount,
            gross_profit,
            gas_cost_token,
            net_profit,
            roi_percent,
        })
    }
}
