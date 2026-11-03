# RFC 2026-AEV-01: Aevum Cross-DEX Cyclic Arbitrage Specification

**Author:** Konstantin ([@markankamaransheogarat-sys](https://github.com/markankamaransheogarat-sys))  
**Status:** Standards Track / Implemented  
**Date:** November 2026  

---

## 1. Abstract
This RFC defines the mathematical, architectural, and algorithmic framework for the **Aevum Intent Solver Engine**. Aevum models decentralized financial liquidity as a directed multigraph, continuously streaming pending mempool transactions, simulating virtual reserve states in memory, and resolving optimal cyclic arbitrage bundles within sub-50 microsecond execution windows.

---

## 2. Graph Modeling & Negative Cycle Equivalence

Let $G = (V, E)$ be a directed multigraph where:
- $V$ is the set of tokens (e.g. WETH, USDC, DAI, WBTC, USDT).
- $E$ is the set of directed pool edges representing active liquidity pools (Uniswap v2, Uniswap v3, Curve).

For each pool edge $e = (u, v)$ with spot exchange rate $R(e)$ and protocol fee $\phi_e \in [0, 1)$, the net exchange factor is:

$$\eta(e) = R(e) \cdot (1 - \phi_e)$$

A cyclic route $C = (e_1, e_2, \dots, e_k)$ starting and ending at $v_0 \in V$ yields a gross capital multiplier:

$$\mathcal{M}(C) = \prod_{i=1}^k \eta(e_i)$$

A risk-free cyclic arbitrage exists if and only if $\mathcal{M}(C) > 1.0$.

By applying a monotonic logarithmic transformation:

$$w(e) = -\ln\left(\eta(e)\right) = -\ln\left(R(e) \cdot (1 - \phi_e)\right)$$

The condition $\prod_{i=1}^k \eta(e_i) > 1.0$ is strictly equivalent to:

$$\sum_{i=1}^k w(e_i) < 0$$

Thus, the problem of discovering cyclic arbitrage is mapped directly onto **Negative Weight Cycle Detection** over directed multigraphs.

---

## 3. AMM Invariant State Math

### 3.1 Constant Product AMM (Uniswap v2 / Sushiswap)
For reserves $(x, y)$ and input $\Delta x$:

$$\Delta y = \frac{y \cdot \Delta x \cdot (1 - \phi)}{x + \Delta x \cdot (1 - \phi)}$$

### 3.2 Concentrated Liquidity (Uniswap v3)
Given active liquidity $L$ and current square root price $\sqrt{P} = \sqrt{\frac{y}{x}}$:
- Selling token 0 for token 1:
  $$\frac{1}{\sqrt{P_{\text{next}}}} = \frac{1}{\sqrt{P}} + \frac{\Delta x \cdot (1 - \phi)}{L}$$
  $$\Delta y = L \cdot \left(\sqrt{P} - \sqrt{P_{\text{next}}}\right)$$
- Selling token 1 for token 0:
  $$\sqrt{P_{\text{next}}} = \sqrt{P} + \frac{\Delta y \cdot (1 - \phi)}{L}$$
  $$\Delta x = L \cdot \left(\frac{1}{\sqrt{P}} - \frac{1}{\sqrt{P_{\text{next}}}}\right)$$

### 3.3 Stableswap Invariant (Curve Finance)
For 2-token pools under invariant $D$ with amplification $A$:

$$A \cdot n^n \sum_{i=1}^n x_i + D = A \cdot D \cdot n^n + \frac{D^{n+1}}{n^n \prod_{i=1}^n x_i}$$

$D$ is solved iteratively via Newton-Raphson:

$$D_{k+1} = \frac{\left(4A \cdot S + 2 D_P\right) D_k}{(4A - 1) D_k + 3 D_P}, \quad \text{where } D_P = \frac{D_k^3}{4 x y}$$

---

## 4. Convex Optimal Sizing via Golden Section Search

Gross output after $k$ sequential pool hops is a strictly concave function $\mathcal{F}(x)$ due to increasing price impact and slippage:

$$\Pi(x) = \mathcal{F}(x) - x - C_{\text{gas}}$$

Because $\Pi(x)$ is unimodal over the interval $[x_{\min}, x_{\max}]$, Golden Section Search finds the global optimum $x^*$ satisfying $\Pi'(x^*) = 0$ in $O\left(\log \frac{x_{\max} - x_{\min}}{\epsilon}\right)$ function evaluations:

$$\phi = \frac{\sqrt{5} - 1}{2} \approx 0.6180339887$$

In fewer than 32 iterations, $x^*$ is resolved to an absolute precision of $\epsilon < 10^{-4}$ without analytical derivative computation across hybrid pool boundaries.
