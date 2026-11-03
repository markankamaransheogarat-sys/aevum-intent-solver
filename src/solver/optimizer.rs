use crate::graph::ArbitrageCycle;
use super::route::{RouteExecutionResult, RouteSimulator};

const INV_PHI: f64 = 0.618033988749895; // (sqrt(5) - 1) / 2
const INV_PHI2: f64 = 0.381966011250105; // 1 - INV_PHI

pub struct ArbitrageOptimizer;

impl ArbitrageOptimizer {
    /// Find the optimal input amount x* using Golden Section Search that maximizes net profit:
    /// Maximize: NetProfit(x) = SimulateCycle(x) - x - GasCost
    pub fn optimize_input_amount(
        cycle: &ArbitrageCycle,
        min_amount: f64,
        max_amount: f64,
        gas_cost_token: f64,
        tolerance: f64,
        max_iterations: usize,
    ) -> Option<RouteExecutionResult> {
        if min_amount >= max_amount || min_amount <= 0.0 {
            return None;
        }

        let mut a = min_amount;
        let mut b = max_amount;
        let mut h = b - a;

        let mut c = a + INV_PHI2 * h;
        let mut d = a + INV_PHI * h;

        let eval = |x: f64| -> f64 {
            match RouteSimulator::simulate_cycle(cycle, x, gas_cost_token) {
                Some(res) => res.net_profit,
                None => -f64::INFINITY,
            }
        };

        let mut fc = eval(c);
        let mut fd = eval(d);

        for _ in 0..max_iterations {
            if (b - a).abs() < tolerance {
                break;
            }

            if fc > fd {
                b = d;
                d = c;
                fd = fc;
                h = b - a;
                c = a + INV_PHI2 * h;
                fc = eval(c);
            } else {
                a = c;
                c = d;
                fc = fd;
                h = b - a;
                d = a + INV_PHI * h;
                fd = eval(d);
            }
        }

        let best_x = (a + b) / 2.0;
        let best_result = RouteSimulator::simulate_cycle(cycle, best_x, gas_cost_token)?;

        if best_result.net_profit > 0.0 {
            Some(best_result)
        } else {
            None
        }
    }
}
