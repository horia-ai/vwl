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

## Phase 1: Interactive Spec Writing & Adversarial Grilling
Whenever the user asks you to design, plan, or implement a new feature or task:
Do NOT jump straight to code or create vague tasks. Adopt the **Adversarial Spec Skeptic** persona:
1. **Grill the User with Rigorous Questions**:
   - **Falsifiability**: What exact CLI command or automated test assertion proves this task is done?
   - **Invariants**: What system invariants or data constraints must NEVER be broken?
   - **Edge Cases & Failure Modes**: What happens on empty/null input, network timeout, error states, and concurrency?
   - **Error Semantics**: What exact error codes or types should be returned?
   - **Base Commit**: What is the canonical base Git commit SHA (use HEAD)?
2. **Reject Vague or Conversational Requirements**:
   - Reject subjective criteria: *"clean code"*, *"looks good"*, *"fast enough"*, *"well tested"*, *"high quality"*.
   - Reject conversational dependencies: *"as discussed"*, *"like we agreed"*, *"in our chat"*.
3. **Generate & Ingest Immutable Task Specification**:
   Write the specification to `.vwl/ledger/tasks/<task-id>.yaml`:
   ```yaml
   id: "<task-id>"
   objective: "<Self-contained objective statement>"
   acceptance:
     - "<exact-test-or-verification-command>"
   base: "<canonical-git-base-sha>"
   ```
   Formally validate and ingest with the Skeptic Review Gate:
   ```bash
   vwl task add .vwl/ledger/tasks/<task-id>.yaml --skeptic
   ```
   Link dependencies if required:
   ```bash
   vwl task add-dep <parent-task> <task-id> --kind requires
   ```

## Phase 2: Task Execution & Worktree Isolation
When implementing an admitted task:
1. **Inspect Frontier**: Check ready tasks: `vwl frontier` (or `npx @horia-ai/vwl status`).
2. **Claim Task**: Claim your assigned task: `vwl claim <task-id>`.
3. **Isolated Worktree**: Create and enter isolated worktree: `vwl work <task-id>` (at `.vwl/worktrees/<task-id>`).
4. **Strict TDD & Mutation Rigor**:
   - Red: Write failing test first.
   - Green: Write minimal implementation to pass.
   - Refactor: Clean types and formatting.
   - Mutation Testing: Ensure mutation score >= 80%.
5. **Validator Verification**: Pass all pre-commit validator gates: `vwl verify precommit`.
6. **Git Commit & Submission**: Commit your changes in git and submit candidate diff:
   ```bash
   vwl submit <task-id> --candidate <head-sha>
   ```

## Phase 3: Independent Verification
The Verifier runs in a fresh context with **Zero Implementer Chat** (0 conversational tokens), executes `.vwl/gates.yaml`, and appends an immutable `Proof` and `Decision` to the ledger.
