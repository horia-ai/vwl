# VWL Bootstrap Work Ledger v1.1
## Building VWL v2.2 using VWL's own evidence discipline

**Status:** authoritative bootstrap execution ledger  
**Supersedes:** Bootstrap Work Ledger v1.0  
**Target protocol:** VWL v2.2  

# 0. Bootstrap rule

The original Gate-0 remains valid in spirit but its object assumptions changed. We still build the smallest falsifiable core first. We do **not** implement REP/WORK economics before the engineering protocol survives.

The bootstrap itself must exercise the protocol it creates.

# 1. Gate sequence

```text
G0  Core authority + replay
G0.5 Work graph + Frontier
G0.6 Verification factory
G0.7 Outcome/Impact/Repometer
G1  Node + Project identity
G2  GitHub WorkHost/discovery
G3  CLI/TUI productization
G4  REP experiments
G5  Local economics (no blockchain)
G6  Economic/Sybil simulations
G7  Smart-contract equivalence
G8  Wallet/chain UX
G9  Optional fiat onramp
G10 Optional productive liquidity
```

Later gates remain blocked until lower falsification gates pass.

# 2. Updated bootstrap WorkUnit record

Legacy:

```yaml
work_unit: WU-G0-001
base: <sha>
commit: <sha>
```

v1.1 bootstrap:

```yaml
work_unit: WU-G0-001
base: <sha>
commits:
  - sha: <sha>
    role: implementation
    actor: bootstrap-worker
candidate: <sha>
spec_digest: <digest>
dependency_snapshot: <digest>
```

Single-commit bootstrap WUs are valid as sequence length 1.

# 3. Preserve original G0 units

WU-G0-001 through WU-G0-003 remain materially unchanged:

```text
001 Rust workspace/CLI
002 repository identity/canonical base
003 immutable Task DTO
```

# 4. Replace WU-G0-004

## WU-G0-004 — WorkUnit v2.2 DTO

**Hypothesis:** bounded candidate work can be represented as base + ordered attributable commit sequence + candidate tree effect.

Acceptance:

```text
sequence length >= 1
all commits exist
base exists
candidate is final declared candidate
actor/role provenance deterministic
tree-effect digest deterministic
legacy one-commit WU maps losslessly
```

# 5. Update WU-G0-005 — Proof DTO

Add:

```text
gate
plane: deterministic|judgment
environment metadata where applicable
visibility profile where applicable
evidence references
```

Do not implement every gate yet.

# 6. Preserve WU-G0-006 — deterministic Decision

Still fundamental. AI/verifier output remains Proof, never authority.

Add test that judgment Proofs are consumed through deterministic policy.

# 7. Update WU-G0-007/008 — ledger + replay

Replay must eventually support new objects, but first milestone remains:

```text
Task
WorkUnit
Proof
Decision
CanonicalState
```

Then later gates extend replay.

# 8. Preserve WU-G0-009/010 — worktree + developer wrapper

Developer wrapper targets Task and may produce multi-commit WU.

# 9. Replace WU-G0-011 — independent verifier wrapper

Command:

```text
vwl worker verifier <work-unit> -- <harness>
```

not `<commit>`.

Verifier receives bounded judgment context and cannot mutate candidate.

# 10. Preserve parallel/composition experiments

WU-G0-012 through 019 remain conceptually valid with these changes:

- candidates are WUs, not single commits;
- composition operates on tree effects;
- Challenges may reference verifier artifacts;
- Spec drift uses dependency/affected-cone semantics once available.

# 11. New Gate G0.5 — Work Graph / Frontier

```text
WU-G05-001 TaskDependency DTO
WU-G05-002 DAG cycle validation
WU-G05-003 dependency satisfaction
WU-G05-004 derived READY/BLOCKED
WU-G05-005 Work Frontier
WU-G05-006 why-blocked
WU-G05-007 Claim object/contention
WU-G05-008 dependency snapshot in WU
WU-G05-009 stale candidate revalidation
WU-G05-010 Spec affected-cone propagation
WU-G05-011 cancellation/UNSATISFIABLE
WU-G05-012 replay DAG + Frontier
```

Hard stop if readiness cannot be replayed deterministically or cycles enter authority.

# 12. New Gate G0.6 — Verification Factory

```text
WU-G06-001 ChangeProfile/RiskProfile
WU-G06-002 VerificationPlan derivation
WU-G06-003 deterministic gate interface
WU-G06-004 judgment gate interface
WU-G06-005 verifier visibility profiles
WU-G06-006 VerifierArtifact isolation
WU-G06-007 Challenge/reproducer flow
WU-G06-008 Wiring Proof
WU-G06-009 Test Integrity
WU-G06-010 targeted mutation adapter
WU-G06-011 UI verification profile
WU-G06-012 deployment verification profile
WU-G06-013 safe shadow-traffic contract
```

Required adversarial experiments:

```text
weak-test relaxation
dead/unwired code
black-box verifier
verifier mutation attempt
reproducer repair
mutation survivor
UI visual defect
deployment-only defect
safe shadow evidence
```

# 13. New Gate G0.7 — Outcomes / Impact / Repometer

```text
WU-G07-001 Outcome object
WU-G07-002 Attribution object/confidence
WU-G07-003 Impact Graph
WU-G07-004 selective revalidation
WU-G07-005 forward-only repair
WU-G07-006 Repometer evidence claim
WU-G07-007 code-lineage survival
WU-G07-008 current/historical coverage
WU-G07-009 four-graph replay
WU-G07-010 regression-after-CONFIRMED experiment
WU-G07-011 superseded-code coverage experiment
```

Hard stop if later Outcomes require rewriting historical Decisions or if Repometer claims cannot bind a canonical tree/methodology.

# 14. Gate G1 identity

Now identity must cover actors inside a WorkUnit, not merely the WU author. Node/configuration identity remains later than core authority.

# 15. Gate G2 GitHub

GitHub projects:

```text
Task dependency state
Work Frontier projection
Verification summaries
Outcome/repair/revalidation references
```

GitHub remains projection only.

# 16. Gate G3 CLI/TUI

Must expose:

```text
WORK
VERIFY
GRAPH (intent/provenance/evidence/impact)
REPOMETER
```

Market/wallet remain optional later modules.

# 17. Gate G4 REP

REP experiments now consume richer evidence:

```text
independent Proofs
verification diversity
Outcomes
Attribution confidence
survival/freshness
Repometer confidence
```

But REP MUST be tested against simple baselines and MUST NOT reward gate/test/commit counts directly.

# 18. G5+ economics

Economic sequence remains. Update only the evidence adapter to Economics v1.1.

No blockchain before local economic semantics survive.

# 19. Bootstrap skeptic review

**Attack:** v2.2 makes Gate-0 too large to falsify quickly.  
**Resolution:** split G0.5/G0.6/G0.7. Core replay/authority remains first; advanced verification is earned progressively.

**Attack:** changing WorkUnit schema invalidates bootstrap history.  
**Resolution:** one-commit legacy WUs map to v2.2 sequence length 1. No history rewrite.

**Attack:** verifier factory is built before independent verification is proven useful.  
**Resolution:** G0 seeded-bad-WU experiment remains before G0.6 expansion. If independent verification adds negligible value, stop before factory complexity.

**Attack:** Repometer before REP could bias REP design.  
**Resolution:** Repometer measures evidence coverage only; G4 separately tests predictive REP.

**Attack:** production mirroring appears too early.  
**Resolution:** G0.6 defines interface/safety and can use isolated simulation; real production shadowing is not required to pass bootstrap.

**Attack:** dependency/verification features delay first usable binary.  
**Resolution:** G0 remains minimal and executable; new gates are separately blocked.

**Attack:** later economics gets built despite technical failure.  
**Resolution:** G1+ blocked on explicit Go/Stop decisions.

No remaining internal bootstrap sequencing contradiction identified. Residual risk is overbuilding; enforce gate boundaries.

# 20. Updated Go/Stop ladder

```text
G0 PASS:
replay exact; rejected work isolated; verifier catches seeded defect; composition safe

G0.5 PASS:
DAG/readiness/frontier replay exactly; claims deterministic

G0.6 PASS:
verification plans cannot be self-downgraded; verifier cannot mutate candidate; wiring/test-integrity attacks detected in target experiments

G0.7 PASS:
regression appends Outcome; impact selective; forward repair works; Repometer current coverage follows lineage

THEN G1...
```

# 21. Final bootstrap instruction

> Execute only the current Work Unit/Gate. Treat every Gate as a falsifiable experiment. Do not implement future gates to rescue a failed lower-layer hypothesis. Preserve rejected candidates and failed experiments as evidence. One bounded logical change, independent verification, evidence, then stop for authority decision.
