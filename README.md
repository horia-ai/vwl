# Virtual Work Ledger (VWL) v2.0
> **Decentralized Autonomous Labor Protocol for Asynchronous Coding Agents**  
> Based on the *VWL Bootstrap Work Ledger v1.0* specification.

[![CI](https://github.com/horia-ai/vwl/actions/workflows/ci.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/ci.yml)
[![Mutation Testing Gate](https://github.com/horia-ai/vwl/actions/workflows/mutation-testing.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/mutation-testing.yml)
[![Security & Secrets](https://github.com/horia-ai/vwl/actions/workflows/security.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/security.yml)
[![GitHub Pages](https://github.com/horia-ai/vwl/actions/workflows/gh-pages.yml/badge.svg)](https://horia-ai.github.io/vwl/)

---

## 🌟 Overview

**Virtual Work Ledger (VWL)** solves asynchronous coordination, verification, reputation, and financial settlement for autonomous AI software developers without centralized platform lock-in.

- **Small Git-native Work Units**: Work is scoped into bounded, falsifiable hypothesis units.
- **Independent Verification**: Evaluated by a bounded verifier with strictly zero conversational context from the implementer.
- **Separation of Authority**:
  - **Mutable Discovery Plane (GitHub)**: Issues, labels, topic discovery (`topic:vwl-project`). *Zero financial authority.*
  - **Authoritative Cryptographic Plane**: Immutable Task, WorkUnit, Proof, and Decision DTOs recorded append-only in `.vwl/ledger/`.
  - **Settlement Plane**: Invariant-enforced local economic state machine (Gate 5), smart contracts (Gate 7), and fiat onramps (Gate 9).
- **100% Replay Portability**: A clean clone reconstructs all historical work and derived state via `vwl replay`.

---

## 📁 Repository Structure

```text
├── Cargo.toml                     # Root Rust workspace (2021 edition)
├── package.json                   # Root npm workspace for TypeScript packages
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                 # Rust & Node tests, clippy, fmt, typecheck
│   │   ├── mutation-testing.yml   # Stryker mutation testing gate (>= 80%)
│   │   ├── security.yml           # Gitleaks secret scanning & audit
│   │   └── gh-pages.yml           # GitHub Pages deployment
│   ├── ISSUE_TEMPLATE/
│   └── pull_request_template.md
├── crates/
│   ├── vwl-core/                  # Core Rust engine: DTOs, Ledger, Replay, Git identity
│   ├── vwl-cli/                   # vwl CLI binary: status, task, work, replay, worker
│   └── vwl-tui/                   # Ratatui Terminal UI shell
├── packages/
│   ├── stripe-mcp/                # Stripe Model Context Protocol (MCP) server & fiat onramp
│   └── opencode-harness/          # Bounded worker and independent verifier agent harness
├── contracts/                     # Solidity smart contracts (Foundry workspace)
│   ├── contracts/                 # NodeRegistry, EngagementEscrow, VestingSettlement
│   └── foundry.toml
├── experiment/                    # Bootstrap experiment ledger
│   ├── hypothesis.yaml            # H-CORE, H-INDEPENDENCE, H-COMPOSITION, H-PORTABILITY
│   └── work-units/                # Bounded Work Unit manifests
└── docs/                          # GitHub Pages interactive documentation
```

---

## 🚦 Gate Sequence & Milestones

VWL strictly adheres to a sequential 11-gate implementation sequence:

1. [**Gate 0 — Minimum Falsifiable VWL**](https://github.com/horia-ai/vwl/milestone/1): Pure local CLI, Git inspection, DTOs, append-only ledger, replay, worktree isolation.
2. [**Gate 1 — Node & Project Identity**](https://github.com/horia-ai/vwl/milestone/2): Ed25519 decentralized signing keys, credentials, signed objects.
3. [**Gate 2 — GitHub-Only Discovery**](https://github.com/horia-ai/vwl/milestone/3): Read-only GitHub adapter, `topic:vwl-project` scanner, Role & Offer projections.
4. [**Gate 3 — CLI/TUI Shell**](https://github.com/horia-ai/vwl/milestone/4): Interactive Ratatui console for Home, Market, Projects, Node, and Inbox.
5. [**Gate 4 — REP (Reputation Scoring)**](https://github.com/horia-ai/vwl/milestone/5): Raw evidence extraction, domain REP scoring v0, predictive benchmarks.
6. [**Gate 5 — Local Economic State Machine**](https://github.com/horia-ai/vwl/milestone/6): WORK state conservation, dual-signed Engagements, atomic budget reservation.
7. [**Gate 6 — Economic Simulator**](https://github.com/horia-ai/vwl/milestone/7): Discrete-event agent simulations of labor markets, quality, and Sybil resistance.
8. [**Gate 7 — Testnet Contracts**](https://github.com/horia-ai/vwl/milestone/8): Solidity contracts with differential model testing against the Rust reference engine.
9. [**Gate 8 — Wallet & Chain TUI**](https://github.com/horia-ai/vwl/milestone/9): Direct EVM contract interaction and funded acceptance UX.
10. [**Gate 9 — Optional Fiat Onramp & Stripe**](https://github.com/horia-ai/vwl/milestone/10): Stripe MCP adapter for fiat-to-WORK token funding.
11. [**Gate 10 — Productive Transfer & Public DEX**](https://github.com/horia-ai/vwl/milestone/11): Comparative DEX order-book vs AMM simulation and legal/security go/no-go gate.

---

## 🛠️ Quickstart

### 1. Build and Run Rust CLI
```bash
# Build the workspace
cargo build

# Run unit and integration tests
cargo test

# Run strict Clippy linting
cargo clippy --all-targets --all-features -- -D warnings

# Check repository status and canonical base
cargo run -p vwl-cli -- status

# Replay ledger state
cargo run -p vwl-cli -- replay
```

### 2. TypeScript / MCP Packages
```bash
# Install dependencies
npm install

# Run Vitest test suite
npm test

# Run Stryker mutation testing (Threshold >= 80%)
cd packages/stripe-mcp && npm run test:mutate
```

---

## 🧪 TDD & Mutation Testing Protocol

Every PR must strictly follow the **Red $\to$ Green $\to$ Refactor $\to$ Mutate** cycle:
1. **Red**: Write unit and integration tests first. Verify they fail.
2. **Green**: Implement minimal production code to pass tests.
3. **Refactor**: Clean up, ensure zero compiler/clippy warnings and strict type adherence.
4. **Mutate**: Execute Stryker mutation testing (`npm run test:mutate`). Must achieve a **mutation score $\ge 80\%$** with zero surviving mutants on core business logic.

---

## 📄 License
Licensed under either of [MIT](LICENSE) or [Apache-2.0](LICENSE) at your option.
