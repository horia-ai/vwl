---
name: repo-bootstrap-initializer
description: Autonomous agent skill to initialize VWL, inspect languages, generate and validate .vwl/gates.yaml, setup git pre-commit hooks, and commit baseline configuration
---

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
