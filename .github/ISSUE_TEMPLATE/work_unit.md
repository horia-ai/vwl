---
name: Work Unit
about: Propose or track an executable VWL Work Unit
title: "WU-G[X]-[YYY] — "
labels: ["type:work-unit"]
---

### Hypothesis
<!-- What falsifiable hypothesis does this Work Unit test? -->

### Objective
<!-- Smallest bounded change satisfying acceptance -->

### Declared Dependencies
- None / WU-G...

### Scope & Constraints
<!-- What is explicitly forbidden or deferred to later gates? -->

### Acceptance Criteria
- [ ] Test 1
- [ ] Test 2

### Verifier Instructions
<!-- Exact acceptance commands to execute in bounded context -->
```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

### STOP Condition
<!-- If this occurs, STOP. Do not engineer around it. -->
