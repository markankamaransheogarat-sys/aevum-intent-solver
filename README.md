# ⚡ AEVUM: Ultra-Low Latency Cyclic Arbitrage & MEV Intent Solver Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-00ff66.svg?style=flat-square)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Rust-1.85%2B%20%7C%202021%20Edition-dea584.svg?style=flat-square)](https://www.rust-lang.org/)
[![Latency: Sub-30µs](https://img.shields.io/badge/Solver%20Latency-29.03%20%C2%B5s-00e5ff.svg?style=flat-square)](#)
[![Pools: Hybrid AMM](https://img.shields.io/badge/DEX%20Invariants-Uni%20v2%20%7C%20Uni%20v3%20%7C%20Curve-ff0055.svg?style=flat-square)](#)
[![Tests: 7/7 Passing](https://img.shields.io/badge/Tests-7%2F7%20Passing-00ff66.svg?style=flat-square)](#)

```
  ▄████▄  ▓█████ ▄▄▄     ▓█████▄▄▄█████▓ ▒█████   ██▓     ██▒   █▓▓█████  ██▀███  
 ▒██▀ ▀█  ▓█   ▀▒████▄   ▓█   ▀▓  ██▒ ▓▒▒██▒  ██▒▓██▒    ▓██░   █▒▓█   ▀ ▓██ ▒ ██▒
 ▒▓█    ▄ ▒███  ▒██  ▀█▄ ▒███  ▒ ▓██░ ▒░▒██░  ██▒▒██░     ▓██  █▒░▒███   ▓██ ░▄█ ▒
 ▒▓▓▄ ▄██▒▒▓█  ▄░██▄▄▄▄██▒▓█  ▄ ░ ▓██▓ ░ ▒██   ██░▒██░      ▒██ █░░▒▓█  ▄ ▒██▀▀█▄  
 ▒ ▓███▀ ░░▒████▒▓█   ▓██▒░▒████▒  ▒██▒ ░ ░ ████▓▒░░██████▒   ▒▀█░  ░▒████▒░██▓ ▒██▒
 ░ ░▒ ▒  ░░░ ▒░ ░▒▒   ▓▒█░░░ ▒░ ░  ▒ ░░   ░ ▒░▒░▒░ ░ ▒░▓  ░   ░ ▐░  ░░ ▒░ ░░ ▒▓ ░▒▓░
   ░  ▒    ░ ░  ░ ▒   ▒▒ ░ ░ ░  ░    ░      ░ ▒ ▒░ ░ ░ ▒  ░   ░ ░░   ░ ░  ░  ░▒ ░ ▒░
 ░           ░    ░   ▒      ░     ░      ░ ░ ░ ▒    ░ ░        ░░     ░     ░░   ░ 
 ░ ░         ░  ░     ░  ░   ░  ░             ░ ░      ░  ░      ░     ░  ░   ░     
 ░                                                               ░                  
```

> *"Every microsecond on the wire is someone else's frontrun."*

**Aevum** is an ultra-low latency, in-memory **Cross-DEX Cyclic Arbitrage & MEV Intent Solver Engine** implemented in modern Rust. It continuously ingests pending mempool transactions, maintains virtual liquidity pool graphs, and solves multi-hop arbitrage loops across heterogeneous AMM invariants (**Uniswap v2**, **Uniswap v3 Concentrated Liquidity**, and **Curve Stableswap**) in **29 microseconds**.

---

## 🌟 Key Capabilities

- **Zero-RPC In-Memory State Delta**: Eliminates slow JSON-RPC queries by applying pending swap intents directly to virtual pool models in RAM.
- **Logarithmic Negative-Cycle Equivalence**: Transforms product-of-rates cyclic search $\prod R_i \cdot (1 - \phi_i) > 1$ into an additive negative weight cycle problem $\sum -\ln(R_i \cdot (1 - \phi_i)) < 0$ via modified Bellman-Ford and bounded Johnson DFS.
- **Convex Optimal Volume Solver**: Golden Section Search determines the exact input capital $x^*$ that maximizes net profit after slippage and gas fees in $<32$ iterations without numerical differentiation.
- **Heterogeneous Pool Math**:
  - **Uniswap v2**: Constant product invariant $(x \cdot y = k)$.
  - **Uniswap v3**: Concentrated liquidity $\sqrt{P}$ tick math and virtual liquidity depth $L$.
  - **Curve Finance**: Multi-asset Stableswap invariant with amplification coefficient $A$ solved via Newton-Raphson approximation.
- **Flashbots MEV Bundle Builder**: Constructs atomic transaction bundles with dynamic builder/miner bribe allocation ($80\%\text{–}95\%$).

---

## 📐 Mathematical Foundations

### 1. Logarithmic Cycle Detection
For a sequence of pool edges $(e_1, e_2, \dots, e_k)$ with spot prices $R(e_i)$ and fee tiers $\phi_{e_i}$:

$$\text{Net Exchange Rate } \eta(e) = R(e) \cdot (1 - \phi_e)$$

A cyclic route is profitable if and only if:

$$\prod_{i=1}^k \eta(e_i) > 1.0 \iff \sum_{i=1}^k -\ln(\eta(e_i)) < 0$$

Edges are weighted with $w(e) = -\ln(R(e) \cdot (1 - \phi_e))$, enabling negative-cycle detection algorithms to isolate profitable loops in sub-millisecond timeframes.

### 2. Convex Trade Sizing via Golden Section Search
Let $\mathcal{F}(x)$ be the composite execution function along the cyclic route. Net profit is unimodal and concave:

$$\Pi(x) = \mathcal{F}(x) - x - C_{\text{gas}}$$

Using the golden ratio conjugate $\phi = \frac{\sqrt{5} - 1}{2} \approx 0.6180339887$, the solver brackets the optimal trade size $x^* \in [x_{\min}, x_{\max}]$:

$$c = a + (1 - \phi)(b - a), \quad d = a + \phi(b - a)$$

Convergence occurs in fewer than 32 scalar evaluations to precision $\epsilon < 10^{-4}$.

---

## ⚡ Live Benchmark & Telemetry

When executed in `--release` profile on standard server hardware:

```
========================================================================
 ⚡ AEVUM: Ultra-Low Latency Cyclic Arbitrage & MEV Intent Solver 
========================================================================
[*] Initialized In-Memory Liquidity Graph with 4 pools and 3 tokens
[*] Pending Mempool Intent Received:
    TxHash:   0x7f4c91a0b3882ef9482701e3b5e408819bc23851b402809f8728d154fa92019a
    Action:   Sell 150 WETH on Pool uni_v2_weth_usdc

[*] Solving Backrun Cyclic Arbitrage & Generating Flashbots Bundle...
[+] ARBITRAGE OPPORTUNITY FOUND & SOLVED in 29.03 µs!

--- Execution Details ---
  Optimal Capital Input:  100.0000 WETH
  Output Received:        134.4441 WETH
  Gross Profit:           34.4442 WETH
  Gas Cost:               0.0080 WETH
  Net Profit:             34.4362 WETH
  Net ROI:                34.44%

--- Cyclic Hop Route ---
  Hop 1: [uni_v3_dai_weth] WETH -> DAI  | In: 100.0000 -> Out: 302141.0006
  Hop 2: [curve_dai_usdc]  DAI  -> USDC | In: 302141.0006 -> Out: 302008.7684
  Hop 3: [uni_v2_weth_usdc] USDC -> WETH | In: 302008.7684 -> Out: 134.4441

--- Flashbots MEV Bundle ---
  Bundle ID:              bundle_21450982_2
  Target Block:           #21450982
  Builder Bribe (85%):    29.2707 WETH
  Searcher Net (15%):     5.1654 WETH
  Bundle Tx Chain:        ["0x7f4c91a0b388...", "0xsearcher_arb_uni_v3"]
```

---

## 🗂 Project Architecture

```
aevum-intent-solver/
├── Cargo.toml
├── LICENSE
├── README.md
├── architecture/
│   ├── ALGORITHMIC_SPEC_RFC.md   # Complete mathematical & algorithmic specification
│   └── FLASHBOTS_INTEGRATION.md  # Mempool backrunning & MEV bundle structure
├── src/
│   ├── amm/                      # AMM Invariant Models
│   │   ├── pool.rs               # AmmPool trait & error definitions
│   │   ├── uniswap_v2.rs         # Constant Product (x * y = k)
│   │   ├── uniswap_v3.rs         # Concentrated Liquidity SqrtPrice & Tick Math
│   │   └── curve.rs              # Curve Stableswap Invariant Newton-Raphson
│   ├── graph/                    # Liquidity Multigraph Engine
│   │   ├── edge.rs               # Negative logarithmic edge weighting
│   │   ├── liquidity_graph.rs    # Directed token-pool adjacency graph
│   │   ├── bellman_ford.rs       # Negative cycle detection
│   │   └── johnson.rs            # Bounded DFS elementary cycle enumerator
│   ├── solver/                   # Arbitrage & Convex Sizing
│   │   ├── route.rs              # Sequential pool hop simulation
│   │   └── optimizer.rs          # Golden Section Search optimizer
│   ├── mempool/                  # Intent Processing & MEV Bundling
│   │   ├── intent.rs             # Pending swap intent representation
│   │   ├── simulator.rs          # In-memory virtual reserve state transitions
│   │   └── bundle.rs             # Flashbots bundle builder & bribe calculator
│   ├── telemetry/                # Microsecond stopwatch & latency profiling
│   ├── lib.rs                    # Public crate exports
│   └── main.rs                   # End-to-end benchmark showcase CLI
└── tests/
    └── arbitrage_tests.rs        # 7 comprehensive unit & integration test suites
```

---

## 🚀 Quickstart

### 1. Build and Test
```bash
git clone https://github.com/markankamaransheogarat-sys/aevum-intent-solver.git
cd aevum-intent-solver

# Run full test suite (7/7 passing)
cargo test
```

### 2. Run the Benchmark CLI
```bash
cargo run --release
```

---

## 📜 License & Attribution

Designed and engineered by **Konstantin ([@markankamaransheogarat-sys](https://github.com/markankamaransheogarat-sys))**.  
Released under the [MIT License](LICENSE).
