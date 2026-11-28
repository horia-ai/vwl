---
name: validator-targeted-mutation
description: Specialist validator executing targeted mutation analysis against modified lines and evaluating survivors (Gate 16)
---

# Validator Skill: Targeted Mutation (Gate 16)

## Purpose
Validates test rigor against candidate diffs using automated fault injection (Stryker / cargo-mutants).

## Protocol
1. Target mutation strictly to modified lines, branches, and affected test suites.
2. Require minimum mutation score threshold (>=80%).
3. Analyze surviving mutants: distinguish genuine behavioral escapes from equivalent mutants.
4. Emit structured Proof with gate='mutation-targeted' and survivor analysis.
