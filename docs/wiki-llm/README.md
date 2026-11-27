# VWL Wiki-LLM — Authoritative Protocol & Architecture Knowledge Base

Welcome to the **VWL (Verified Work Ledger) Wiki-LLM**. This directory contains the complete, authoritative, and unabridged specifications governing the VWL v2.2 decentralized autonomous work and evidence protocol.

This wiki is specifically curated and indexed for **Large Language Models (LLMs)**, autonomous coding agents (Gemini, OpenCode, Claude, Copilot), verifier harnesses, and protocol engineers.

---

## 📚 Document Architecture & Index

| File | Document Title | Scope & Authority |
| :--- | :--- | :--- |
| **[01-protocol-core-and-verification-factory.md](01-protocol-core-and-verification-factory.md)** | **VWL v2.2 Core Protocol & Verification Factory** | Authoritative protocol specification. Governs the 4-Graph model (Intent, Provenance, Evidence, Impact), Work Units (commit sequences & tree effects), Verification Factory (planes, visibility profiles, shadow contracts), Deterministic Decision Engine, and Replay. |
| **[02-protocol-cli-and-github-projection.md](02-protocol-cli-and-github-projection.md)** | **VWL v2.2 Protocol CLI & GitHub Projection** | Authoritative CLI, TUI, and GitHub WorkHost interaction contract. Covers `vwl` subcommands (`work`, `verify`, `graph`, `repometer`, `outcome`), GitHub issue projection, and Work Frontier synchronization. |
| **[03-economics-and-settlement.md](03-economics-and-settlement.md)** | **VWL Economics & Settlement v1.1** | Authoritative economic design. Governs REP scoring formulas, WORK wage rates, dual-signed Escrow Engagements, Vesting schedules, Smart Contract equivalence (Solidity differential testing), and DEX liquidity. |
| **[04-bootstrap-work-ledger.md](04-bootstrap-work-ledger.md)** | **VWL Bootstrap Work Ledger v1.1** | Authoritative bootstrap execution ledger. Defines the experimental falsification gates (G0 through G10) used to build and verify VWL using VWL's own evidence discipline. |
| **[05-incident-ripple-and-sla-protocol-v2.3.md](05-incident-ripple-and-sla-protocol-v2.3.md)** | **VWL v2.3 Incident Ripple, Roll-Forward & SLA Protocol** | Authoritative v2.3 addendum. Governs the incident state machine (`CLEAN`, `SUSPECT`, `REVALIDATE_REQUIRED`, `REPAIR_REQUIRED`, `WAITING_ON_REPAIR`, `INVALIDATED`, `REVALIDATED`), Proof premise staleness, impact frontier, repair barriers, wave revalidation, runtime containment, SRT vs ERT restoration clocks, and validator obligation. |
| **[06-incident-cli-and-github-ux-v2.3.md](06-incident-cli-and-github-ux-v2.3.md)** | **VWL v2.3 Incident CLI & GitHub UX** | Authoritative v2.3 CLI commands (`vwl incident`, `vwl impact frontier`, `vwl proof stale`, `vwl revalidate frontier`, `vwl repair start`, `vwl sla service`), TUI incident view, and GitHub projection rules. |
| **[07-economics-and-rep-incident-alignment-v1.2.md](07-economics-and-rep-incident-alignment-v1.2.md)** | **VWL Economics & REP Incident Alignment v1.2** | Governs REP consequences during incidents. Upholds that ripple staleness != negative REP, decouples node REP from service SLA/SLO, bounds validator misses to declared verification obligations, and isolates skill/config attribution. |
| **[08-incident-simulation-report-v2.3.md](08-incident-simulation-report-v2.3.md)** | **VWL v2.3 One-Year Incident Simulation Report** | Empirical stress testing report across 2,487 Work Units, 12 services, 18 nodes, and 30 incidents validating SRT/ERT separation, repair barriers, and zero ripple blame. |
| **[09-bootstrap-work-ledger-incident-extension-v1.2.md](09-bootstrap-work-ledger-incident-extension-v1.2.md)** | **VWL Bootstrap Work Ledger v1.2 — Gate 0.8** | Gate 0.8 Work Units (WU-G0.8-001 through WU-G0.8-012) for falsifying the incident ripple and evidence restoration protocol. |

---

## ⚡ Core Invariants (For LLMs & Autonomous Workers)

When implementing, verifying, or auditing any task in the VWL ecosystem, agents MUST uphold these invariant rules:

### 1. Zero-Chat Verifier Invariant
```text
Verifier Input: (Task, Acceptance Criteria, Canonical Base, Candidate Commit/Diff, Test Capability)
Verifier PROHIBITED: Developer conversation history, chain-of-thought, apologies, self-evaluations.
```
*Why:* If verification requires reading the developer's chat logs, the code is not independently verifiable.

### 2. Candidate Isolation Invariant ("Judge cannot silently rewrite defendant")
```text
Verifier Output: Structured Proof OR Adversarial Challenge + Reproducer Artifact.
Verifier PROHIBITED: Directly modifying, fixing, reformatting, or committing code to the candidate branch.
```
*Why:* If the judge fixes the candidate, the author did not produce the work, and the resulting commit provenance is falsified.

### 3. Deterministic Decision Engine
```text
Authority = f(Proofs, Gate Policy).
Model calls produce Proofs (evidence). Models DO NOT decide technical authority.
```
*Rule:*
- All required gates PASS $\rightarrow$ `CONFIRMED`
- Any required gate FAILS $\rightarrow$ `REJECTED`
- Otherwise $\rightarrow$ `PENDING`

### 4. Post-Confirmation Historical Invariant
```text
WU-A CONFIRMED @ t1 is permanent historical technical truth.
A regression observed @ t2 appends an OUTCOME object.
It strictly NEVER mutates or deletes historical Decision records.
```

### 5. Attribution Decoupling Invariant
```text
Outcome Observation != Node Blame.
Attribution carries explicit confidence: High, Medium, Low, Unknown.
```
*Rule:* Low or Unknown confidence cannot be treated as proven fault by downstream REP or economic penalty systems.

### 6. Forward-Only Repair
```text
Regressions DO NOT trigger git rollbacks or ledger rewriting.
Regressions trigger forward-only repair:
New Task -> New Claim -> New Candidate -> Verification -> Confirmed Cutover -> Resolution Outcome.
```

### 7. Repometer Denominator Binding
```text
Repometer evidence coverage MUST declare:
- Target Git Tree SHA
- Denominator LOC
- Declared Methodology
```
*Prohibition:* Raw, ungrounded LOC claims are rejected. Repometer answers how much of the current canonical tree is backed by surviving evidence.

### 8. Pure Functional Replay
```text
Canonical State = Replay(Git Repository + Append-Only Ledger)
```
*Standard:* A clean clone running `vwl replay` must reconstruct all four graphs, active claims, and derived canonical state with zero divergence, without network calls or database servers.

### 9. Incident Ripple Uncertainty Invariant
```text
CONFIRMED W1 -> later Outcome(REGRESSION) -> Ripple propagates UNCERTAINTY, not retroactive rejection.
```
*Rule:* Historical `CONFIRMED` remains historical truth. Ripple membership alone (`SUSPECT`/`STALE`) strictly MUST NOT reduce Node REP.

### 10. Two Restoration Clocks (SRT vs ERT)
```text
SRT: Time to restore user-facing service (containment, rollback, isolate, roll-forward).
ERT: Time to restore complete evidence cone and canonical confidence.
```
*Rule:* Restoring service does not falsely mark stale evidence valid. Operational rollback does not rewrite canonical Git or ledger history.

### 11. Repair Barrier Invariant
```text
If W2 requires code repair, descendants behind W2 become WAITING_ON_REPAIR until Repair-W2 is canonical.
```
*Rule:* Authoritative deep revalidation waits until upstream code repairs become canonical, preventing wasted verification cycles against changing state.

### 12. Validator Obligation Boundary
```text
Negative validator evidence requires a demonstrable escape/miss INSIDE its declared verification obligation and visibility profile.
```

---

## 🛠️ Quick Command Reference

```bash
# Core status & replay
vwl status
vwl replay

# Work Frontier & Graph Inspection
vwl frontier [--json]
vwl graph intent
vwl graph provenance
vwl graph evidence
vwl graph impact <task-id>

# Task Execution & Verification
vwl claim <task-id> --actor <node-id>
vwl work <task-id>
vwl submit <task-id> --candidate <sha>
vwl verify plan <work-unit>
vwl verify run <work-unit> --verifier gemini
vwl verify challenge <challenge-id>

# Evidence Coverage & Post-Confirmation Outcomes
vwl repometer [--explain] [--json]
vwl outcome list
vwl outcome add <id> <subject> --type regression --severity high --evidence "<log>"
vwl outcome impact <outcome-id>

# Incident Ripple & Evidence Restoration (v2.3)
vwl incident show <outcome-id>
vwl incident contain <outcome-id> --strategy rollback|roll-forward|isolate
vwl impact frontier <outcome-id>
vwl proof stale --because <repair-id>
vwl revalidate frontier <outcome-id>
vwl repair start <work-unit>
vwl incident metrics <outcome-id>
vwl sla service <service-id> [--window 30d]

# Economics & Settlement
vwl economics show
vwl economics deposit --project-id horia-ai/vwl --amount-usd 500
vwl economics engage --project-id horia-ai/vwl --node-id <node> --amount-work 50000
vwl report --gh-pages --json
```

---

## 📜 License & Intellectual Property
Use of these specifications and software is governed by the **[VWL Community Source-Available License](../../LICENSE)**:
- ✅ **Permitted:** Viewing, inspection, local compilation, private execution, testing, evaluation, and contributing to the official repository.
- ❌ **Prohibited:** Redistribution, distribution, mirroring, public hosting/SaaS cloning, commercial resale, or licensing for a fee.
