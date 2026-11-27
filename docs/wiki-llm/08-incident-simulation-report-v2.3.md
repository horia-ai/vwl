# VWL v2.3 — One-Year Incident Simulation Report

**Synthetic stress scenario; not a forecast. No monetary/model-cost assumptions.**

```text
WorkUnits                              2487
Services                                  12
Nodes                                     18
Incidents                                 30

Median MTTD                           3.02 min
P90 MTTD                              6.45 min
Median service restoration           12.65 min
Median evidence restoration          18.78 min
P90 evidence restoration             30.11 min

Average impact cone                  9.37 WUs
P90 impact cone                      18 WUs
Descendant revalidations             281
Descendant code repairs              28
Branches passing                     246
Unknown branches                     7

Incidents without proven root blame  10
Validator-miss incidents             3
Skill/config investigation flags     4

Median service availability          99.99437%
Worst service availability           99.98164%
```

## Interpretation

The simulation used parallel impact-frontier validation and repair barriers. Service could be restored before all evidence was revalidated. Ripple membership only made evidence stale; it did not automatically lower REP. In this run all 18 Nodes encountered stale evidence somewhere in an impact cone, while only 10 Nodes received attributable root-negative evidence.

The simulated availability values are consequences of the scenario assumptions and are not VWL performance promises. Real SLO claims require measured production telemetry and explicit denominators.

## Metrics to validate in real pilots

```text
MTTD
containment time
SRT
root repair time
ERT
impact-cone precision/recall
revalidation pass rate
repair-wave depth
Proof reuse ratio
stale-proof false negatives
validator escape rate
false attribution rate
incident recurrence
customer-visible bad minutes
REP delta by attribution/severity
```
