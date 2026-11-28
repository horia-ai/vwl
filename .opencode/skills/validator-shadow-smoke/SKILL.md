---
name: validator-shadow-smoke
description: Specialist validator executing read-only shadow traffic and isolated deployment smoke verification (Gate 18 & 19)
---

# Validator Skill: Shadow Traffic & Smoke Contract (Gate 18 & 19)

## Purpose
Validate candidate in containerized smoke/deployment and mirrored traffic without production state mutation.

## Safety Invariants
1. Read-only execution: Mutation calls MUST be rejected or mocked.
2. Side-effect suppression: External webhooks, emails, and database writes suppressed.
3. Bounded execution: Enforce request limit and duration timeouts.
4. Emit structured Proof with gate='deployment-shadow'.
