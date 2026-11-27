# Virtual Work Ledger (VWL) v2.0
> **Decentralized Autonomous Labor Protocol for Asynchronous Coding Agents**  
> Based on the *VWL Bootstrap Work Ledger v1.0* specification.

[![CI](https://github.com/horia-ai/vwl/actions/workflows/ci.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/ci.yml)
[![IaC Check](https://github.com/horia-ai/vwl/actions/workflows/iac.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/iac.yml)
[![Mutation Testing Gate](https://github.com/horia-ai/vwl/actions/workflows/mutation-testing.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/mutation-testing.yml)
[![Security & Secrets](https://github.com/horia-ai/vwl/actions/workflows/security.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/security.yml)
[![Publish Package](https://github.com/horia-ai/vwl/actions/workflows/publish-package.yml/badge.svg)](https://github.com/horia-ai/vwl/actions/workflows/publish-package.yml)
[![GitHub Pages](https://github.com/horia-ai/vwl/actions/workflows/gh-pages.yml/badge.svg)](https://horia-ai.github.io/vwl/)
[![Analytics Dashboard](https://img.shields.io/badge/GitHub%20Pages-Analytics%20%26%20REP-success)](https://horia-ai.github.io/vwl/analytics.html)
[![DEX Testnet Portal](https://img.shields.io/badge/DEX%20Guide-Token--Gated-purple)](https://horia-ai.github.io/vwl/dex-guide.html)
[![Project Board](https://img.shields.io/badge/GitHub%20Project-VWL%20Board-blue)](https://github.com/orgs/horia-ai/projects/1)

---

## 🌟 Overview

**Virtual Work Ledger (VWL)** solves asynchronous coordination, verification, reputation, and financial settlement for autonomous AI software developers without centralized platform lock-in.

- **Small Git-native Work Units**: Work is scoped into bounded, falsifiable hypothesis units.
- **Independent Verification**: Evaluated by a bounded verifier with strictly zero conversational context from the implementer.
- **Pluggable Agent Harnesses**: Support for OpenCode, Claude Code (`.claude/skills/`), GitHub Copilot (`.github/copilot-instructions.md`), and custom runtime harnesses.
- **Dynamic Skill Injection**: Auto-configure repo-level gate rules and behavioral protocols via `vwl skill install`.
- **NPX Distribution**: Direct execution without manual cloning or toolchain configuration via `npx @horia-ai/vwl`.
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
│   │   ├── publish-package.yml    # Build & publish @horia-ai/vwl to GitHub Packages
│   │   ├── security.yml           # Gitleaks secret scanning & audit
│   │   └── gh-pages.yml           # GitHub Pages deployment
│   ├── ISSUE_TEMPLATE/
│   └── pull_request_template.md
├── crates/
│   ├── vwl-core/                  # Core Rust engine: DTOs, Ledger, Replay, Git identity, Skills
│   ├── vwl-cli/                   # vwl CLI binary: status, task, work, replay, worker, skill, report
│   └── vwl-tui/                   # Ratatui Terminal UI shell
├── packages/
│   ├── vwl/                       # @horia-ai/vwl executable NPX runner package
│   ├── stripe-mcp/                # Stripe Model Context Protocol (MCP) server & fiat onramp
│   └── opencode-harness/          # Pluggable harness adapters (OpenCode, Claude, Copilot)
├── contracts/                     # Solidity smart contracts (Foundry workspace)
│   ├── contracts/                 # WorkToken, MockUSDC, NodeRegistry, EngagementEscrow
│   ├── test/                      # DEXPoolIntegration.t.sol (Uniswap v3 constant product)
│   └── foundry.toml
├── experiment/                    # Bootstrap experiment ledger
│   ├── hypothesis.yaml            # H-CORE, H-INDEPENDENCE, H-COMPOSITION, H-PORTABILITY
│   └── work-units/                # Bounded Work Unit manifests
├── iac/                           # Infrastructure as Code (Terraform for GitHub teams & projects)
└── docs/                          # GitHub Pages interactive documentation & token-gated DEX portal
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

### 1. Instant Execution via NPX
```bash
# Run status check directly without local installation
npx @horia-ai/vwl status

# Generate contributor effectiveness & cost allocation report
npx @horia-ai/vwl report

# List available protocol skills
npx @horia-ai/vwl skill list

# Inject TDD & mutation testing skills into current repo (Claude, Copilot, OpenCode)
npx @horia-ai/vwl skill install tdd-mutation-testing --harness all
```

### 2. Build and Run Native Rust CLI
```bash
# Build the workspace
cargo build

# Run unit and integration tests
cargo test

# Run strict Clippy linting
cargo clippy --all-targets --all-features -- -D warnings

# Check repository status and canonical base
cargo run -p vwl-cli -- status

# Generate contributor effectiveness, REP, and cost allocation report in terminal
cargo run -p vwl-cli -- report

# Export report to GitHub Pages dashboard (docs/data/ledger_report.json)
cargo run -p vwl-cli -- report --gh-pages

# Replay ledger state
cargo run -p vwl-cli -- replay
```

### 3. Agent Harness & Multi-Harness Skill Management
```bash
# List built-in protocol skills
vwl skill list

# Install independent verifier skill for Claude Code (.claude/skills/)
vwl skill install bounded-verifier --harness claude

# Install economic conservation rules for GitHub Copilot (.github/copilot-instructions.md)
vwl skill install economic-conservation --harness copilot

# Run developer worker using selected agent harness
vwl worker developer TASK-2026-001 --harness opencode
```

### 4. TypeScript / MCP Packages & Mutation Testing
```bash
# Install dependencies
npm install

# Run Vitest test suite across all packages
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

## 📚 Wiki-LLM & Authoritative Specifications

The complete v2.2 protocol and architecture specifications are curated for LLMs, autonomous coding agents, and developers in [`docs/wiki-llm/`](docs/wiki-llm/README.md) and [`llms.txt`](llms.txt):

- **[VWL v2.2 Core Protocol & Verification Factory](docs/wiki-llm/01-protocol-core-and-verification-factory.md)**: 4-Graph model, WorkUnits, Verification Factory planes/gates/shadow contracts, and Replay.
- **[VWL v2.2 Protocol CLI & GitHub Projection](docs/wiki-llm/02-protocol-cli-and-github-projection.md)**: CLI subcommands (`work`, `verify`, `graph`, `repometer`, `outcome`), TUI, and GitHub WorkHost projection.
- **[VWL Economics & Settlement v1.1](docs/wiki-llm/03-economics-and-settlement.md)**: REP scoring, WORK wages, Escrow, Vesting schedules, Solidity smart contract equivalence, and DEX liquidity.
- **[VWL Bootstrap Work Ledger v1.1](docs/wiki-llm/04-bootstrap-work-ledger.md)**: Falsification gates (G0 through G10) and bootstrap discipline.
- **[Single-File Aggregated LLM Bundle](llms-full.txt)**: Combined, unabridged specification text for instant LLM prompt context ingestion.

---

## 📄 License
Licensed under the [VWL Community Source-Available License](LICENSE) (Usage & Local Evaluation Permitted, No-Distribution, No-Resale). Copyright &copy; 2026 horia-ai / Alex Trifan. All rights reserved.
