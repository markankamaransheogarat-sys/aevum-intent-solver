use aevum_intent_solver::{
    AmmPool, ArbitrageOptimizer, BellmanFord, CurvePool, CycleEnumerator,
    FlashbotsBundle, LiquidityGraph, MempoolSimulator, SwapIntent,
    UniswapV2Pool, UniswapV3Pool,
};

#[test]
fn test_uniswap_v2_exact_math() {
    let pool = UniswapV2Pool::new(
        "test_v2",
        "WETH",
        "USDC",
        100.0,
        300_000.0,
        0.003, // 0.3%
    );

    // Swap 1 WETH -> USDC
    let amount_out = pool.get_amount_out("WETH", 1.0).expect("Swap failed");

    // Math: net_in = 1.0 * 0.997 = 0.997
    // out = 0.997 * 300,000 / (100 + 0.997) = 299,100 / 100.997 ≈ 2,961.47
    assert!(amount_out > 2960.0 && amount_out < 2963.0);
}

#[test]
fn test_uniswap_v3_concentrated_liquidity() {
    let pool = UniswapV3Pool::new(
        "test_v3",
        "USDC",
        "DAI",
        1.0,
        1_000_000.0,
        0.0005, // 0.05%
        0.99,
        1.01,
    );

    let amount_out = pool.get_amount_out("USDC", 1000.0).expect("Swap failed");
    // Near 1:1 with 0.05% fee, out should be ~999.5
    assert!(amount_out > 998.0 && amount_out < 1000.0);
}

#[test]
fn test_curve_stableswap_invariant() {
    let pool = CurvePool::new(
        "test_curve",
        "DAI",
        "USDC",
        5_000_000.0,
        5_000_000.0,
        100.0,
        0.0004,
    );

    let d = pool.calculate_invariant_d().expect("Invariant calculation failed");
    assert!((d - 10_000_000.0).abs() < 1.0);

    let amount_out = pool.get_amount_out("DAI", 10_000.0).expect("Curve swap failed");
    // With A=100 and near peg, slippage is tiny: ~9996
    assert!(amount_out > 9990.0 && amount_out <= 10_000.0);
}

#[test]
fn test_bellman_ford_negative_cycle_detection() {
    let mut graph = LiquidityGraph::new();

    // Artificially create an arbitrage triangle:
    // WETH -> USDC (3100)
    // USDC -> DAI (1.0)
    // DAI -> WETH (1 / 3000) -> Net gain ~3100/3000 > 1.0
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "pool_weth_usdc",
        "WETH",
        "USDC",
        100.0,
        310_000.0, // 3100
        0.001,
    )));
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "pool_usdc_dai",
        "USDC",
        "DAI",
        1_000_000.0,
        1_000_000.0, // 1.0
        0.001,
    )));
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "pool_dai_weth",
        "DAI",
        "WETH",
        300_000.0,
        100.0, // 3000
        0.001,
    )));

    let cycles = BellmanFord::find_negative_cycles(&graph, "WETH");
    assert!(!cycles.is_empty(), "Should detect negative cycle");
    assert!(cycles[0].theoretical_multiplier > 1.0);
}

#[test]
fn test_golden_section_optimal_sizing() {
    let mut graph = LiquidityGraph::new();

    graph.add_pool(Box::new(UniswapV2Pool::new(
        "p1", "WETH", "USDC", 100.0, 310_000.0, 0.003,
    )));
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "p2", "USDC", "WETH", 300_000.0, 100.0, 0.003,
    )));

    let cycles = CycleEnumerator::find_cycles(&graph, "WETH", 2, 2);
    assert!(!cycles.is_empty());

    let optimal_exec = ArbitrageOptimizer::optimize_input_amount(
        &cycles[0],
        0.1,
        50.0,
        0.005, // 0.005 WETH gas cost
        1e-4,
        32,
    ).expect("Optimizer should find positive net profit");

    assert!(optimal_exec.net_profit > 0.0);
    assert!(optimal_exec.input_amount > 0.1 && optimal_exec.input_amount < 50.0);
}

#[test]
fn test_flashbots_bundle_generation() {
    let mut graph = LiquidityGraph::new();
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "p1", "WETH", "USDC", 100.0, 310_000.0, 0.001,
    )));
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "p2", "USDC", "WETH", 300_000.0, 100.0, 0.001,
    )));

    let cycles = CycleEnumerator::find_cycles(&graph, "WETH", 2, 2);
    let exec = ArbitrageOptimizer::optimize_input_amount(
        &cycles[0], 0.1, 20.0, 0.001, 1e-4, 32,
    ).unwrap();

    let bundle = FlashbotsBundle::build(20_000_000, Some("0xvictim_tx"), &exec, 0.85);

    assert_eq!(bundle.target_block, 20_000_000);
    assert!(bundle.miner_bribe_eth > 0.0);
    assert!(bundle.searcher_profit_eth > 0.0);
    assert!((bundle.miner_bribe_eth + bundle.searcher_profit_eth - exec.net_profit).abs() < 1e-6);
    assert_eq!(bundle.transaction_hashes.len(), 2);
}

#[test]
fn test_mempool_whale_backrun_pipeline() {
    let mut graph = LiquidityGraph::new();

    // Baseline balanced pools
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "pool_uni_v2", "WETH", "USDC", 200.0, 600_000.0, 0.003, // 1 WETH = 3000 USDC
    )));
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "pool_sushi_v2", "USDC", "WETH", 600_000.0, 200.0, 0.003, // 1 WETH = 3000 USDC
    )));

    // Whale dumps 50 WETH on Sushi pool, driving WETH price down and USDC price up in that pool
    let whale_intent = SwapIntent::new(
        "0xwhale_swap_hash",
        "pool_sushi_v2",
        "WETH",
        "USDC",
        50.0,
        130_000.0,
    );

    let (exec, bundle) = MempoolSimulator::simulate_backrun(
        &graph,
        &whale_intent,
        21_000_000,
        "WETH",
        0.005,
        0.80,
    ).expect("Backrun opportunity must be captured after whale swap");

    assert!(exec.net_profit > 0.0);
    assert!(bundle.miner_bribe_eth > 0.0);
    assert!(bundle.searcher_profit_eth > 0.0);
    assert!(bundle.is_backrun);
}
