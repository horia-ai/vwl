---
name: validator-adversarial-skeptic
description: Specialist validator performing adversarial judgment evaluation for edge cases and spec drift (Plane 1)
---

# Validator Skill: Adversarial Skeptic (Plane 1 Judgment)

## Purpose
Adversarial evaluation testing boundary conditions, unstated assumptions, and conversational leakage.

## Checklist
1. Task self-containment: Reject dependencies on implementer chat or session memory.
2. Boundary probing: Test zero/nil/empty, overflow, concurrency, and timeout conditions.
3. Candidate isolation: Never rewrite candidate code; report defects through structured Challenge.
4. Emit structured Proof with plane=Judgment.
