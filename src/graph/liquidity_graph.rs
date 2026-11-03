use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::amm::AmmPool;
use super::edge::PoolEdge;

#[derive(Default, Clone)]
pub struct LiquidityGraph {
    pub tokens: HashSet<String>,
    pub adjacency: HashMap<String, Vec<PoolEdge>>,
    pub pools: HashMap<String, Arc<Box<dyn AmmPool>>>,
}

impl LiquidityGraph {
    pub fn new() -> Self {
        Self {
            tokens: HashSet::new(),
            adjacency: HashMap::new(),
            pools: HashMap::new(),
        }
    }

    pub fn add_pool(&mut self, pool: Box<dyn AmmPool>) {
        let pool_id = pool.pool_id().to_string();
        let t0 = pool.token0().to_string();
        let t1 = pool.token1().to_string();

        self.tokens.insert(t0.clone());
        self.tokens.insert(t1.clone());

        let pool_arc = Arc::new(pool);
        self.pools.insert(pool_id.clone(), pool_arc.clone());

        // Edge 0 -> 1
        if let Some(edge01) = PoolEdge::new(&t0, &t1, pool_arc.clone()) {
            self.adjacency.entry(t0.clone()).or_default().push(edge01);
        }

        // Edge 1 -> 0
        if let Some(edge10) = PoolEdge::new(&t1, &t0, pool_arc) {
            self.adjacency.entry(t1).or_default().push(edge10);
        }
    }

    pub fn get_neighbors(&self, token: &str) -> Option<&Vec<PoolEdge>> {
        self.adjacency.get(token)
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }
}
