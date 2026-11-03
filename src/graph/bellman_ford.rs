use std::collections::{HashMap, HashSet};
use super::edge::PoolEdge;
use super::liquidity_graph::LiquidityGraph;

#[derive(Debug, Clone)]
pub struct ArbitrageCycle {
    pub hops: Vec<PoolEdge>,
    pub theoretical_multiplier: f64,
    pub cumulative_weight: f64,
}

pub struct BellmanFord;

impl BellmanFord {
    /// Detect negative weight cycles starting from a base token (e.g. "WETH" or "USDC")
    pub fn find_negative_cycles(graph: &LiquidityGraph, start_token: &str) -> Vec<ArbitrageCycle> {
        let mut distances: HashMap<String, f64> = HashMap::new();
        let mut predecessors: HashMap<String, PoolEdge> = HashMap::new();

        for token in &graph.tokens {
            distances.insert(token.clone(), f64::INFINITY);
        }
        distances.insert(start_token.to_string(), 0.0);

        let v = graph.token_count();
        if v == 0 {
            return Vec::new();
        }

        // Relax edges V - 1 times
        for _ in 0..(v - 1) {
            let mut updated = false;
            for (u, edges) in &graph.adjacency {
                let dist_u = match distances.get(u) {
                    Some(&d) if d < f64::INFINITY => d,
                    _ => continue,
                };

                for edge in edges {
                    let v_token = &edge.to_token;
                    let current_dist_v = *distances.get(v_token).unwrap_or(&f64::INFINITY);
                    let new_dist_v = dist_u + edge.weight;

                    if new_dist_v < current_dist_v - 1e-9 {
                        distances.insert(v_token.clone(), new_dist_v);
                        predecessors.insert(v_token.clone(), edge.clone());
                        updated = true;
                    }
                }
            }
            if !updated {
                break;
            }
        }

        // V-th relaxation step: if any distance still decreases, there is a negative cycle!
        let mut cycles = Vec::new();
        let mut visited_cycle_nodes = HashSet::new();

        for (u, edges) in &graph.adjacency {
            let dist_u = match distances.get(u) {
                Some(&d) if d < f64::INFINITY => d,
                _ => continue,
            };

            for edge in edges {
                let v_token = &edge.to_token;
                let current_dist_v = *distances.get(v_token).unwrap_or(&f64::INFINITY);

                if dist_u + edge.weight < current_dist_v - 1e-9 {
                    // Negative cycle detected! Trace back to find cycle nodes
                    let mut curr = v_token.clone();
                    for _ in 0..v {
                        if let Some(pred_edge) = predecessors.get(&curr) {
                            curr = pred_edge.from_token.clone();
                        } else {
                            break;
                        }
                    }

                    if visited_cycle_nodes.contains(&curr) {
                        continue;
                    }

                    // Reconstruct cycle starting from `curr`
                    let mut cycle_edges = Vec::new();
                    let cycle_start = curr.clone();
                    let mut step = curr.clone();
                    let mut total_weight = 0.0;
                    let mut rate_prod = 1.0;

                    loop {
                        if let Some(edge_to_step) = predecessors.get(&step) {
                            cycle_edges.push(edge_to_step.clone());
                            total_weight += edge_to_step.weight;
                            rate_prod *= edge_to_step.spot_price * (1.0 - edge_to_step.fee);
                            step = edge_to_step.from_token.clone();
                            visited_cycle_nodes.insert(step.clone());
                            if step == cycle_start || cycle_edges.len() > v {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    cycle_edges.reverse();

                    if !cycle_edges.is_empty() && total_weight < -1e-9 && rate_prod > 1.0001 {
                        cycles.push(ArbitrageCycle {
                            hops: cycle_edges,
                            theoretical_multiplier: rate_prod,
                            cumulative_weight: total_weight,
                        });
                    }
                }
            }
        }

        cycles
    }
}
