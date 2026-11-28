# Agent Instructions: Virtual Work Ledger (VWL) Protocol

This repository is governed by the Virtual Work Ledger (VWL) decentralized labor and evidence protocol.
All autonomous agents (OpenCode, Claude, Antigravity, Copilot) operating in this repository must strictly adhere to this protocol.

## Step 0: Initial Repository Setup & Gate Verification
Whenever an autonomous agent starts or resumes work in this repository:
1. **Verify VWL Readiness**:
   - Check that `.vwl/gates.yaml` exists.
   - Check that `.git/hooks/pre-commit` is installed and executable.
2. **If VWL is Uninitialized or Gates are Missing**:
   - Run `npx @horia-ai/vwl bootstrap` (or `vwl bootstrap --commit`).
   - Run `npx @horia-ai/vwl verify precommit` (or `cargo test --workspace`) to confirm all static validator gates pass on the clean base.
   - Commit the baseline bootstrap configuration before starting any task code.

## Phase 1: Dual-Path Spec Writing & Adversarial Grilling
Whenever you are asked to design, plan, or implement a new feature, bugfix, or issue:
Do NOT jump straight to code or create vague tasks. Follow the dual-path specification protocol:
1. **Path A (New Features & Ambiguous Requests)** — Adopt the **Adversarial Spec Skeptic** persona:
   - **Grill the User**: Ask pointed questions on Falsifiability, Invariants, Edge Cases & Failure Modes, Error Semantics, and Base Commit SHA.
   - **Reject Subjective / Conversational Drift**: Reject criteria like *"clean code"*, *"looks good"*, *"fast enough"*, *"as discussed"*.
   - Create specification: `vwl spec new --id <task-id> --objective "<objective>" -a "<test-cmd>"`
2. **Path B (Reproducible Bugs & GitHub Issues)**:
   - Ingest directly: `vwl spec from-issue <id_or_url>` (or `--issue <id>`).
   - The Spec Writer auto-extracts deterministic reproducer commands (`cargo test`, `npm test`, code blocks), runs self-review with `TaskSkeptic`, and auto-admits to the ledger without human interrogation.
3. **Path C (Roll-Forward Regressions & Incidents)**:
   - Ingest incident: `vwl spec repair <incident-id>`.
   - Generates and admits a forward-repair task targeting the regression root cause.

## Phase 2: Task Execution & WorkUnit Submission
When implementing an admitted task:
1. **Inspect Frontier**: Check ready tasks: `vwl frontier` (or `vwl wu list --unverified`).
2. **Targeting or Claim**:
   - Auto-claim or target a specific task: `vwl dev --task <task-id>`
   - Target a specific issue: `vwl dev --issue <issue-number>`
3. **Isolated Worktree**: Create and enter isolated worktree: `vwl work <task-id>` (at `.vwl/worktrees/<task-id>`).
4. **Strict TDD & Mutation Rigor**:
   - Red: Write failing test first.
   - Green: Write minimal implementation to pass.
   - Refactor: Clean types and formatting.
   - Mutation Testing: Ensure mutation score >= 80%.
5. **Validator Verification**: Pass all non-bypassable pre-commit validator gates: `vwl verify precommit`.
6. **Git Commit & WorkUnit Submission**: Commit atomic candidate changes in Git and submit candidate WorkUnit:
   ```bash
   vwl wu submit <task-id> --candidate <head-sha>
   # Or alias:
   vwl submit <task-id> --candidate <head-sha>
   ```

## Phase 3: Independent Verification (Zero Implementer Chat)
The Verifier runs in a fresh context with **Zero Implementer Chat** (0 conversational tokens):
1. **Continuous or Targeted Mode**:
   - Scan ledger: `vwl verifier`
   - Target specific task: `vwl verifier --task <task-id>`
   - Target specific issue: `vwl verifier --issue <issue-number>`
   - Target specific candidate: `vwl verifier <candidate-sha>`
2. **Verification & Ledger Attestation**:
   - Executes `.vwl/gates.yaml` in read-only isolation.
   - Appends cryptographic `Proof` and `Decision` to the ledger.
   - Inspect confirmed work units: `vwl wu list` and `vwl wu show <commit>`.

## Phase 4: Node Telemetry & Protocol Settlement
- Inspect node balances, liquid WORK, vesting schedules, and project escrows: `vwl wallet` (or `vwl node wallet`).
- Inspect contributor reputation score, tiers, and first-pass acceptance rates: `vwl rep` (or `vwl node rep`).
- Discover high-paying repositories and funded bounties: `vwl market list` and `vwl market scan`.
