# VWL v2.2 — Verified Work Ledger
## Verification Factory, Work Graph, Impact Graph, Repometer, Canonicalization and Replay

**Status:** Authoritative protocol specification  
**Supersedes:** VWL v2.1  
**Implementation:** Rust core; harness/tool adapters configurable  
**Rule:** Agents create candidates and evidence. Deterministic policy grants authority.

## 0. Scope

v2.2 preserves v2.1 Task dependencies, Work Frontier, independent Proofs, deterministic Decisions, composition, canonicalization and replay, and adds:

- deterministic vs judgment verification planes;
- change/risk-derived VerificationPlans;
- WorkUnits as bounded commit sequences/tree effects;
- attributable subagent commits;
- isolated verifier artifacts and Challenge/repair flow;
- black-box verifier visibility profiles;
- Wiring Proof and Test Integrity gates;
- targeted mutation evidence;
- UI/deployment/shadow-production verification;
- Outcomes and regression without historical rewriting;
- Impact Graph and selective revalidation;
- forward-only repair;
- Repometer evidence coverage with code-lineage survival.

Not defined here: WORK economics, REP scoring, token prices, model prices, model routing economics, Skill/MCP economics. These remain downstream/configuration concerns.

## 1. Pipeline

```text
SPEC → TASK DAG → WORK FRONTIER → CLAIM
 → DEVELOPER/SUBAGENTS → WORK UNIT
 → CHANGE/RISK CLASSIFICATION → VERIFICATION PLAN
 → [DETERMINISTIC GATES + JUDGMENT GATES]
 → PROOFS → DETERMINISTIC DECISION
 → COMPOSITION → CANONICAL
 → OBSERVATION → OUTCOME/REGRESSION
 → ATTRIBUTION → IMPACT CONE
 → REVALIDATION / FORWARD REPAIR
```

## 2. Authority invariants

1. GitHub/UI/chat/process memory are not authority.
2. A verifier never mutates the candidate it judges.
3. Model verdicts are Proofs, never direct authority.
4. Decisions are deterministic from policy + authoritative evidence.
5. Rejected work never contaminates canonical state.
6. Individual confirmation does not imply composition validity.
7. Historical Decisions are append-only facts relative to evidence at decision time.
8. Later regression appends an Outcome; it does not rewrite CONFIRMED history.
9. Repair is forward-only by default.
10. READY/BLOCKED are derived and replayable.
11. `requires` normally means confirmed effect present in canonical state.
12. Replay must reproduce authoritative and derived state exactly.
13. REP/economics cannot alter technical truth.
14. Evidence quantity alone never implies evidence quality.

## 3. Four graphs

**Intent Graph:** `Spec → Task → TaskDependency`  
**Provenance Graph:** `Task → WorkUnit → internal commits → actors → tree effect`  
**Evidence Graph:** `WorkUnit → VerificationPlan → Proof/Challenge → Decision → Outcome`  
**Impact Graph:** `canonical effect → behavioral dependents → Outcome → affected cone → revalidation/repair`

They may reference each other but MUST NOT be collapsed into one graph.

## 4. Task DAG and Frontier

Dependency types remain:

```text
requires | after | evidence | conflicts
```

Cycles are rejected before authority.

`requires` checks the required canonical effect, not merely a historical CONFIRMED label.

READY/BLOCKED are derived:

```text
Ready(T,C,P) =
 TaskExists(T)
 AND NotCancelled(T)
 AND DependenciesSatisfied(T,C)
 AND SpecCurrent(T)
 AND BaseSatisfiable(T,C)
 AND ClaimPolicyAllows(T)
 AND PolicyAllows(T,P)
```

The Work Frontier is all Tasks satisfying `Ready`.

GitHub labels may project this state but cannot create it.

## 5. Dependency execution

Default:

```text
A candidate → verify → CONFIRMED → canonical effect → dependent B eligible
```

Dependent work does not normally start from unconfirmed upstream work.

Optional speculative execution may later be enabled. Speculative descendants bind the exact upstream candidate and can never become canonical unless the upstream chain becomes canonical and required revalidation passes.

Independent Tasks may start from the same compatible canonical base; completion order alone does not force rebasing.

## 6. WorkUnit = bounded commit sequence

```yaml
work_unit:
  id: WU-381-A
  task: T-381
  base: C100
  commits:
    - {sha: A1, role: unit-test, actor: N-test}
    - {sha: A2, role: integration-test, actor: N-int}
    - {sha: A3, role: e2e-test, actor: N-e2e}
    - {sha: A4, role: implementation, actor: N-dev}
    - {sha: A5, role: refactor, actor: N-dev}
  candidate: A5
  spec_digest: sha256:...
  dependency_snapshot: sha256:...
```

Authority candidate:

```text
Δ(WU) = tree(candidate) - tree(base)
```

Internal commits are immutable provenance, not independently canonicalized unless promoted to separate Tasks/WUs.

## 7. TDD

Default developer workflow:

```text
RED → GREEN → REFACTOR
```

Developer-produced tests are provenance, not independent Proof.

Policy may require evidence that tests failed before implementation and passed afterward.

## 8. Verifier visibility

VerificationPlans declare enforceable visibility profiles.

- **White-box unit:** Spec + source + tests + diff.
- **Interface integration:** Spec + public interfaces/schemas/connectors; implementation may be hidden.
- **Black-box E2E:** Spec + running public UI/API; source may be unavailable.
- **Adversarial black-box:** declared contract + input/output only.

Prompt-only “do not inspect source” is weaker evidence than environment-enforced restriction and must be labeled accordingly.

## 9. ChangeProfile / RiskProfile

Developer may propose classification but cannot solely determine its gates.

```yaml
change_profile:
  languages: [rust]
  surfaces: [backend, api]
  effects: [persistence-change]
risk_profile:
  data_integrity: high
  security: medium
```

VerificationPlan derives from Spec + actual diff/effect + dependency context + independent/deterministic classification + policy.

Disagreement resolves conservatively or escalates.

## 10. VerificationPlan

```yaml
verification_plan:
  subject: WU-381-A
  required:
    - compile
    - lint
    - unit
    - test-integrity
    - integration-blackbox
    - mutation-targeted
    - wiring
    - adversarial-risk
  conditional:
    - deployment-shadow
```

The plan is durable evidence. Post-failure relaxation requires explicit append-only amendment and rationale; the failed Proof remains.

## 11. Two verification planes

### Deterministic/reproducible

Examples:

```text
format, lint, typecheck, compile, build,
unit, integration, contract, E2E,
schema compatibility, dependency checks,
a11y scanners, container build, smoke,
targeted mutation execution, mechanical wiring checks
```

Tool/version/environment are recorded.

### Judgment/evaluation

Examples:

```text
adversarial skeptic, risk, architecture,
Spec drift, security reasoning, test-quality,
visual screenshot evaluation, behavioral evals,
telemetry interpretation, mutation-survivor interpretation
```

These emit Proofs/confidence/evidence. Their output is not authority by itself.

## 12. Validator artifacts

Validators may create tests, reproducers, fuzz seeds, scripts, screenshots, probes, queries or benchmarks but cannot mutate the candidate.

```yaml
verifier_artifact:
  id: VA-88
  subject: WU-381-A
  verifier: N-V9
  commit: V1
  purpose: reproducer
```

Artifacts are isolated, untrusted evidence until reproduced/validated according to policy.

If validator code should become canonical, it enters through a new/repair WorkUnit and is independently verified.

**Judge cannot silently rewrite defendant.**

## 13. Challenge / repair

```text
WU-A → FAIL → Challenge + verifier artifact
 → fresh repair worker → WU-B → verification again
```

The rejected candidate remains evidence.

## 14. Wiring Proof

First-class gate proving implementation is actually reachable/active through the intended runtime path.

Examples:

```text
route registered; dependency injected; module linked;
feature enabled; migration invoked; event handler subscribed;
UI reachable; intended runtime path exercises behavior
```

Artificial direct invocation is insufficient when the Task requires real wiring.

## 15. Test Integrity Gate

Detects possible oracle relaxation:

```text
deleted/weakened assertions
removed cases
ignored/disabled tests
broader mocks
coverage loss
snapshot regeneration
timeout inflation
exception swallowing
reduced property domains
expected values changed only to match implementation
```

Legitimate changes are allowed when justified by Spec/evidence. The gate surfaces relaxation; policy/judgment determines acceptability.

## 16. Mutation evidence

Target mutation to changed/risk-relevant behavior, lines, branches and tests.

Surviving mutants are evidence, not universal automatic failure. High-risk policy may require thresholds or mandatory review.

## 17. UI verification

```text
build → component/unit → Playwright → a11y
 → screenshot capture/diff → visual judgment
```

Functional and visual evidence remain distinct.

## 18. Deployment verification

```text
artifact → container/package → isolated deploy
 → health → smoke → E2E
 → optional shadow traffic
 → logs/traces/metrics → deployment Proof
```

## 19. Safe shadow/mirrored production validation

Optional high-risk gate. Defaults:

```text
read-only/shadow execution
side-effect suppression
sanitized traffic where required
explicit authorization
bounded request count/time
candidate isolation
no uncontrolled production writes
```

Evidence can compare canonical/candidate responses, latency, errors, logs, traces and Grafana-style metrics.

## 20. Telemetry Proof

Raw observations and interpretation are separate.

Telemetry evidence records source, environment, time window and query sufficient for interpretation.

## 21. Decision

Basic deterministic rule:

```text
required PASS set satisfied + no required FAIL → CONFIRMED
any required FAIL → REJECTED
otherwise → PENDING
```

Judgment Proofs may be required inputs, but policy decides their authority effect.

## 22. Composition

Confirmed WUs are still verified together before canonicalization:

```text
A ✓ + B ✓ → structural/dependency/build/test/semantic composition
```

Failure = `COMPOSITION_CONFLICT`; canonical unchanged.

## 23. Meaning of green

CONFIRMED means:

> Given the Spec, canonical context, VerificationPlan, policy and evidence available at time T, this WorkUnit was sufficiently proven to enter canonical state.

It does not mean eternally correct.

## 24. Outcomes / regression

Later evidence appends:

```yaml
outcome:
  id: O-551
  subject: WU-A
  type: regression
  severity: high
  evidence: [...]
```

`WU-A CONFIRMED @ t1` remains historical truth.

`REGRESSION @ t2` is new truth.

Current canonical health may become `DEGRADED`, `CANONICAL_WITH_KNOWN_REGRESSION`, or `REVALIDATION_REQUIRED`.

## 25. Attribution

Observation and blame are separate.

```text
HIGH | MEDIUM | LOW | UNKNOWN
```

Low/unknown attribution cannot be treated as proven blame by downstream REP/economics.

## 26. Impact cone

A regression in A does not automatically fail all descendants.

Potential states:

```text
UNAFFECTED | REVALIDATE | INVALIDATED | UNKNOWN
```

Impact uses declared dependencies plus behavioral/evidence relationships. UNKNOWN may conservatively trigger revalidation.

## 27. Forward-only repair

Preferred:

```text
current canonical → Repair-A → verify → new canonical
```

Do not normally rewind A and rebase/rewrite all descendants, because that destroys stable WorkUnit/Proof identities and append-only provenance.

Historical reconstruction forks may exist for analysis only.

## 28. Selective revalidation

After a repair, affected existing effects are tested in the new canonical environment.

Pass → `REVALIDATED`.

Fail → create a new repair Task.

Historical WorkUnits are not rewritten.

## 29. Repometer

Repometer measures **evidence coverage/confidence**, not REP quality.

Example:

```text
Ledger-attributed          78%
Independently verified     61%
Unit constrained           72%
Mutation assessed          48%
Integration observed       56%
E2E observed               34%
Wiring verified            63%
Deployment verified        29%
Production observed        21%
Unknown current code       12%
```

Every percentage MUST declare denominator, methodology, target canonical tree and evidence root.

Raw LOC alone is insufficient.

## 30. Evidence units and lineage

Adapters may measure lines, branches, functions, symbols, modules, components, runtime paths or behavioral contracts.

Evidence follows canonical code lineage.

Superseded code does not continue to inflate current coverage.

Repometer distinguishes:

```text
historically evidenced
currently surviving
currently independently verified
currently runtime observed
superseded
unknown
```

## 31. Repometer confidence

May summarize:

```text
HIGH | MEDIUM | LOW | UNKNOWN
```

based on independence, freshness, survival, attribution and evidence policy—not validator count.

Algorithm/methodology is versioned and replayable.

## 32. REP interface

REP is downstream.

VWL exposes WorkUnits, Proofs, Challenges, Decisions, Outcomes, Attribution, coverage, freshness and survival.

Normative:

```text
more validators != automatically more REP
more tests      != automatically more REP
more commits    != automatically more REP
```

REP cannot alter VWL technical truth.

## 33. Economics interface

Economics is downstream.

VWL may expose settlement-relevant evidence, but WORK/USDC/vesting/fees are outside this spec.

Money cannot make failing technical work pass.

## 34. Configuration boundary

Outside authority:

```text
model/provider; reasoning tier; model/token cost;
Skills; MCP selection; routing optimization;
retry budget; parallelism; preferred vendors;
telemetry sampling; UI
```

They may be recorded as provenance/telemetry but cannot rewrite technical truth.

## 35. Replay

Fresh replay reconstructs:

```text
Specs, Tasks, Intent Graph, Claims,
WorkUnits, Provenance Graph,
VerificationPlans, Proofs, Challenges,
VerifierArtifacts, Decisions,
CompositionManifests, CanonicalState,
Outcomes, Attributions,
Evidence Graph, Impact Graph,
READY/BLOCKED, Work Frontier,
Repometer claims for target tree
```

Divergence is hard failure.

## 36. CLI implications

```text
vwl graph intent|provenance|evidence|impact
vwl frontier
vwl task why-blocked <task>
vwl verify plan <wu>
vwl verify run <wu>
vwl challenge show <id>
vwl outcome inspect <id>
vwl impact <outcome>
vwl revalidate <target>
vwl repometer [--tree SHA] [--dimension X] [--explain]
```

## 37. Adversarial review — round 1

- **Developer lies about risk.** Independent/deterministic classification participates; disagreement escalates/conservatively unions gates.
- **Verifier fixes then approves.** Forbidden; verifier artifacts isolated; fix becomes new WU.
- **Multi-commit WU hides intermediate mistakes.** Final tree effect is candidate; immutable internal commits remain inspectable provenance.
- **Subagent commits explode canonical dependencies.** Internal commits are atomic WU provenance.
- **Black-box verifier secretly sees source.** Strong independence requires enforced environment; prompt-only restriction labeled weaker.
- **Mutation testing explodes.** Targeted by change/risk; survivors are evidence.
- **Mirroring damages prod.** Side effects suppressed by default; explicit authorization; bounded execution.
- **Dependent starts from “green” but noncanonical A.** Normal requires waits for canonical effect.
- **Regression means CONFIRMED was false.** No: Decision was evidence-relative; Outcome appends later knowledge.
- **A regression fails all descendants.** No: Impact Graph selectively classifies.
- **Repair rebases descendants and destroys identities.** Forward-only repair.
- **Repometer percentage is meaningless.** Denominator/method/tree/evidence root mandatory.

## 38. Adversarial review — round 2

- **Independent classifier is nondeterministic.** Classification evidence recorded; mandatory deterministic rules remain; uncertainty escalates rather than reducing gates.
- **Plan relaxed after failure.** Amendments append; failed Proof persists; rationale/policy required.
- **Judgment FAIL makes authority nondeterministic.** Judgment is Proof; deterministic policy defines treatment/quorum/escalation.
- **“Deterministic” gate depends on external state.** Environment digest recorded; external observation is not falsely labeled reproducible.
- **Wiring test invokes dead code artificially.** Proof must demonstrate intended entry path.
- **Test Integrity rejects legitimate simplification.** It flags relaxation; Spec/policy/judgment decides legitimacy.
- **Repometer rewards instrumentation over quality.** Repometer measures knowledge coverage only, not quality.
- **Old evidence launders onto rewritten code.** Claims bind target tree/unit lineage; superseded code loses current coverage.
- **Impact analysis is wrong.** Carries evidence/confidence; UNKNOWN triggers conservative policy.
- **Speculation cascades waste.** Disabled by default; bounded by config; no authority until dependencies canonical.

## 39. Adversarial review — round 3

- **Malicious verifier artifact poisons repair.** Artifact is untrusted until reproduced/validated; provenance retained.
- **Developer writes intentionally weak RED tests.** Developer tests are not independent Proof; black-box/mutation/Test Integrity constrain them.
- **Judgment verifiers collude.** Provenance/independence recorded; heterogeneous quorum possible. Collusion remains residual risk.
- **Telemetry finds regression but blame unclear.** Observation and Attribution separate; UNKNOWN is not proven blame.
- **Forward repair leaves bad historical code.** Intentional: history is provenance; current canonical includes corrective effect.
- **Huge monorepo makes Repometer expensive.** Incremental adapters allowed; scalability remains empirical.
- **Gate explosion creates bureaucracy.** Risk-selective VerificationPlans; gate ROI is a falsification metric.
- **Protocol is coupled to specific tools.** Tool names are examples; normative abstraction is gate capability/evidence/visibility/environment.

**Result:** no further internal authority contradiction found after round 3. Residual empirical/security risks remain.

## 40. Residual risks

```text
verifier/model collusion
poor Specs
incorrect risk classification
false-negative tests
production mirroring safety
telemetry quality
impact/attribution accuracy
semantic lineage through large refactors
Repometer scalability
gate cost/latency
dependency overconstraint
toolchain nondeterminism
supply-chain compromise
malicious verifier artifacts
```

These are not claimed solved.

## 41. Required falsification experiments

1. Weak-test attack caught by Test Integrity.
2. Dead-code implementation caught by Wiring Proof.
3. Black-box verifier catches behavior without source.
4. Verifier mutation attempt leaves candidate digest unchanged.
5. FAIL + verifier reproducer supports fresh repair WU.
6. Targeted mutation provides useful evidence.
7. UI functional pass + visual defect caught separately.
8. Local pass + deployment smoke failure.
9. Shadow validation produces evidence without side effects.
10. Regression after CONFIRMED appends Outcome without rewriting Decision.
11. Impact cone revalidates only affected descendants.
12. Forward repair avoids destructive rebase.
13. Superseded code reduces current Repometer coverage.
14. Failed speculative dependency never gains authority.
15. Fresh replay reconstructs all graphs, Frontier and Repometer identically.

## 42. Migration from v2.1

```text
v2.1 one-commit WU
→ v2.2 commit sequence of length 1

v2.1 verification requirements
→ explicit VerificationPlan

v2.1 Proof
→ optional plane/gate/environment metadata

v2.1 later failure
→ Outcome + Attribution + Impact Graph

v2.1 repository evidence
→ Repometer claim bound to canonical tree

v2.1 dependency DAG / Frontier
→ preserved
```

No historical v2.1 Decision is rewritten.

## 43. Implementation WorkUnits

```text
WU-2201 WorkUnit commit-sequence schema
WU-2202 internal actor/provenance
WU-2203 ChangeProfile + RiskProfile
WU-2204 VerificationPlan derivation
WU-2205 deterministic gate interface
WU-2206 judgment gate interface
WU-2207 verifier visibility enforcement
WU-2208 VerifierArtifact isolation
WU-2209 Challenge/reproducer flow
WU-2210 Wiring Proof
WU-2211 Test Integrity
WU-2212 targeted mutation adapter
WU-2213 UI verification profile
WU-2214 deployment verification profile
WU-2215 shadow-traffic safety contract
WU-2216 Outcome
WU-2217 Attribution
WU-2218 Impact Graph
WU-2219 selective revalidation
WU-2220 forward-only repair
WU-2221 Repometer claim
WU-2222 code-lineage survival
WU-2223 current/historical coverage
WU-2224 four-graph replay
WU-2225 adversarial falsification suite
```

## 44. Final statement

> VWL v2.2 is an evidence-governed autonomous-work protocol. Intent is a Task DAG. The Work Frontier determines what may execute. A WorkUnit is a bounded tree effect with attributable internal provenance. Verification is selected from actual change and risk and runs through deterministic and judgment planes. Validators produce evidence but cannot mutate candidates. Deterministic policy decides authority. Confirmed effects are composition-checked before canonicalization. Later evidence appends Outcomes rather than rewriting history; impact is selectively revalidated and repairs move forward. Repometer measures how much current canonical state is actually supported by surviving evidence. A fresh process must reconstruct authority, all four graphs, the Work Frontier and evidence-coverage claims from Git + the VWL ledger.

**Git remembers changes. VWL remembers what was attempted, what was proven, what later failed, what remains covered, and why the current state is trusted.**
