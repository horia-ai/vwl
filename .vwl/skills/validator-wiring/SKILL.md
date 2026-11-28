---
name: validator-wiring
description: Specialist validator proving real runtime path reachability, route registration, and module linkage (Gate 14)
---

# Validator Skill: Wiring Proof (Gate 14)

## Purpose
Proves implementation is actively wired and reachable through the intended runtime path, not merely invoked in isolated synthetic unit tests.

## Verification Rules
1. Inspect candidate diff for real wiring points (route registration, dependency injection, CLI commands, event listeners).
2. Execute end-to-end wiring tests or symbol link checks.
3. Fail if behavior exists in isolation without active integration.
4. Emit structured Proof with gate='wiring' and plane=Deterministic.
