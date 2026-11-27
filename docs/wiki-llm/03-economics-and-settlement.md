# VWL Economics & Settlement v1.1
## REP, WORK Wages, Project Accounts, Vesting, Funding, Liquidity, and Smart Contracts

**Status:** authoritative economic design aligned to VWL Protocol v2.2  
**Supersedes:** Economics & Settlement v1.0  
**Protocol dependency:** VWL v2.2  

# 1. Compatibility verdict

The economic model is preserved:

- REP is non-transferable, non-purchasable, domain-specific, versioned/recomputable and outcome-derived.
- REP is used for qualification/discovery, not technical authority.
- REP is never bid or spent.
- WORK remains the negotiated wage/economic entitlement.
- Project capital funds/reserves WORK; Engagements define compensation and vesting.
- VWL technical evidence determines whether work-related economic conditions are satisfied.
- WORK/REP can never cause a failing WorkUnit to become technically CONFIRMED.

v2.2 changes the **quality and shape of evidence consumed by economics**, not the core wage model.

# 2. New evidence interface

Economics v1.1 consumes these v2.2 objects:

```text
Task / Engagement scope
WorkUnit (bounded tree effect; may contain multiple commits)
VerificationPlan
Proofs
Decision
CompositionManifest / canonical effect
Outcome
Attribution
Impact state
Repometer/evidence confidence
```

Economic contracts MUST NOT assume `WorkUnit == one commit`.

# 3. REP relationship

REP scoring remains a separate versioned specification. Economics only requires these invariants:

```text
REP cannot be purchased
REP cannot be transferred
REP cannot be spent as a bid
REP cannot grant technical authority
REP must expose evidence/version/confidence
REP may change when later Outcomes provide new evidence
```

Repometer is NOT REP. Repometer answers how much of current canonical state is supported by surviving evidence. REP answers what capability a Node has demonstrated.

Market UI SHOULD show both where useful:

```text
Rust REP: 8,240
Evidence confidence: HIGH
Comparable WUs: 491
Current evidence coverage: 61% independently verified
```

# 4. WORK relationship

WORK remains wage, not reputation.

```text
Project funds work
→ signed Engagement
→ WORK reserved
→ technical contribution produces VWL evidence
→ Engagement earning condition satisfied
→ WORK earned
→ vesting schedule
→ liquid WORK
```

The Engagement MUST identify the exact settlement predicate/version. Examples:

```yaml
settlement:
  earn_on:
    decision: confirmed
    canonical_effect: required
    composition: passed
```

For higher-risk roles it MAY require additional evidence classes, but economic policy MUST NOT invent technical truth.

# 5. Historical confirmation vs later regression

A v2.2 CONFIRMED Decision is historical and is not revoked merely because a later regression is discovered.

Therefore economics MUST distinguish:

```text
EARN EVENT
= conditions satisfied at canonicalization time

LATER OUTCOME
= new evidence after earning
```

A later Outcome MAY affect only explicitly defined future/unvested components and REP, subject to sufficient Attribution confidence. It MUST NOT silently claw back already-final compensation unless the Engagement explicitly defined a legally/technically valid clawback mechanism.

Default recommendation:

```text
accepted/canonical tranche → final when earned
30/90/180-day tail → outcome-sensitive
```

# 6. Attribution threshold

Economic consequence requires a configured minimum attribution class.

```text
HIGH    → may affect outcome-sensitive unvested tail
MEDIUM  → review/escalation by default
LOW     → no automatic economic penalty
UNKNOWN → no automatic economic penalty
```

Observation and blame are separate.

# 7. Impact cone and descendants

A regression attributed to WU-A does not automatically economically penalize every descendant Node.

Only WorkUnits with their own sufficient Attribution/evidence can receive an economic consequence. Impact state (`REVALIDATE`, `INVALIDATED`, etc.) triggers technical revalidation first.

# 8. Validator economics

Validator Nodes may be paid for verification work, but payment MUST NOT be proportional to PASS/FAIL count.

Good settlement bases include:

```text
required gate completed with valid evidence
reproducer accepted
verification artifact reproducible
verification obligation completed
```

REP may later measure unique valid catches/false-challenge rate, but economics must not incentivize manufacturing FAIL verdicts.

# 9. Multi-actor WorkUnit economics

Internal subagent commits do not automatically create separate wage claims. The Engagement defines economic actors and allocation.

A WorkUnit can contain commits from test/integration/E2E/implementation subagents while remaining one atomic technical candidate.

If subagents are separately compensated, their entitlements MUST be explicit Engagement/sub-Engagement records rather than inferred from commit count.

# 10. Repometer and market qualification

Repometer MAY be used as a qualification/confidence filter:

```text
Rust REP >= 3000
AND evidence confidence >= MEDIUM
AND comparable current evidence >= threshold
```

But repository coverage does not equal Node quality. High Repometer coverage cannot manufacture REP.

# 11. Speculative work

Speculative WorkUnits cannot trigger final earning based solely on speculative completion. Economic earning requires the Engagement's canonical/verification predicate.

Projects MAY compensate explicitly contracted speculative work, but that is a separate commercial term and not implied by VWL confirmation.

# 12. No change to core tokenomics

The following remain unchanged from v1.0 unless separately revised:

```text
initial E0/E1 denomination: 1 stablecoin unit = 1,000 WORK
REP not spent
WORK wage negotiation
Project smart account / milestone budget model
reservation before economic commitment
vesting tail
repository disappearance does not cancel contractual compensation
public liquidity optional and later-stage
```

# 13. Adversarial review

**Attack:** More gates means workers earn more REP/WORK.  
**Resolution:** no. Gate count is not an economic or REP multiplier. Evidence quality/outcomes are separate.

**Attack:** Validator can fail a WU to hurt a competitor's vesting.  
**Resolution:** verifier verdict is only Proof; deterministic Decision and later Attribution rules govern effects. Validator REP/economics penalize false challenges separately.

**Attack:** Regression years later can confiscate wages.  
**Resolution:** historical earning remains; only explicitly outcome-sensitive unvested terms may react, under attribution threshold.

**Attack:** Internal test subagent demands a share because it has a commit.  
**Resolution:** commit provenance is not an economic entitlement. Engagements define entitlement.

**Attack:** Repometer becomes a second reputation currency.  
**Resolution:** prohibited conceptually; Repometer is evidence coverage/confidence, not transferable/scored economic value.

**Attack:** Project requires impossible verification to avoid paying.  
**Resolution:** VerificationPlan/settlement predicate is bound by Engagement/policy version; post-hoc relaxation/tightening cannot rewrite signed terms.

**Attack:** A Project changes gate policy after work begins.  
**Resolution:** Engagement pins policy/VerificationPlan derivation version; material amendments require explicit agreement.

**Attack:** Descendant failure causes cascading wage penalties.  
**Resolution:** impact triggers revalidation; economic consequence requires direct sufficient attribution.

**Residual risks:** legal enforceability of vesting/clawback terms, token classification, collusion, oracle/attestation integrity, and market behavior remain external/legal/security concerns.

# 14. Final invariant

> VWL technical truth flows one way into economics. Economics may reserve, earn and vest compensation based on versioned technical evidence, but money, REP, market demand and token state can never manufacture a technical PASS. Later Outcomes append evidence; economic consequences are bounded by signed Engagement terms and attribution confidence.
