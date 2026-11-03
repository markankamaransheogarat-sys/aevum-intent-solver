# Flashbots MEV & In-Memory Mempool Backrunning Architecture

## 1. Overview
MEV (Maximal Extractable Value) backrunning relies on atomic transaction inclusion immediately behind a target transaction (the "victim" or "whale" transaction) within the same block.

```
+-------------------------------------------------------------+
|                      PROPOSED BLOCK #N                      |
|                                                             |
|   +-----------------------------------------------------+   |
|   |                  FLASHBOTS BUNDLE                   |   |
|   |                                                     |   |
|   |  Tx #1: Victim Whale Swap (e.g. 150 WETH -> USDC)   |   |
|   |         --> Distorts Pool Spot Price                |   |
|   |                                                     |   |
|   |  Tx #2: Aevum Cyclic Backrun Arbitrage              |   |
|   |         --> Restores Invariant, Extracts Spread     |   |
|   |         --> Pays 85-90% Builder Tip via Coinbase    |   |
|   +-----------------------------------------------------+   |
|                                                             |
+-------------------------------------------------------------+
```

## 2. In-Memory Virtual State Simulation
Aevum achieves sub-30 µs latency by eliminating JSON-RPC node roundtrips. When a pending swap transaction intent arrives over WebSocket:
1. Virtual pool state is mutated in RAM via `execute_swap(token_in, amount_in)`.
2. Graph edges update their logarithmic weights $w(e) = -\ln(R \cdot (1 - \phi))$.
3. Elementary cycle discovery traverses candidate paths.
4. Golden Section optimizer calculates the exact capital $x^*$ that maximizes:
   $$\Pi_{\text{net}} = \text{AmountOut}(x^*) - x^* - \text{GasCost}$$
5. An atomic bundle is formatted with builder bribe:
   $$\text{Bribe} = \Pi_{\text{net}} \times \beta, \quad \beta \in [0.80, 0.95]$$
