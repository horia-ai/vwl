## VWL Work Unit Pull Request

### 1. Work Unit Details
- **Work Unit ID:** `WU-G...`
- **Milestone / Gate:** Gate ...
- **Hypothesis Tested:** ...
- **Base Commit:** `...`
- **Resulting Commit:** `...`

---

### 2. TDD & Mutation Testing Protocol Checklist
- [ ] **Phase 1: RED** — Unit/integration tests written first and verified failing.
- [ ] **Phase 2: GREEN** — Minimal production implementation written; all tests pass cleanly.
- [ ] **Phase 3: REFACTOR** — Strict formatting, zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`), and typecheck exit 0.
- [ ] **Phase 4: MUTATE** — Stryker mutation testing executed on modified TypeScript code with **score $\ge 80\%$** (`npm run test:mutate`).
- [ ] **Zero Surviving Mutants** on core boundary/decision logic.

---

### 3. Verification Commands Run
```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
npm test
npm run test:mutate
```

### 4. Known Limitations & Deferred Work
<!-- State strictly what this Work Unit does NOT attempt to do -->
