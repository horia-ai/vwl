# VWL Gate Sequence (Gate 0 to Gate 10)

| Gate | Title | Scope | Hard Stop / Falsification Gate |
|------|-------|-------|--------------------------------|
| **Gate 0** | **Minimum Falsifiable VWL** | Rust core, CLI, Git inspection, DTOs, append-only ledger, replay, OpenCode worker harness | Any replay divergence or loss of deterministic reproducibility |
| **Gate 1** | **Node and Project Identity** | Ed25519 decentralized signing keys, Node credentials, Project credentials, signed objects | Identity cannot be verified offline |
| **Gate 2** | **GitHub-Only Discovery** | Read-only GitHub adapter, `topic:vwl-project` convention, Role and Offer projections | Manual GitHub tampering alters signed VWL authority |
| **Gate 3** | **CLI/TUI Shell** | Terminal user interface (Ratatui): Home, Market, Projects, Node, Inbox screens | High latency or fragile GitHub dependency |
| **Gate 4** | **REP (Reputation Scoring)** | Raw evidence feature extraction, domain REP score v0, predictive benchmarks | Zero predictive lift over naive commit count |
| **Gate 5** | **Local Economic State Machine** | Double-entry accounting, WORK state conservation, atomic reservations, vesting | WORK state conservation violation |
| **Gate 6** | **Economic Simulator** | Agent-based Monte Carlo simulations: supply/demand, Sybil attacks, collusion | Sybil/collusion attack drains protocol |
| **Gate 7** | **Testnet Contracts** | Solidity contracts (NodeRegistry, Escrow, VestingSettlement), differential testing | Divergence between Solidity and Rust reference model |
| **Gate 8** | **Wallet & Chain TUI** | Direct contract interaction, milestone budgets, node wallet balance UX | Discrepancy between on-chain state and TUI |
| **Gate 9** | **Optional Fiat Onramp** | Funding provider abstraction, Stripe MCP server POC, purchase receipts | Fiat onramp creates hidden centralized authority |
| **Gate 10** | **Productive Transfer & Public DEX** | Internal order-book vs AMM comparison, legal & security go/no-go gate | Insufficient productive liquidity or regulatory failure |
