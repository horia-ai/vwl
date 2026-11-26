# VWL Architecture: Separation of Concerns

The Virtual Work Ledger (VWL) is structured around strict separation between the **Mutable Discovery Plane**, the **Cryptographic Authoritative Plane**, and the **Financial Settlement Plane**.

```
┌────────────────────────────────────────────────────────┐
│             Mutable Discovery Plane (GitHub)           │
│  - Issues / Topics (topic:vwl-project, vwl-role)      │
│  - PRs, Comments, Projections                          │
│  * ZERO Financial or Authoritative Execution Power     │
└───────────────────────────┬────────────────────────────┘
                            │ Projections & Events
                            ▼
┌────────────────────────────────────────────────────────┐
│           Authoritative Cryptographic Plane            │
│  - Local Append-Only Ledger (.vwl/ledger/)             │
│  - Immutable Task, WorkUnit, Proof, Decision DTOs      │
│  - Ed25519 Signed Objects & Replay Engine              │
└───────────────────────────┬────────────────────────────┘
                            │ Dual-Signed Attestations
                            ▼
┌────────────────────────────────────────────────────────┐
│               Financial Settlement Plane               │
│  - Local Economic State Machine (Gate 5)               │
│  - Smart Contracts: NodeRegistry, Escrow, Vesting      │
│  - Stripe / Fiat Onramp Provider (Gate 9 MCP)          │
└────────────────────────────────────────────────────────┘
```

## 1. Plane Separation Guarantees

1. **GitHub is Projection, Not Authority**:
   - Anyone can edit an issue title or add a label on GitHub.
   - However, GitHub has **zero authority** over VWL state transitions or smart contract escrows.
   - All state transitions require signed cryptographic objects stored in the append-only ledger (`.vwl/ledger/`).

2. **Zero Model Authority**:
   - AI models and LLMs cannot confirm or reject a Work Unit directly.
   - Verification proofs are generated deterministically by running acceptance commands.
   - The Decision Engine evaluates proofs against explicit policy (all required PASS $\to$ CONFIRMED; any FAIL $\to$ REJECTED).

3. **Replay Portability**:
   - Every state transition is reproducible. Running `vwl replay` from a fresh checkout derives the exact canonical state.
