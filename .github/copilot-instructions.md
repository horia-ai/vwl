

## Skill: tdd-mutation-testing
# TDD & Mutation Testing Protocol
1. RED: Failing test first.
2. GREEN: Minimal production code.
3. REFACTOR: Clean code & types.
4. MUTATE: Stryker mutation score >= 80% with zero surviving mutants.


## Skill: bounded-verifier
# Independent Verifier Protocol
1. Receive only Task, acceptance, base commit, and candidate diff.
2. Zero implementer chat delivered.
3. Execute acceptance commands and output structured Proof DTO.


## Skill: economic-conservation
# Economic Conservation Rules
Total WORK = Liquid + ProjectLocked + Reserved + Earned + Vesting.
No unauthorized inflation or discretionary balance changes.


## Skill: dex-liquidity-rules
# DEX Liquidity Rules
1 USD = 1,000 WORK (0.001 USD per WORK).
Public liquidity requires formal verification and decoupled floating rates.


## Skill: repo-bootstrap-initializer
# VWL Repository Bootstrap & Initializer Protocol

## 1. Readiness Detection
Check whether the current repository has VWL initialized and verified:
- If '.vwl/gates.yaml' is missing or baseline gates are unverified, run 'vwl bootstrap --harness <harness>'.
- Verify '.git/hooks/pre-commit' is present and executable.
- Verify node credential exists in '.vwl/identity/credential.yaml'.

## 2. Deterministic Static Gate Validation
- Inspect discovered gates in '.vwl/gates.yaml'. Ensure every precommit gate executes a static, predictable toolchain command.
- Run 'vwl verify precommit' on the current codebase to validate that all manual gates pass on the clean base.
- If any gate fails, adjust '.vwl/gates.yaml' or fix repository baseline defects before proceeding.

## 3. Baseline Commit & Task Execution
- Once all gates pass cleanly, stage and commit the bootstrap files ('git add .vwl .git/hooks/pre-commit && git commit -m "chore(vwl): bootstrap VWL protocol gates and skills"').
- Ingest verified tasks with 'vwl task add <spec.yaml> --skeptic'.
- Launch worker loop with 'vwl worker run --role developer'.


## Skill: validator-wiring
# Validator Skill: Wiring Proof (Gate 14)

## Purpose
Proves implementation is actively wired and reachable through the intended runtime path, not merely invoked in isolated synthetic unit tests.

## Verification Rules
1. Inspect candidate diff for real wiring points (route registration, dependency injection, CLI commands, event listeners).
2. Execute end-to-end wiring tests or symbol link checks.
3. Fail if behavior exists in isolation without active integration.
4. Emit structured Proof with gate='wiring' and plane=Deterministic.


## Skill: validator-test-integrity
# Validator Skill: Test Integrity Gate (Gate 15)

## Purpose
Surfaces and prevents test or oracle relaxation where tests are weakened simply to let an improper candidate pass.

## Detection Invariants
1. Check for deleted or commented-out assertion statements.
2. Flag inflated timeouts, loosened property ranges, or catch-all exception swallows.
3. Verify test assertions check real invariants rather than echo implementation output.
4. Emit structured Proof or Challenge with reproducer artifact if tests are degraded.


## Skill: validator-targeted-mutation
# Validator Skill: Targeted Mutation (Gate 16)

## Purpose
Validates test rigor against candidate diffs using automated fault injection (Stryker / cargo-mutants).

## Protocol
1. Target mutation strictly to modified lines, branches, and affected test suites.
2. Require minimum mutation score threshold (>=80%).
3. Analyze surviving mutants: distinguish genuine behavioral escapes from equivalent mutants.
4. Emit structured Proof with gate='mutation-targeted' and survivor analysis.


## Skill: validator-adversarial-skeptic
# Validator Skill: Adversarial Skeptic (Plane 1 Judgment)

## Purpose
Adversarial evaluation testing boundary conditions, unstated assumptions, and conversational leakage.

## Checklist
1. Task self-containment: Reject dependencies on implementer chat or session memory.
2. Boundary probing: Test zero/nil/empty, overflow, concurrency, and timeout conditions.
3. Candidate isolation: Never rewrite candidate code; report defects through structured Challenge.
4. Emit structured Proof with plane=Judgment.


## Skill: validator-shadow-smoke
# Validator Skill: Shadow Traffic & Smoke Contract (Gate 18 & 19)

## Purpose
Validate candidate in containerized smoke/deployment and mirrored traffic without production state mutation.

## Safety Invariants
1. Read-only execution: Mutation calls MUST be rejected or mocked.
2. Side-effect suppression: External webhooks, emails, and database writes suppressed.
3. Bounded execution: Enforce request limit and duration timeouts.
4. Emit structured Proof with gate='deployment-shadow'.


## Skill: spec-writer-adversarial-skeptic
# Adversarial Spec Writer & Skeptic Protocol (Gate 0)

## 1. Dual-Path Specification Protocol
- **Path A (New Features & Ambiguous Specs)**: Grilling Interview. The agent refuses subjective or conversational drift ('clean code', 'as discussed') and grills the user until falsifiable commands and system invariants are defined.
- **Path B (Reproducible Bugs & GitHub Issues)**: Ingest via `vwl spec from-issue <id>`. If reproducible with clear test commands (`cargo test`, `npm test`), auto-admit via Skeptic self-review with zero human interrogation.
- **Path C (Roll-Forward Regressions)**: Ingest via `vwl spec repair <incident-id>`. Self-reviews the reproducer test and admits to the Work Frontier.

## 2. Invariant Checklist
1. Falsifiability: Deterministic command assertion.
2. Invariants: Explicit data constraints that must not break.
3. Base SHA: Canonical Git base SHA (HEAD).
4. Self-Containment: Zero conversational dependencies.

## 3. Spec Ingestion
Write `.vwl/ledger/tasks/<task_id>.yaml` and admit via `vwl task add <file> --skeptic` or `vwl spec new`.
