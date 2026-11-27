# Agent Instructions: Virtual Work Ledger (VWL) Protocol

This repository is governed by the Virtual Work Ledger (VWL) decentralized labor and evidence protocol.

## Step 0: Initial Repository Setup & Gate Verification
Whenever an autonomous agent starts or resumes work in this repository:
1. **Verify VWL Readiness**:
   - Check that `.vwl/gates.yaml` exists.
   - Check that `.git/hooks/pre-commit` is installed and executable.
2. **If VWL is Uninitialized or Gates are Missing**:
   - Run `npx @horia-ai/vwl bootstrap` (or `vwl bootstrap --commit`).
   - Run `npx @horia-ai/vwl verify precommit` (or `cargo test --workspace`) to confirm all static validator gates pass on the clean base.
   - Commit the baseline bootstrap configuration before starting any task code.
3. **Task Execution Protocol**:
   - Inspect the current Work Frontier: `vwl frontier` (or `npx @horia-ai/vwl status`).
   - Claim your assigned task: `vwl claim <task-id>`.
   - Work in an isolated Git worktree (`vwl work <task-id>`).
   - Enforce strict TDD (Red-Green-Refactor) and Stryker mutation testing (>=80%).
   - Pass all static pre-commit gates (`vwl verify precommit`).
   - Submit candidate diff and proofs via `vwl submit`.
