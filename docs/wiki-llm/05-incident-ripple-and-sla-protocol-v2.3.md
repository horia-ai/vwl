# VWL v2.3 — Incident Ripple, Roll-Forward, Evidence Restoration & SLA Protocol Addendum

**Status:** Authoritative; supersedes VWL v2.2 incident/revalidation semantics. All unaffected v2.2 rules remain normative.

## 1. Core change

A regression propagates **uncertainty**, not retroactive rejection.

```text
CONFIRMED W1
→ later Outcome(REGRESSION)
→ Attribution
→ Impact Frontier
→ contain/restore service
→ repair root
→ revalidate immediate affected frontier
→ stop below any node requiring code repair
→ repair that node
→ recompute frontier
→ continue in waves
```

Git ancestry alone is not semantic dependency.

## 2. Incident states

```text
CLEAN
SUSPECT
REVALIDATE_REQUIRED
REPAIR_REQUIRED
WAITING_ON_REPAIR
INVALIDATED
REVALIDATED
```

Historical `CONFIRMED` remains historical truth.

## 3. Two restoration clocks

VWL MUST distinguish:

- **Service Restoration Time (SRT):** detection → service returned to acceptable SLO, possibly via rollback/traffic isolation/feature disable/safe roll-forward.
- **Evidence Restoration Time (ERT):** detection → affected evidence cone is repaired/revalidated and canonical confidence restored.

Restoring service does not falsely mark stale evidence valid.

## 4. Containment

On regression detection:

1. append Outcome;
2. preserve historical Decision;
3. assess severity;
4. identify candidate attribution;
5. derive conservative initial impact frontier;
6. contain service using configured policy:
   - traffic isolation;
   - feature disable;
   - rollback to previously known-safe deployable artifact;
   - or forward hotfix;
7. block new authority through affected uncertain paths.

Rollback is an operational containment action, not Git-history rewriting.

## 5. Wave/barrier algorithm

For an attributed root W1:

```text
repair W1
→ verify/canonicalize repair
→ derive immediate affected frontier F1
→ validate F1 in parallel

for each member:
  PASS       → REVALIDATED; stop that branch unless deeper independent stale Proof exists
  UNAFFECTED → stop branch
  UNKNOWN    → conservative policy / broader evidence
  FAIL but no code change → rerun/escalate required evidence
  CODE CHANGE REQUIRED → REPAIR_REQUIRED; BARRIER below that node
```

When W2 is `REPAIR_REQUIRED`, descendants behind W2 become `WAITING_ON_REPAIR`. Do not perform authoritative deep validation against a state already known to be changing.

After Repair-W2 becomes canonical:

```text
recompute impact frontier below W2
→ validate next wave
```

Cheap static impact planning MAY run behind the barrier while repair is in progress.

## 6. Proof invalidation

Proofs SHOULD declare relevant premises:

```yaml
proof:
  subject: W2
  premises:
    spec: S7
    interface: API-X
    environment: ENV4
    canonical_effects: [W1]
```

If Repair-W1 changes API-X, dependent Proofs become `STALE`; unrelated Proofs may remain reusable.

RevalidationPlan reruns only evidence whose premises changed plus mandatory policy gates.

## 7. Rollback versus roll-forward

Default preference:
- restore user-facing service by the safest fastest configured containment;
- preserve append-only Git/VWL authority history;
- produce forward Repair WorkUnits for permanent canonical correction.

A deployment rollback may point runtime traffic at an earlier artifact while canonical engineering history remains append-only.

## 8. Node REP effect

Ripple membership alone MUST NOT reduce Node REP.

```text
descendant becomes SUSPECT/STALE
≠ negative REP
```

REP consequence requires the Node's own attributable work to receive sufficient negative evidence or fail revalidation under the REP specification.

A high-REP Node may lose REP after a serious attributable regression, but penalty MUST be:
- attribution-confidence weighted;
- severity/domain weighted;
- bounded against evidence volume/history;
- capable of later recovery;
- separate from temporary confidence/staleness.

One incident MUST NOT erase years of evidence merely because the Node has high REP.

## 9. Validator and Skill attribution

Incident review records which gates had jurisdiction before confirmation:

```text
required gate?
executed?
PASS/FAIL/INCONCLUSIVE?
validator identity/config epoch?
Skill/config commitment?
could the gate reasonably have detected this failure class?
```

A validator is not penalized merely because a later bug exists. Negative validator evidence requires a demonstrable miss inside its declared verification obligation.

A Skill is not blamed merely because it was installed/used. Skill/config attribution requires evidence that it materially contributed to the failure or failed a declared obligation.

## 10. SLA/SLO metrics

A Node/project/service may publish SLO/SLA claims only with an explicit denominator/window.

Example availability:

```text
availability = good service minutes / eligible service minutes
```

99.9% is a service metric, not a REP number.

Track separately:
- MTTD;
- containment time;
- Service Restoration Time;
- root repair time;
- Evidence Restoration Time;
- impact-cone size;
- revalidation fanout;
- repair depth/waves;
- stale-proof count;
- revalidation pass rate;
- validator escape rate;
- recurrence rate;
- attribution confidence;
- customer-visible bad minutes.

A Node's attributed incident history may inform REP, but SLA is not mechanically identical to REP.

## 11. Deployment release gates

Before deployment:
```text
canonical effect
→ deployable artifact
→ release VerificationPlan
→ rollout policy
```

After deployment:
```text
canary/shadow
→ telemetry
→ SLO check
→ widen rollout OR contain
```

Incident repair can use:
```text
rollback runtime
OR safe roll-forward
```
while canonical repair/evidence restoration proceeds.

## 12. Metrics attribution

For each incident preserve:

```yaml
incident:
  outcome: O-...
  service: ...
  detected_at: ...
  severity: ...
  root_attribution:
    work_unit: ...
    confidence: HIGH
  affected_proofs: [...]
  validators:
    - gate: integration
      obligation: ...
      result_at_confirmation: pass
      later_assessment: miss|not_applicable|unknown
  skills:
    - commitment: ...
      assessment: contributed|not_applicable|unknown
  srt: ...
  ert: ...
  repair_waves: ...
```

## 13. Adversarial review

**Attack: Ripple invalidates the entire tree and incident recovery becomes slower than CI.**  
Resolution: wave/barrier traversal, parallel frontier validation, premise-level Proof staleness, branch stop on PASS/UNAFFECTED.

**Attack: Stopping below W2 hides W4 defects.**  
Resolution: it delays authoritative validation only until Repair-W2 establishes the state W4 must actually be judged against; static impact planning may proceed meanwhile.

**Attack: Roll-forward is slower than rollback during outage.**  
Resolution: service containment and canonical repair are separate. Runtime rollback may restore service first.

**Attack: Rollback rewrites canonical history.**  
Resolution: runtime deployment pointer changes; Git/VWL history does not rewind.

**Attack: A huge impact cone misses 99.9% SLA.**  
Resolution: SLA measures service bad time, not evidence-restoration completion. Containment can restore service before ERT.

**Attack: High-REP Node loses everything after one incident.**  
Resolution: ripple staleness is not negative REP; only sufficiently attributed negative outcomes affect REP, bounded by history/severity/confidence.

**Attack: High REP makes a Node immune to new failures.**  
Resolution: historical evidence dampens volatility but cannot suppress current attributable negative evidence.

**Attack: Validator is blamed for every escaped bug.**  
Resolution: validator miss requires that failure class fell within the gate's declared obligation and was detectable under its visibility/environment.

**Attack: Skill author is blamed because Skill was present.**  
Resolution: usage correlation is not causal attribution.

**Attack: Minimal revalidation reuses a Proof whose hidden premise changed.**  
Resolution: premise declarations + conservative invalidation; unknown premise relationships trigger broader plan.

**Attack: Repair waves oscillate forever.**  
Resolution: bound automated repair depth/attempts in configuration; escalation occurs after policy threshold while preserving evidence.

No further internal authority contradiction identified. Residual risk remains in attribution accuracy, dependency/Proof premise quality, deployment tooling, and real gate latency.

## 14. One-year synthetic incident simulation

Scenario only; not a forecast and no monetary/model cost assumptions.

```text
WorkUnits produced                         2487
Services                                     12
Nodes                                        18
Incidents                                    30

Median MTTD                              3.02 min
P90 MTTD                                 6.45 min
Median service restoration              12.65 min
Median evidence restoration             18.78 min
P90 evidence restoration                30.11 min

Average impact cone                     9.37 WUs
P90 impact cone                         18 WUs
Descendant revalidations                281
Descendant code repairs                 28
Branches passing revalidation           246
Unknown branches                        7

Incidents without proven root blame     10
Validator-miss incidents                3
Skill/config investigation flags        4

Median simulated service availability   99.99437%
Worst simulated service availability    99.98164%
```

The scenario deliberately distinguishes stale evidence from negative Node REP. All 18 Nodes acquired some stale evidence during ripple analysis, while only 10 received attributable negative root evidence in this run.

Interpretation: wave/barrier validation prevented automatic deep work behind nodes already requiring repair; service restoration preceded complete evidence restoration; no ripple membership alone created negative REP.

## 15. New falsification tests

1. Root regression with 50+ descendants must not force unconditional full-tree rebuild.
2. W2 requires repair → W2 descendants wait until repair canonical, then frontier recomputed.
3. Runtime rollback restores service without rewriting Git/VWL canonical history.
4. Proof premise changed → Proof becomes STALE.
5. Unrelated Proof premise unchanged → eligible for reuse.
6. Descendant ripple membership alone → zero negative REP.
7. High-confidence root regression → bounded REP consequence under REP policy.
8. Validator outside failure-class obligation → no validator penalty.
9. Validator obligated/detectable miss → negative verifier evidence.
10. 99.9% availability calculation uses explicit service/window denominator.
11. SRT and ERT are independently measured.
12. Fresh replay reconstructs incident waves, barriers, stale Proofs and restoration metrics.

## 16. Final invariant

> Incident handling is a forward-moving evidence repair process. VWL restores service as safely and quickly as possible, then restores confidence through parallel impact-frontier waves. A branch requiring code repair becomes a barrier; deeper authoritative revalidation waits for that repair. Ripple membership creates uncertainty, not blame. REP, validator quality, Skill quality and SLA consequences require their own attributable evidence.
