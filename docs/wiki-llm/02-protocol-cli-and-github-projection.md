# VWL v2.2 Protocol CLI & GitHub Projection
## CLI/TUI, GitHub WorkHost, Verification Factory, Work Frontier, Repometer and Market Projection

**Status:** authoritative CLI/GitHub interaction contract aligned to VWL Protocol v2.2  
**Supersedes:** VWL v2.0 Protocol_CLI_GitHub  

# 1. Principle

GitHub is collaboration/discovery/projection. VWL ledger + Git objects are technical authority. Smart contracts/economic state, when enabled, are economic authority.

```text
GitHub labels/comments/issues != technical authority
CLI cache != authority
TUI state != authority
```

# 2. Core command surface

```text
vwl init
vwl status
vwl task ...
vwl work <task>
vwl submit <task|workunit>
vwl verify ...
vwl decision ...
vwl cutover ...
vwl replay
```

# 3. Work graph

```text
vwl task deps <task>
vwl task why-blocked <task>
vwl frontier
vwl graph intent
vwl graph provenance
vwl graph evidence
vwl graph impact
vwl claim <task>
vwl release <task>
vwl revalidate <target>
```

`READY/BLOCKED` shown in CLI/TUI/GitHub are derived projections.

# 4. WorkUnit UX

A WorkUnit is no longer assumed to be one commit.

```text
vwl submit T-381
→ base C100
→ commits A1..A5
→ candidate A5
→ tree effect digest
→ actor/provenance summary
```

CLI MUST display both atomic candidate and internal provenance.

# 5. Verification UX

```text
vwl verify plan WU-381-A
vwl verify run WU-381-A
vwl proof show P-991
vwl challenge show CH-88
```

TUI verification view should separate:

```text
DETERMINISTIC
compile ✓
lint ✓
unit ✓
integration ✓
wiring ✓

JUDGMENT
risk ✓
spec-drift ✓
adversarial ?
visual n/a
```

A validator-created reproducer is shown as `VerifierArtifact`, never as candidate mutation.

# 6. Repometer UX

```text
vwl repometer
vwl repometer --tree <sha>
vwl repometer --dimension integration
vwl repometer --explain
```

Display denominator/methodology and current-vs-historical coverage. Never present one unexplained percentage.

# 7. Regression/impact UX

```text
vwl outcome inspect O-551
vwl impact O-551
vwl revalidate WU-B
```

TUI should show:

```text
Outcome: REGRESSION
Attribution: WU-A HIGH
Current canonical: DEGRADED
Impact:
  WU-B REVALIDATE
  WU-C UNAFFECTED
  WU-D UNKNOWN
```

Historical CONFIRMED state remains visible.

# 8. GitHub Issue projection

A VWL Task may project to a GitHub Issue. Recommended machine-readable block:

```yaml
vwl:
  task: T-381
  spec: S-44
  dependencies:
    requires: [T-377]
  role: developer
```

VWL may project labels:

```text
vwl:ready
vwl:blocked
vwl:claimed
vwl:verifying
vwl:confirmed
vwl:rejected
vwl:revalidate
vwl:degraded
```

Manual label edits do not change VWL state. `vwl sync` restores projection.

# 9. GitHub verification projection

GitHub checks/comments may summarize VerificationPlan and Proofs, but the authoritative evidence object lives in VWL/Git.

Example check summary:

```text
VWL WU-381-A
Deterministic: 7/7 PASS
Judgment: 2 PASS, 1 PENDING
Decision: PENDING
Candidate: A5
Evidence root: ...
```

# 10. GitHub dependency behavior

Dependent Issues remain projected BLOCKED until upstream required effect is canonical. Implementation completion or PR-open state is insufficient.

Independent Tasks may be worked concurrently from the same compatible canonical base.

# 11. GitHub regression behavior

A later Outcome SHOULD NOT rewrite/close/reopen historical Issues as if the original Decision never existed. Instead create/project:

```text
Outcome reference
Repair Task
Affected revalidation Tasks
```

This preserves history.

# 12. TUI main surfaces

```text
HOME       canonical health, Frontier, active claims
WORK       Tasks/claims/WUs
VERIFY     VerificationPlans/Proofs/Challenges
GRAPH      intent/provenance/evidence/impact
REPOMETER  repository evidence coverage
MARKET     optional downstream REP/WORK view
WALLET     optional economics view
```

MARKET/WALLET are optional modules; core CLI works without economics.

# 13. Node worker UX

```text
vwl worker developer T-381 -- <harness>
vwl worker verifier WU-381-A -- <harness>
```

Visibility profile is enforced by adapter/sandbox where possible and displayed in Proof metadata.

# 14. GitHub as WorkHost

GitHub adapter provides:

```text
repository discovery
Issue projection
comments/notifications
Actions integration
optional Pages observatory
```

Protocol must retain a WorkHost abstraction so GitHub is not a protocol dependency.

# 15. Adversarial review

**Attack:** GitHub says READY while VWL says BLOCKED.  
**Resolution:** VWL wins; sync repairs projection.

**Attack:** one-commit CLI commands cannot represent v2.2 WUs.  
**Resolution:** commands target WorkUnit identity/candidate range; commit remains an inspectable field.

**Attack:** verifier posts a test commit into candidate PR.  
**Resolution:** adapter rejects/labels it as VerifierArtifact; repair must create new WU.

**Attack:** GitHub check status becomes authority.  
**Resolution:** checks are projections referencing evidence roots.

**Attack:** regression rewrites old Issue state and destroys audit trail.  
**Resolution:** append Outcome + Repair/Revalidation Issues.

**Attack:** Repometer becomes vanity score.  
**Resolution:** CLI requires dimension/denominator/explainability.

**Attack:** optional market makes core dependent on wallets/contracts.  
**Resolution:** MARKET/WALLET modules are optional downstream views.

No remaining internal CLI/protocol contradiction identified. Residual risks: GitHub rate limits, anti-abuse behavior, sandbox enforcement and UX complexity.

# 16. Final invariant

> The CLI/TUI and GitHub are views/controllers over VWL objects. They may create proposed intent/evidence through explicit commands, but they never become the hidden database of truth. Every status important to authority is reconstructable by replay.
