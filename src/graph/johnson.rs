use super::bellman_ford::ArbitrageCycle;
use super::edge::PoolEdge;
use super::liquidity_graph::LiquidityGraph;

pub struct CycleEnumerator;

impl CycleEnumerator {
    /// Enumerate all elementary cycles of length between `min_hops` and `max_hops` originating from `base_token`
    pub fn find_cycles(
        graph: &LiquidityGraph,
        base_token: &str,
        min_hops: usize,
        max_hops: usize,
    ) -> Vec<ArbitrageCycle> {
        let mut profitable_cycles = Vec::new();
        let mut current_path: Vec<PoolEdge> = Vec::new();

        Self::dfs(
            graph,
            base_token,
            base_token,
            min_hops,
            max_hops,
            &mut current_path,
            &mut profitable_cycles,
        );

        profitable_cycles
    }

    fn dfs(
        graph: &LiquidityGraph,
        current_token: &str,
        target_token: &str,
        min_hops: usize,
        max_hops: usize,
        path: &mut Vec<PoolEdge>,
        results: &mut Vec<ArbitrageCycle>,
    ) {
        if path.len() >= max_hops {
            return;
        }

        let neighbors = match graph.get_neighbors(current_token) {
            Some(n) => n,
            None => return,
        };

        for edge in neighbors {
            let next_token = &edge.to_token;

            // Check if we reached the cycle target
            if next_token == target_token {
                if path.len() + 1 >= min_hops {
                    let mut full_path = path.clone();
                    full_path.push(edge.clone());

                    let mut total_weight = 0.0;
                    let mut rate_prod = 1.0;
                    for e in &full_path {
                        total_weight += e.weight;
                        rate_prod *= e.spot_price * (1.0 - e.fee);
                    }

                    if rate_prod > 1.0002 {
                        results.push(ArbitrageCycle {
                            hops: full_path,
                            theoretical_multiplier: rate_prod,
                            cumulative_weight: total_weight,
                        });
                    }
                }
                continue;
            }

            // Avoid intermediate loops (do not revisit tokens already in path)
            if path.iter().any(|e| e.from_token == *next_token) {
                continue;
            }

            path.push(edge.clone());
            Self::dfs(graph, next_token, target_token, min_hops, max_hops, path, results);
            path.pop();
        }
    }
}
