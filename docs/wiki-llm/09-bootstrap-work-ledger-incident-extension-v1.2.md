# VWL Bootstrap Work Ledger v1.2 — v2.3 Incident Extension

**Base:** Bootstrap Work Ledger v1.1 remains valid. Add the following after v2.2 Outcome/Impact work and before REP/economics gates.

## G0.8 — Incident Ripple & Evidence Restoration

### WU-G0.8-001 Incident state machine
Implement CLEAN/SUSPECT/REVALIDATE_REQUIRED/REPAIR_REQUIRED/WAITING_ON_REPAIR/INVALIDATED/REVALIDATED.

### WU-G0.8-002 Proof premise/staleness model
Proofs declare relevant premises; changed premise derives STALE.

### WU-G0.8-003 Impact frontier
Compute immediate affected frontier rather than full descendant rebuild.

### WU-G0.8-004 Repair barrier
If W2 requires code repair, descendants behind W2 become WAITING_ON_REPAIR until Repair-W2 is canonical.

### WU-G0.8-005 Parallel wave revalidation
Validate independent members of the current impact frontier concurrently.

### WU-G0.8-006 Branch stop
PASS/UNAFFECTED branches stop traversal unless deeper independent stale evidence exists.

### WU-G0.8-007 Runtime containment
Represent rollback/isolate/roll-forward as operational events without rewriting canonical Git/VWL history.

### WU-G0.8-008 SRT versus ERT
Measure service restoration independently from complete evidence restoration.

### WU-G0.8-009 Validator obligation analysis
Determine whether an escaped defect was inside a validator gate's declared detectable obligation.

### WU-G0.8-010 Node/Skill attribution separation
Prove ripple staleness does not create negative REP evidence; Skill usage does not equal causation.

### WU-G0.8-011 SLA metric contract
Explicit service/window/denominator/good-bad definition.

### WU-G0.8-012 Replay incident waves
Fresh replay reconstructs containment, repair barriers, stale Proofs, frontier waves, SRT/ERT and final health.

## Hard experiments

1. 50-descendant regression; only minimum affected frontier runs.
2. W2 repair barrier prevents premature W4/W5 authoritative validation.
3. runtime rollback restores service while ledger remains append-only.
4. high-REP Node ripple only → no negative REP evidence.
5. direct high-confidence regression → negative Node evidence.
6. validator outside obligation → no miss.
7. validator inside obligation/detectable defect → miss evidence.
8. Proof premise unchanged → reused; changed → stale.
9. incident reaches healthy canonical state without destructive rebase.
10. replay reproduces exact incident state.

## Stop conditions

Stop/redesign if:
- ripple routinely degenerates into full-tree validation;
- repair barriers deadlock descendants;
- SRT cannot be separated from ERT;
- rollback requires rewriting canonical history;
- attribution causes descendant REP cascades;
- Proof reuse accepts changed premises;
- replay cannot reconstruct incident waves.
