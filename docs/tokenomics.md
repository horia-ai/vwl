# VWL Economics & Settlement Invariants

## 1. The WORK Token State Machine

WORK tokens transition through explicit, conserved accounting states:

```
[Simulated / Fiat Purchase]
           │
           ▼
     [ LIQUID ] ──────────┐
           │              │
    (Project Fund)   (Transfer / Reinvest)
           │              │
           ▼              │
  [ PROJECT_LOCKED ]      │
           │              │
 (Milestone Allocation)   │
           │              │
           ▼              │
     [ RESERVED ]         │
           │              │
 (Confirmed Attestation)  │
           │              │
           ▼              │
    [ EARNED / VESTING ] ─┘
           │
     (Vesting Claim)
           │
           ▼
     [ LIQUID ]
```

## 2. Invariants

1. **Conservation of WORK**:
   $$\text{Total WORK} = \text{Liquid} + \text{ProjectLocked} + \text{Reserved} + \text{Earned} + \text{Vesting}$$
   No transition creates or destroys WORK tokens outside explicit funded issuance.

2. **Atomic Milestone Reservation**:
   Milestone budgets cannot be over-allocated. When an Engagement is dual-signed, the required WORK is atomically decremented from the Milestone available balance and locked in Escrow:
   $$\sum \text{Reserved}_{\text{Engagement}} \le \text{Milestone Budget}$$

3. **Settlement Nonce Idempotency**:
   Every settlement attestation contains a monotonic, unique nonce. Duplicate submissions are strictly rejected, preventing double-spending of work compensation.

4. **Fiat Onramp (Gate 9)**:
   - Fixed baseline reference: 1.00 USD = 1,000 WORK units (1 USD cent = 10 WORK units).
   - Incoming fiat payments via Stripe MCP generate cryptographically verifiable purchase receipts.
