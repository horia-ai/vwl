---
name: validator-test-integrity
description: Specialist validator detecting test/oracle relaxation, weakened assertions, and mock abuse (Gate 15)
---

# Validator Skill: Test Integrity Gate (Gate 15)

## Purpose
Surfaces and prevents test or oracle relaxation where tests are weakened simply to let an improper candidate pass.

## Detection Invariants
1. Check for deleted or commented-out assertion statements.
2. Flag inflated timeouts, loosened property ranges, or catch-all exception swallows.
3. Verify test assertions check real invariants rather than echo implementation output.
4. Emit structured Proof or Challenge with reproducer artifact if tests are degraded.
