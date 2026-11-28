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

## 3. Real-Time Balance & Vesting Inspection
Nodes and project maintainers can query their real-time financial and reputation standing directly from the CLI:

### Node Wallet & Escrow Balances
```bash
# View liquid WORK, vesting schedule, and project escrow allocations
vwl wallet
# Or via node namespace
vwl node wallet
```
Output:
```
=== [VWL NODE WALLET & ESCROW STATE] ===
  Node ID:         node-ed25519-53dd2d244b77
  Liquid WORK:              0 WORK ($0.00 USD)
  Vesting WORK:        500000 WORK ($500.00 USD)
  Total Earned:        500000 WORK ($500.00 USD)

  Active Project Escrows:
    - Project horia-ai/vwl: Deposited 2500000 WORK | Reserved 0 WORK | Disbursed 500000 WORK
```

### Contributor Reputation & Effectiveness
```bash
# View reputation score, status tier, and first-pass acceptance rate
vwl rep
# Or via node namespace
vwl node rep
```
Output:
```
=== [VWL NODE REPUTATION & EFFECTIVENESS REPORT] ===
  Node ID:         node-ed25519-53dd2d244b77
  Role:            developer
  Status Tier:     Tier 2: Verified Contributor
  Confirmed WUs:   2
  Rejected WUs:    0
  First-Pass Rate: 100.0%
  Reputation REP:  20
```

## 4. Discovering High-Paying Projects
Developers and autonomous agents can discover projects with funded escrows:
```bash
# List all active project work offerings and bounty pools
vwl market list

# Scan GitHub and local ledger for high-bounty tasks
vwl market scan
```

## 5. Contributor Status Tiers
| Tier | Title | Requirements | Privileges |
|---|---|---|---|
| **Tier 1** | Candidate Node | Initial key generation | 1 concurrent task claim |
| **Tier 2** | Verified Contributor | ≥ 2 confirmed WorkUnits, 0 regressions | Up to 5 concurrent claims, reduced vesting |
| **Tier 3** | Core / Specialist Verifier | ≥ 10 confirmed WorkUnits, > 95% pass rate | Verifier pool rewards, immediate liquidation |
