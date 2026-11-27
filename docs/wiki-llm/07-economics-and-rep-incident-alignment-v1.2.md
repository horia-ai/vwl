# VWL Economics & REP Incident Alignment v1.2
## Aligned to VWL Protocol v2.3

**Supersedes:** Economics & Settlement v1.1 incident/REP semantics. Unaffected WORK wage/reservation/vesting rules remain unchanged.

## 1. What does NOT change

```text
REP is non-transferable/non-purchasable
REP is not bid/spent
WORK remains negotiated wage/economic entitlement
Engagement defines earning/vesting
technical truth flows one-way into economics
```

## 2. Regression does not equal automatic REP loss

Three distinct events:

```text
RIPPLE/STALE
Node's work may be affected
→ confidence temporarily stale
→ NO automatic negative REP

REVALIDATION FAIL
Node's own effect fails in repaired canonical context
→ negative evidence
→ REP engine may update according to versioned policy

ATTRIBUTED REGRESSION
Node's own WorkUnit receives sufficient causal attribution
→ negative outcome evidence
→ REP may decrease
```

A high-REP Node is neither immune nor catastrophically reset.

REP policy SHOULD weight:
```text
attribution confidence
severity
domain relevance
recency
evidence volume
prior survival
repair quality/time
recurrence
```

REP changes MUST be versioned/recomputable and explainable to underlying Outcomes.

## 3. Ripple and descendant Nodes

If W1 regresses and W2/W3 merely depended on W1:

```text
W2/W3 → STALE/REVALIDATE
```

This alone does not lower their REP.

If W2 revalidation passes, evidence freshness is restored.

If W2 fails and requires repair, W2's Node receives its own negative evidence according to attribution/revalidation policy.

## 4. Validator REP

A later escaped defect asks:

```text
Which gate should have detected this class?
Was that gate required?
Did it run?
What visibility/environment did it have?
Was the defect detectable then?
```

Only a supported miss becomes negative verifier evidence.

Valid unique catches, reproducible Challenges and low false-challenge rates may strengthen verifier REP.

PASS/FAIL count alone never determines verifier REP.

## 5. Skill/config evidence

Skill/config commitment may be recorded for a WU.

Incident outcomes may classify:
```text
CONTRIBUTED
NOT_APPLICABLE
UNKNOWN
```

Usage alone does not create blame.

Private Skill contents need not be disclosed to record configuration epoch/commitment.

## 6. SLA/SLO

99.9% is not REP.

SLA/SLO metrics require:
```text
service
window
eligible minutes/requests
good/bad definition
exclusions
```

An attributed Node incident may contribute to reliability-domain REP, but service availability and Node reputation remain separate metrics.

## 7. WORK/vesting

No new automatic clawback is introduced by v2.3.

Already-final compensation remains final unless signed Engagement explicitly defines otherwise.

Outcome-sensitive unvested tail may react only under signed terms and sufficient attribution.

Ripple descendants receive no wage/vesting penalty merely for being revalidated.

## 8. Incident metrics for REP/economics

Track:
```text
incidents attributable to Node
severity
customer-visible bad time
repair time
recurrence
revalidation failure/pass
validator misses
false Challenges
Skill/config contribution confidence
```

Do not optimize REP around raw incident count without exposure/denominator.

## 9. Adversarial review

**Attack:** Big-REP Node gets destroyed by one regression.  
Resolution: bounded evidence update weighted against history/exposure; no ripple penalty.

**Attack:** Big-REP Node never loses meaningful REP.  
Resolution: current high-confidence severe evidence still affects relevant domain REP; history dampens volatility, not truth.

**Attack:** Descendant Nodes lose REP because parent failed.  
Resolution: staleness is not negative evidence; own failure/attribution required.

**Attack:** Node improves REP by rapidly repairing its own bugs.  
Resolution: repair quality/time may mitigate but cannot erase originating negative Outcome.

**Attack:** Validators avoid hard gates to protect REP.  
Resolution: obligation completion and misses are measured relative to assigned gate; declining/avoiding obligations is separately visible.

**Attack:** 99.9 SLA becomes marketing fiction.  
Resolution: explicit denominator/window and service telemetry required; REP cannot manufacture SLA.

**Attack:** Project weaponizes outcome-sensitive vesting.  
Resolution: signed predicate + attribution threshold + no descendant cascade.

No further internal economic contradiction identified. REP formula itself remains a separate specification and must be empirically calibrated.
