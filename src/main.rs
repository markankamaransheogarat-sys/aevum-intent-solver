use aevum_intent_solver::{
    CurvePool, LatencyStopwatch, LiquidityGraph, MempoolSimulator,
    SwapIntent, UniswapV2Pool, UniswapV3Pool,
};

fn main() {
    println!("\x1b[36m========================================================================\x1b[0m");
    println!("\x1b[1;32m ⚡ AEVUM: Ultra-Low Latency Cyclic Arbitrage & MEV Intent Solver \x1b[0m");
    println!("\x1b[36m========================================================================\x1b[0m");

    let mut graph = LiquidityGraph::new();

    // Pool 1: Uniswap v2 WETH / USDC (Spot: 1 WETH = 3,000 USDC)
    graph.add_pool(Box::new(UniswapV2Pool::new(
        "uni_v2_weth_usdc",
        "WETH",
        "USDC",
        500.0,
        1_500_000.0,
        0.003, // 30 bps
    )));

    // Pool 2: Uniswap v3 USDC / DAI (Spot: 1 USDC = 1.0005 DAI)
    graph.add_pool(Box::new(UniswapV3Pool::new(
        "uni_v3_usdc_dai",
        "USDC",
        "DAI",
        1.0005,
        10_000_000.0,
        0.0005, // 5 bps
        0.99,
        1.01,
    )));

    // Pool 3: Curve DAI / USDC Stableswap (Amplification A = 200, Fee = 0.04%)
    graph.add_pool(Box::new(CurvePool::new(
        "curve_dai_usdc",
        "DAI",
        "USDC",
        20_000_000.0,
        20_000_000.0,
        200.0,
        0.0004,
    )));

    // Pool 4: Uniswap v3 DAI / WETH (Spot: 1 WETH = 3,025 DAI - Price discrepancy!)
    graph.add_pool(Box::new(UniswapV3Pool::new(
        "uni_v3_dai_weth",
        "DAI",
        "WETH",
        1.0 / 3025.0,
        8_000_000.0,
        0.0005, // 5 bps
        1.0 / 3200.0,
        1.0 / 2800.0,
    )));

    println!("[*] Initialized In-Memory Liquidity Graph with {} pools and {} tokens",
        graph.pool_count(), graph.token_count());

    // Simulated Pending Mempool Intent: Whale swaps 150 WETH -> USDC on Uniswap v2
    let whale_intent = SwapIntent::new(
        "0x7f4c91a0b3882ef9482701e3b5e408819bc23851b402809f8728d154fa92019a",
        "uni_v2_weth_usdc",
        "WETH",
        "USDC",
        150.0,
        420_000.0,
    );

    println!("[*] Pending Mempool Intent Received:");
    println!("    TxHash:   {}", whale_intent.tx_hash);
    println!("    Action:   Sell {} {} on Pool {}", whale_intent.amount_in, whale_intent.token_in, whale_intent.pool_id);

    println!("\n[*] Solving Backrun Cyclic Arbitrage & Generating Flashbots Bundle...");
    let timer = LatencyStopwatch::start();

    let target_block = 21_450_982;
    let gas_cost_weth = 0.008; // ~200k gas at 25 gwei
    let bribe_ratio = 0.85;    // 85% bribe to block builder

    let result = MempoolSimulator::simulate_backrun(
        &graph,
        &whale_intent,
        target_block,
        "WETH",
        gas_cost_weth,
        bribe_ratio,
    );

    let latency_us = timer.elapsed_micros();

    match result {
        Some((execution, bundle)) => {
            println!("\x1b[1;32m[+] ARBITRAGE OPPORTUNITY FOUND & SOLVED in {:.2} µs!\x1b[0m", latency_us);
            println!("\n--- Execution Details ---");
            println!("  Optimal Capital Input:  {:.4} WETH", execution.input_amount);
            println!("  Output Received:        {:.4} WETH", execution.output_amount);
            println!("  Gross Profit:           {:.4} WETH", execution.gross_profit);
            println!("  Gas Cost:               {:.4} WETH", execution.gas_cost_token);
            println!("  Net Profit:             \x1b[1;33m{:.4} WETH\x1b[0m", execution.net_profit);
            println!("  Net ROI:                {:.2}%", execution.roi_percent);

            println!("\n--- Cyclic Hop Route ---");
            for (idx, hop) in execution.hops.iter().enumerate() {
                println!("  Hop {}: [{}] {} -> {} | In: {:.4} -> Out: {:.4} (Eff Price: {:.6})",
                    idx + 1, hop.pool_id, hop.token_in, hop.token_out, hop.amount_in, hop.amount_out, hop.effective_price);
            }

            println!("\n--- Flashbots MEV Bundle ---");
            println!("  Bundle ID:              {}", bundle.bundle_id);
            println!("  Target Block:           #{}", bundle.target_block);
            println!("  Builder Bribe (85%):    {:.4} WETH", bundle.miner_bribe_eth);
            println!("  Searcher Net (15%):     \x1b[1;32m{:.4} WETH\x1b[0m", bundle.searcher_profit_eth);
            println!("  Bundle Tx Chain:        {:?}", bundle.transaction_hashes);
        }
        None => {
            println!("\x1b[31m[-] No profitable arbitrage cycle found above gas threshold.\x1b[0m");
        }
    }
}
