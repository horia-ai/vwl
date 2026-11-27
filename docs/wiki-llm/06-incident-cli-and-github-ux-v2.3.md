# VWL v2.3 — Protocol CLI / GitHub Incident & Release UX

**Dependency:** VWL Protocol v2.3

## Incident commands

```text
vwl incident show <outcome>
vwl incident contain <outcome> --strategy rollback|roll-forward|isolate
vwl impact <outcome>
vwl impact frontier <outcome>
vwl proof stale --because <repair>
vwl revalidate frontier <outcome>
vwl repair start <work-unit>
vwl incident metrics <outcome>
```

## TUI incident view

```text
INCIDENT O-551
Severity: HIGH
Service: payments
Root attribution: W1 HIGH

SERVICE
SRT: 12m
SLO: restored

EVIDENCE
ERT: running
Impact: 17 WUs
REVALIDATED  9
REPAIR       1
WAITING      5
UNKNOWN      2

BARRIER
W2 requires repair
W4/W5/W7 waiting behind W2
```

## GitHub projection

Create/reference:
- Outcome Issue;
- root Repair Task;
- immediate Revalidation Tasks;
- barrier/waiting relationships;
- later descendant tasks only when their frontier opens.

Do not spam GitHub by materializing every possible descendant before it becomes actionable.

Labels are projections:
```text
vwl:incident
vwl:suspect
vwl:revalidate
vwl:repair-required
vwl:waiting-on-repair
vwl:revalidated
```

Changing labels never changes authority.

## Release/deployment UX

```text
vwl release status
vwl deploy verify
vwl deploy canary
vwl deploy shadow
vwl deploy rollback
vwl deploy roll-forward
```

CLI must distinguish:
```text
runtime deployment pointer
canonical engineering state
```

A rollback may restore runtime service while canonical history remains append-only.

## REP/validator/Skill incident UX

```text
vwl incident attribution O-551
vwl node evidence N1 --incident O-551
vwl validator evidence V1 --incident O-551
vwl skill evidence <commitment> --incident O-551
```

UI must never display ripple membership as blame.

Show:
```text
STALE EVIDENCE
NEGATIVE ATTRIBUTION
VALIDATOR MISS
SKILL CONTRIBUTION
```
as distinct concepts.

## SLA metrics

```text
vwl sla service <service> --window 30d
vwl incident metrics O-551
```

Show MTTD, containment, SRT, ERT, impact size, repair waves, bad minutes and explicit SLO denominator.

## Skeptic checks

- GitHub Issues cannot become incident authority.
- Rollback cannot rewrite ledger history.
- Waiting descendants cannot be manually relabeled into authority.
- High REP cannot hide incident evidence.
- Ripple membership cannot be rendered as negative REP.
- Validator/Skill blame requires explicit attributable evidence.
- GitHub issue volume is bounded by actionable frontier, avoiding incident spam.

No further CLI contradiction identified.
