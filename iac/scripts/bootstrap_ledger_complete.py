#!/usr/bin/env python3
"""
VWL Ledger Bootstrap Script
Reconciles labels, milestones, governance issues (Hypotheses, Kill Criteria, Metrics),
roadmap umbrella issues (G1-G10), and success-critical issues (SUCCESS-001 to SUCCESS-054),
then synchronizes them to GitHub Projects V2 Board #1 with proper status fields.
"""

import os
import sys
import json
import time
import urllib.request
import urllib.error

TOKEN = os.environ.get("GITHUB_TOKEN") or os.environ.get("ATRIFAN_GITHUB_PAT_TOKEN")
if not TOKEN:
    print("Error: GITHUB_TOKEN or ATRIFAN_GITHUB_PAT_TOKEN must be set.")
    sys.exit(1)

REPO = "horia-ai/vwl"
PROJECT_ID = "PVT_kwDOFRLXoM4BtZXB"
STATUS_FIELD_ID = "PVTSSF_lADOFRLXoM4BtZXBzhrjCyQ"

# Option IDs on ProjectV2 Status field
STATUS_OPTIONS = {
    "BACKLOG": "8b557752",
    "READY": "bd25319f",
    "IMPLEMENTING": "2439a5d2",
    "VERIFYING": "d54159be",
    "CONFIRMED": "76d4a156",
    "REJECTED": "e0cf8daa",
    "BLOCKED": "c497e9b6",
    "EXPERIMENT": "9f286160",
    "STOP": "6626dbcd",
}

HEADERS = {
    "Authorization": f"token {TOKEN}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "VWL-Bootstrap",
}

def api_request(url, method="GET", data=None):
    payload = json.dumps(data).encode("utf-8") if data else None
    req = urllib.request.Request(url, data=payload, headers=HEADERS, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            content = resp.read().decode("utf-8")
            return json.loads(content) if content else {}
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode("utf-8")
        # Treat 422 for already existing entity as soft warning
        return {"_error_code": e.code, "_error_msg": err_msg}

def graphql_query(query, variables=None):
    url = "https://api.github.com/graphql"
    data = json.dumps({"query": query, "variables": variables or {}}).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers=HEADERS, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        print(f"GraphQL Error {e.code}: {e.read().decode('utf-8')}")
        raise

# -------------------------------------------------------------
# 1. Labels
# -------------------------------------------------------------
LABELS = [
    # Gates
    {"name": "vwl:gate-0", "color": "0E8A16", "description": "Gate 0 - Falsify VWL Core"},
    {"name": "vwl:gate-1", "color": "1D76DB", "description": "Gate 1 - Node & Project Identity"},
    {"name": "vwl:gate-2", "color": "5319E7", "description": "Gate 2 - GitHub-Native Discovery"},
    {"name": "vwl:gate-3", "color": "0052CC", "description": "Gate 3 - CLI/TUI"},
    {"name": "vwl:gate-4", "color": "FBCA04", "description": "Gate 4 - REP"},
    {"name": "vwl:gate-5", "color": "F9D0C4", "description": "Gate 5 - Local Economics"},
    {"name": "vwl:gate-6", "color": "D93F0B", "description": "Gate 6 - Economic Simulation"},
    {"name": "vwl:gate-7", "color": "B60205", "description": "Gate 7 - Testnet Contracts"},
    {"name": "vwl:gate-8", "color": "7057FF", "description": "Gate 8 - Wallet/Chain UX"},
    {"name": "vwl:gate-9", "color": "006B75", "description": "Gate 9 - Optional Fiat Onramp"},
    {"name": "vwl:gate-10", "color": "C2E0C6", "description": "Gate 10 - Productive Liquidity"},
    # Types
    {"name": "type:implementation", "color": "BFD4F2", "description": "Implementation work unit"},
    {"name": "type:experiment", "color": "D4C5F9", "description": "Experimental work unit"},
    {"name": "type:verification", "color": "0E8A16", "description": "Verification work unit"},
    {"name": "type:measurement", "color": "FEF2C0", "description": "Measurement & telemetry"},
    {"name": "type:security", "color": "B60205", "description": "Security & attack simulation"},
    {"name": "type:research", "color": "5319E7", "description": "Research & hypothesis testing"},
    {"name": "type:product", "color": "1D76DB", "description": "Product hypothesis & UX"},
    {"name": "type:trust", "color": "0E8A16", "description": "Trust & credibility"},
    {"name": "type:adoption", "color": "FBCA04", "description": "Adoption & network growth"},
    {"name": "type:market", "color": "0052CC", "description": "Market dynamics & discovery"},
    {"name": "type:tokenomics", "color": "F9D0C4", "description": "Tokenomics & economic invariants"},
    {"name": "type:ip", "color": "E99695", "description": "Intellectual property & lineage"},
    {"name": "type:legal", "color": "D93F0B", "description": "Legal & regulatory review"},
    {"name": "type:distribution", "color": "C2E0C6", "description": "Distribution & ecosystem channels"},
    {"name": "type:domain", "color": "7057FF", "description": "Domain generalization & packs"},
    # Status
    {"name": "status:blocked", "color": "D93F0B", "description": "Blocked by dependency"},
    {"name": "status:ready", "color": "0E8A16", "description": "Ready for execution"},
    {"name": "status:experiment", "color": "D4C5F9", "description": "Active experiment"},
    {"name": "status:stop-condition", "color": "B60205", "description": "Hard stop condition trigger"},
    # Priority
    {"name": "priority:p0", "color": "B60205", "description": "P0 - Critical path"},
    {"name": "priority:p1", "color": "D93F0B", "description": "P1 - Important"},
    {"name": "priority:p2", "color": "FBCA04", "description": "P2 - Nice to have"},
    # Roles
    {"name": "role:spec", "color": "FEF2C0", "description": "Specification authority"},
    {"name": "role:developer", "color": "1D76DB", "description": "Implementation worker"},
    {"name": "role:verifier", "color": "0E8A16", "description": "Independent verifier"},
    # Areas
    {"name": "area:core", "color": "0052CC", "description": "Core protocol engine"},
    {"name": "area:git", "color": "5319E7", "description": "Git inspection & worktrees"},
    {"name": "area:ledger", "color": "1D76DB", "description": "Authoritative append-only ledger"},
    {"name": "area:identity", "color": "7057FF", "description": "Node & Project cryptographic identity"},
    {"name": "area:github", "color": "24292E", "description": "GitHub projection & discovery"},
    {"name": "area:cli", "color": "0E8A16", "description": "Command-line interface"},
    {"name": "area:tui", "color": "FBCA04", "description": "Terminal user interface"},
    {"name": "area:rep", "color": "F9D0C4", "description": "Reputation scoring algorithm"},
    {"name": "area:economics", "color": "D93F0B", "description": "Local economic state machine"},
    {"name": "area:contracts", "color": "B60205", "description": "Smart contracts & EVM verification"},
    {"name": "area:market", "color": "006B75", "description": "Decentralized labor market discovery"},
]

def reconcile_labels():
    print("Reconciling GitHub Labels...")
    url = f"https://api.github.com/repos/{REPO}/labels?per_page=100"
    existing = {l["name"].lower(): l for l in api_request(url)}

    for lbl in LABELS:
        name_lower = lbl["name"].lower()
        if name_lower in existing:
            # Update if color or description differ
            patch_url = f"https://api.github.com/repos/{REPO}/labels/{lbl['name']}"
            api_request(patch_url, method="PATCH", data=lbl)
        else:
            create_url = f"https://api.github.com/repos/{REPO}/labels"
            api_request(create_url, method="POST", data=lbl)
    print(f"✓ Reconciled {len(LABELS)} labels.")

# -------------------------------------------------------------
# 2. Milestones
# -------------------------------------------------------------
MILESTONES = [
    {"number": 1, "title": "G0 — Falsify VWL Core"},
    {"number": 2, "title": "G1 — Node & Project Identity"},
    {"number": 3, "title": "G2 — GitHub-Native Discovery"},
    {"number": 4, "title": "G3 — CLI/TUI"},
    {"number": 5, "title": "G4 — REP"},
    {"number": 6, "title": "G5 — Local Economics"},
    {"number": 7, "title": "G6 — Economic Simulation"},
    {"number": 8, "title": "G7 — Testnet Contracts"},
    {"number": 9, "title": "G8 — Wallet/Chain UX"},
    {"number": 10, "title": "G9 — Optional Fiat Onramp"},
    {"number": 11, "title": "G10 — Productive Liquidity"},
]

def reconcile_milestones():
    print("Reconciling Milestones...")
    url = f"https://api.github.com/repos/{REPO}/milestones?state=all&per_page=100"
    current = api_request(url)
    m_by_num = {m["number"]: m for m in current}

    for target in MILESTONES:
        num = target["number"]
        title = target["title"]
        if num in m_by_num:
            if m_by_num[num]["title"] != title:
                patch_url = f"https://api.github.com/repos/{REPO}/milestones/{num}"
                api_request(patch_url, method="PATCH", data={"title": title})
                print(f"  Updated milestone #{num} -> '{title}'")
        else:
            post_url = f"https://api.github.com/repos/{REPO}/milestones"
            api_request(post_url, method="POST", data={"title": title})
            print(f"  Created milestone #{num} -> '{title}'")
    print("✓ Reconciled milestones.")

# -------------------------------------------------------------
# 3. Helpers to Create Issues and Add to Project
# -------------------------------------------------------------
def get_all_repo_issues():
    all_issues = []
    page = 1
    while True:
        url = f"https://api.github.com/repos/{REPO}/issues?state=all&per_page=100&page={page}"
        batch = api_request(url)
        if not batch or "_error_code" in batch or not isinstance(batch, list):
            break
        all_issues.extend(batch)
        if len(batch) < 100:
            break
        page += 1
    return all_issues

def get_existing_issue_titles():
    issues = get_all_repo_issues()
    return {i["title"].strip(): i["number"] for i in issues if "pull_request" not in i}

def create_issue(title, body, labels, milestone=None):
    url = f"https://api.github.com/repos/{REPO}/issues"
    payload = {
        "title": title,
        "body": body,
        "labels": labels,
    }
    if milestone:
        payload["milestone"] = milestone
    resp = api_request(url, method="POST", data=payload)
    if "_error_code" in resp:
        print(f"  Error creating issue '{title}': {resp['_error_msg']}")
        return None
    return resp

def pin_issue(issue_node_id):
    mutation = """
    mutation PinIssue($issueId: ID!) {
      pinIssue(input: { issueId: $issueId }) {
        issue {
          id
        }
      }
    }
    """
    try:
        graphql_query(mutation, {"issueId": issue_node_id})
        print(f"  Pinned issue {issue_node_id}")
    except Exception as e:
        print(f"  Could not pin issue (may require manual pin): {e}")

# -------------------------------------------------------------
# 4. Governance & Meta Issues (Hypotheses, Kill Criteria, Metrics)
# -------------------------------------------------------------
def create_governance_issues(existing_titles):
    print("Creating Governance & Meta Issues...")

    # VWL Hypotheses
    hypotheses_title = "VWL Hypotheses"
    if hypotheses_title not in existing_titles:
        body = """# VWL Core & Success Hypotheses

VWL is not assumed to be correct. Every layer exists to test a specific falsifiable hypothesis.

### H1 — Defect Detection
Independent verification catches useful defects that developer self-check misses.

### H2 — Parallel Scaling
Verified parallel work reduces coordination cost compared to serial branching and manual meetings.

### H3 — Replay Determinism
Canonical state is exactly reconstructible from Git + durable append-only ledger without server state.

### H4 — Context Independence
Fresh workers can continue from ledger evidence and retained Challenges without conversational history.

### H5 — REP Predictive Value
REP predicts future relevant Task outcomes better than simple baselines (PR count, star count, raw commits).

### H6 — Project Selection
Projects use REP to make better selections and achieve higher first-pass success.

### H7 — Portable Reputation
Nodes care about accumulating portable, verifiable, tamper-proof REP across repositories.

### H8 — Repeat Project Demand
Projects repeatedly fund VWL work after completing their first Milestone.

### H9 — Repeat Node Supply
Nodes repeatedly accept VWL Engagements within 30/60/90 days of completion.

### H10 — WORK Accounting Necessity
WORK accounting provides useful functionality (budget partitioning, atomic reservation, vesting) beyond direct stablecoin escrow.

### H11 — Productive Public Liquidity
Public WORK liquidity, if ever introduced, improves rather than damages the labor market."""
        iss = create_issue(hypotheses_title, body, ["type:research", "priority:p0", "area:core"])
        if iss:
            print(f"  Created #{iss['number']}: {hypotheses_title}")
            pin_issue(iss["node_id"])
            existing_titles[hypotheses_title] = iss["number"]

    # Kill Criteria
    kill_title = "Kill Criteria"
    if kill_title not in existing_titles:
        body = """# VWL Kill Criteria & Stop Conditions

### STOP OR REDESIGN IF:
- Replay is nondeterministic across independent runs or machines.
- Canonical state cannot be reproduced from Git + append-only ledger.
- Independent verification adds negligible value over developer self-test.
- Verification costs exceed expected avoided failure cost.
- Composition failures routinely escape to canonical state.
- VWL coordination is worse than ordinary Git/PR workflow.
- Fresh workers require original implementer conversation history to continue.
- REP has negligible predictive lift over simple PR counts.
- REP is cheaply gameable via Sybil attacks or circular hiring.
- Capable new Nodes cannot build REP within reasonable effort.
- Projects do not repeat funding after initial trial.
- Nodes do not repeat participation in the protocol.
- WORK adds little beyond standard stablecoin escrow.
- Economic settlement creates excessive human disputes.
- Public liquidity dominates productive labor activity.

---
> **Governing Invariant**: Failure of a higher layer does not automatically invalidate lower layers.
> Example: `WORK fails ≠ VWL fails`."""
        iss = create_issue(kill_title, body, ["type:research", "status:stop-condition", "priority:p0", "area:core"])
        if iss:
            print(f"  Created #{iss['number']}: {kill_title}")
            pin_issue(iss["node_id"])
            existing_titles[kill_title] = iss["number"]

    # Metrics Issue
    metrics_title = "Metrics"
    if metrics_title not in existing_titles:
        body = """# VWL Telemetry & Protocol Metrics

Track from the first Work Unit:

### Implementation
- `time to spec`
- `time to implement`
- `time to verify`
- `time to repair`
- `LOC` & `files touched`
- `model used` & `reasoning tier`
- `tokens` & `cost`

### Developer Node
- `first-pass success`
- `rejection rate`
- `repair rate`
- `time-to-confirm`
- `cost-to-confirm`

### Verifier
- `unique defects found`
- `false Challenges`
- `cost` & `tokens`
- `verification latency`
- `verifier usefulness ROI`

### Tools
- `Skill usage`
- `MCP usage`
- `tool invocation`
- `whether invocation materially affected outcome`

### Protocol
- `replay divergence` (must remain 0)
- `composition conflicts`
- `manual Git intervention`
- `fresh-worker continuation`
- `canonical reproducibility` (must remain 100%)

### Later Market Metrics
- `REP predictive lift`
- `time-to-fill`
- `qualified candidates / Role`
- `accepted wages`
- `repeat Engagement rate`
- `repeat Project funding rate`
- `vesting disputes`
- `settlement latency`"""
        iss = create_issue(metrics_title, body, ["type:measurement", "priority:p0", "area:core"])
        if iss:
            print(f"  Created #{iss['number']}: {metrics_title}")
            existing_titles[metrics_title] = iss["number"]

# -------------------------------------------------------------
# 5. Roadmap Umbrella Issues (G1 to G10)
# -------------------------------------------------------------
ROADMAP_EPICS = [
    {
        "title": "G1 — Node & Project Identity (Roadmap)",
        "milestone": 2,
        "labels": ["vwl:gate-1", "type:research", "status:blocked", "area:identity", "priority:p1"],
        "body": """### Scope
- Operational signing keys (Ed25519)
- Node credential
- Signed Work Units
- Signed Proofs
- Project credential
- Project membership

**Blocked by**: `G0-020 — VWL Core Go / Stop Decision` PASS."""
    },
    {
        "title": "G2 — GitHub-Native Discovery (Roadmap)",
        "milestone": 3,
        "labels": ["vwl:gate-2", "type:research", "status:blocked", "area:github", "priority:p1"],
        "body": """### Scope
- GitHub adapter (read-only)
- `topic:vwl-project` discovery
- `.vwl/project.yaml` convention
- Role objects & Role Issues
- Signed Offers & Offer comments
- Local market scanner (`vwl market`)
- Zero VWL server required

**Blocked by**: `G1 — Node & Project Identity`."""
    },
    {
        "title": "G3 — CLI/TUI (Roadmap)",
        "milestone": 4,
        "labels": ["vwl:gate-3", "type:research", "status:blocked", "area:tui", "priority:p1"],
        "body": """### Scope
Planned Ratatui interactive screens:
- Home console
- Market discovery
- Projects & Work Units
- Node profile & keys
- Inbox & notifications

**Blocked by**: `G2 — GitHub-Native Discovery`."""
    },
    {
        "title": "G4 — REP (Roadmap)",
        "milestone": 5,
        "labels": ["vwl:gate-4", "type:research", "status:blocked", "area:rep", "priority:p1"],
        "body": """### Scope
- Raw evidence feature extraction
- REP algorithm v0
- Mature vs provisional REP
- Baseline comparison benchmark
- Archived & private evidence bundles

**Hard Hypothesis**: REP must predict useful future outcomes better than simpler baselines.
**Blocked by**: `G3 — CLI/TUI`."""
    },
    {
        "title": "G5 — Local Economics (Roadmap)",
        "milestone": 6,
        "labels": ["vwl:gate-5", "type:research", "status:blocked", "area:economics", "priority:p1"],
        "body": """### Scope (NO BLOCKCHAIN YET)
Implement purely local state machine:
- NodeAccount & ProjectAccount
- Simulated WORK denomination (1 stablecoin = 1,000 WORK)
- Milestone budgets
- Offers & dual-signed Engagements
- Atomic budget reservation
- Earning & linear vesting
- Repo disappearance handling
- Economic replay verification

**Blocked by**: `G4 — REP`."""
    },
    {
        "title": "G6 — Economic Simulation (Roadmap)",
        "milestone": 7,
        "labels": ["vwl:gate-6", "type:research", "status:blocked", "area:economics", "priority:p2"],
        "body": """### Scope
Discrete-event agent simulations:
- Labor shortage & oversupply
- Node quality distributions
- Sybil attacks & fake Projects
- Developer/verifier collusion
- One-year circulation stress test
- Repeat hiring & funding dynamics

**Blocked by**: `G5 — Local Economics`."""
    },
    {
        "title": "G7 — Testnet Contracts (Roadmap)",
        "milestone": 8,
        "labels": ["vwl:gate-7", "type:research", "status:blocked", "area:contracts", "priority:p2"],
        "body": """### Scope
Implemented only after G5/G6 semantics survive:
- Solidity contracts matching Rust reference model
- Differential model testing: `Rust economics model == Solidity contracts final state`
- Any divergence is a hard stop.

**Blocked by**: `G6 — Economic Simulation`."""
    },
    {
        "title": "G8 — Wallet / Chain UX (Roadmap)",
        "milestone": 9,
        "labels": ["vwl:gate-8", "type:research", "status:blocked", "area:contracts", "priority:p2"],
        "body": """### Scope
- Project wallet & Node wallet
- Balances & Milestone allocations
- Reserved WORK & vesting views
- Funded acceptance UX

**Blocked by**: `G7 — Testnet Contracts`."""
    },
    {
        "title": "G9 — Optional Fiat Onramp (Roadmap)",
        "milestone": 10,
        "labels": ["vwl:gate-9", "type:research", "status:blocked", "area:economics", "priority:p2"],
        "body": """### Scope
Optional convenience only (Core VWL works with existing stablecoins):
- Fiat onramp -> supported stablecoin -> non-custodial wallet -> signed WORK purchase
- Zero protocol custody requirement.

**Blocked by**: `G8 — Wallet / Chain UX`."""
    },
    {
        "title": "G10 — Productive Liquidity (Roadmap)",
        "milestone": 11,
        "labels": ["vwl:gate-10", "type:research", "status:blocked", "area:market", "priority:p2"],
        "body": """### Scope
Only after real productive labor use exists:
- Node/Project WORK transfers
- Internal market experiments
- DEX simulations (Uniswap v3)
- Public market go/no-go gate
- Public WORK trading is OPTIONAL.

**Blocked by**: `G9 — Optional Fiat Onramp` plus legal/security/economic review."""
    },
]

def create_roadmap_epics(existing_titles):
    print("Creating Roadmap Umbrella Issues...")
    for epic in ROADMAP_EPICS:
        title = epic["title"]
        if title not in existing_titles:
            iss = create_issue(title, epic["body"], epic["labels"], epic["milestone"])
            if iss:
                print(f"  Created #{iss['number']}: {title}")
                existing_titles[title] = iss["number"]
            time.sleep(0.3)

# -------------------------------------------------------------
# 6. Success-Critical Issues (SUCCESS-001 through SUCCESS-054)
# -------------------------------------------------------------
SUCCESS_ISSUES = [
    {
        "title": "SUCCESS-001 — Prove independent verification has positive economic value",
        "labels": ["type:experiment", "type:verification", "priority:p0", "area:core"],
        "body": """### Question
Does independent verification prevent enough real defects to justify its cost?

### Measure
```text
implementation cost
verification cost
unique verifier catches
severity of catches
repair avoided
production failures avoided
tokens & latency
```

### Calculate
$$Verifier\\ ROI = \\frac{\\text{expected avoided failure cost}}{\\text{verification cost}}$$

### Segment by
- Task difficulty
- language
- security sensitivity
- change size
- Node REP
- verifier type

### Success Condition
Verifier discovered information that materially changed the outcome (not merely produced a Proof)."""
    },
    {
        "title": "SUCCESS-002 — Determine optimal verification intensity",
        "labels": ["type:experiment", "type:verification", "priority:p1", "area:core"],
        "body": """### Question
Test whether every Work Unit needs the same verification.

### Compare
- 0 verifiers (developer self-check alone)
- 1 verifier
- 2 verifiers
- specialized verifier
- adversarial verifier

### Measure
Cost, latency, unique defects caught, false Challenges, and downstream failures.

### Goal
Derive risk-adjusted verification policy."""
    },
    {
        "title": "SUCCESS-003 — Prove VWL beats ordinary Git/PR workflow",
        "labels": ["type:experiment", "type:product", "priority:p0", "area:core"],
        "body": """### Experiment
Run comparable real Tasks through:
- **A**: Normal GitHub PR workflow
- **B**: VWL protocol (bounded work unit, independent verification, replay)

### Compare
Delivery time, human coordination time, agent idle time, defects, repair burden, parallelism, reconstruction, cost, and tokens.

### Invariant
VWL must demonstrate a measurable advantage somewhere meaningful; not merely create additional metadata."""
    },
    {
        "title": "SUCCESS-004 — Measure protocol bureaucracy",
        "labels": ["type:measurement", "type:product", "priority:p1", "area:core"],
        "body": """### Track
- Protocol objects / Work Unit
- Human interactions / Work Unit
- Agent interactions / Work Unit
- Ledger bytes & Git operations
- Verification latency & manual interventions

### Goal
Keep protocol complexity strictly machine-facing. Human experience remains: create project, publish work, handle escalations, inspect results."""
    },
    {
        "title": "SUCCESS-005 — Define Node configuration lineage",
        "labels": ["type:trust", "type:identity", "priority:p1", "area:identity"],
        "body": """### Problem
A Node earns REP using one configuration and later changes:
`model, system prompt, skills, MCPs, tools, operator, verification configuration`.

### Design
```text
Node
└── Configuration Epoch
    ├── model family
    ├── skills
    ├── MCP capabilities
    ├── policy
    └── operator/controller
```
REP must not become transferable by silently replacing the system underneath a successful Node."""
    },
    {
        "title": "SUCCESS-006 — REP must beat simple baselines",
        "labels": ["type:experiment", "type:rep", "priority:p0", "area:rep"],
        "body": """### Hypothesis
Domain REP must predict the NEXT relevant Task outcome better than simple baselines:
- GitHub contribution count
- Merged PR count
- Confirmed Work Unit count
- First-pass success alone
- Model identity alone

### Hard Condition
If sophisticated REP provides negligible predictive lift, simplify it."""
    },
    {
        "title": "SUCCESS-007 — REP difficulty normalization",
        "labels": ["type:rep", "type:tokenomics", "priority:p1", "area:rep"],
        "body": """### Goal
Prevent Nodes from maximizing REP by choosing only trivial/easy Tasks.
Design difficulty/risk normalization using: Task complexity, historical failure rate, affected components, verification difficulty, security sensitivity, and repair history."""
    },
    {
        "title": "SUCCESS-008 — REP gaming / adversarial economy",
        "labels": ["type:security", "type:rep", "priority:p0", "area:rep"],
        "body": """### Attack Scenarios
- Fake Projects & Sybil Nodes
- Developer/verifier collusion
- Self-created defects & self-repair loops
- Easy-task farming & circular hiring
- Private-attestation farming

### Goal
Make fake REP more expensive than producing genuine useful work."""
    },
    {
        "title": "SUCCESS-009 — Verifier REP",
        "labels": ["type:rep", "type:verification", "priority:p1", "area:rep"],
        "body": """### Define Signals
Unique valid catches, defect severity, false Challenge rate, reproducer quality, downstream confirmation, cost, and latency.
Avoid rewarding verbosity or raw number of FAIL verdicts."""
    },
    {
        "title": "SUCCESS-010 — Spec Writer REP",
        "labels": ["type:rep", "type:product", "priority:p2", "area:rep"],
        "body": """### Signals
Task ambiguity, rework caused by Spec, Spec drift, human escalations, Task completion rate, and downstream repair.
Test whether good Spec REP predicts lower implementation rework."""
    },
    {
        "title": "SUCCESS-011 — Skill usefulness",
        "labels": ["type:measurement", "type:domain", "priority:p1", "area:core"],
        "body": """### Analysis
Ledger must attribute tool/skill configuration context sufficiently to answer:
How often was Skill X used? Did success probability improve? Did cost/latency change?
Identify: useful Skills, neutral Skills, and harmful Skills."""
    },
    {
        "title": "SUCCESS-012 — MCP usefulness",
        "labels": ["type:measurement", "type:domain", "priority:p1", "area:core"],
        "body": """### Analysis
Measure MCP usage, successful action rate, Task outcome, cost, and latency.
Answer: *Does this MCP actually make this Node better at this class of work?*"""
    },
    {
        "title": "SUCCESS-013 — Coding-agent/model efficiency",
        "labels": ["type:measurement", "type:research", "priority:p1", "area:core"],
        "body": """### Track
First-pass success, confirmed Work Units, repair rate, verification cost generated, tokens, and time.
Derive best agent/model for Task class X at cost/risk Y."""
    },
    {
        "title": "SUCCESS-014 — Dynamic Node routing",
        "labels": ["type:experiment", "type:market", "priority:p2", "area:market"],
        "body": """### Experiment
Use ledger evidence to test routing:
`Task class -> Node/model/configuration` versus random/default assignment.
Measure whether routing improves success, cost, and latency."""
    },
    {
        "title": "SUCCESS-015 — Cold-start Node pathway",
        "labels": ["type:adoption", "type:market", "priority:p0", "area:market"],
        "body": """### Hard Condition
A genuinely capable new Node must not require months of unpaid work merely to enter the market.
Design: qualification Tasks, open-source Tasks, shadow verification, and low-risk paid Tasks."""
    },
    {
        "title": "SUCCESS-016 — Marketplace liquidity: first 100 Nodes / first 20 Projects",
        "labels": ["type:adoption", "type:market", "priority:p0", "area:market"],
        "body": """### Cold-Start Cohort
Target: 100 Nodes, 20 real Projects, 1,000+ verified Work Units.
Optimize for funded Roles, qualified bids, accepted Engagements, completed verified work, and repeat participation."""
    },
    {
        "title": "SUCCESS-017 — Repeat Engagement Rate",
        "labels": ["type:measurement", "type:market", "priority:p0", "area:market"],
        "body": """### Primary Supply-Side Metric
Nodes completing an Engagement that accept another Engagement within 30/60/90 days.
Track by REP, domain, wage, Project quality, and Task type."""
    },
    {
        "title": "SUCCESS-018 — Repeat Project Funding Rate",
        "labels": ["type:measurement", "type:market", "priority:p0", "area:market"],
        "body": """### Primary Demand-Side Metric
Projects completing a funded Milestone that voluntarily fund another Milestone.
This is one of the strongest indicators of real protocol value."""
    },
    {
        "title": "SUCCESS-019 — Time-to-fill and qualified liquidity",
        "labels": ["type:measurement", "type:market", "priority:p1", "area:market"],
        "body": """### Metric
`Role published -> first qualified bid -> accepted bid -> active Engagement`.
Measure qualified bidders per Role across domain, REP, and wage."""
    },
    {
        "title": "SUCCESS-020 — Wage discovery",
        "labels": ["type:research", "type:market", "priority:p1", "area:market"],
        "body": """### Study
Relationship between REP, difficulty, scarcity, availability, domain, and accepted wage.
Determine whether demonstrated capability earns a measurable market premium."""
    },
    {
        "title": "SUCCESS-021 — GitHub discovery scaling",
        "labels": ["type:experiment", "type:github", "priority:p1", "area:github"],
        "body": """### Scale Test
Test indexing 100, 1,000, 10,000, and 100,000 Projects.
Measure GitHub API calls, rate limits, latency, cache size, and synchronization completeness."""
    },
    {
        "title": "SUCCESS-022 — GitHub anti-abuse/spam model",
        "labels": ["type:security", "type:github", "priority:p1", "area:github"],
        "body": """### Goal
Ensure VWL participation looks like legitimate open-source collaboration, not automated issue spam.
Evaluate signed Offer references, compact comments, separate Offer refs, and Discussions."""
    },
    {
        "title": "SUCCESS-023 — WorkHost abstraction",
        "labels": ["type:architecture", "type:git", "priority:p2", "area:git"],
        "body": """### Abstraction
Define minimal interface separating VWL from GitHub (`GitHubWorkHost`, `GitLabWorkHost`, `ForgejoWorkHost`, `LocalGitWorkHost`).
GitHub dependency must not become protocol lock-in."""
    },
    {
        "title": "SUCCESS-024 — Repository deletion survival",
        "labels": ["type:experiment", "type:git", "priority:p1", "area:git"],
        "body": """### Destructive Experiment
Complete Project work, archive evidence, delete origin repository.
A fresh machine must reconstruct Node evidence, recompute REP, and continue vesting."""
    },
    {
        "title": "SUCCESS-025 — Private Project evidence",
        "labels": ["type:product", "type:trust", "priority:p1", "area:ledger"],
        "body": """### Design
Define how private Projects contribute credible REP without exposing proprietary code:
Signed attestations, commit digests, Decision digests, and outcome statistics."""
    },
    {
        "title": "SUCCESS-026 — Project quality / Project reputation",
        "labels": ["type:rep", "type:product", "priority:p1", "area:rep"],
        "body": """### Signals
Payment reliability, scope stability, Spec quality, dispute rate, Node retention, and verification quality.
Avoid a one-sided marketplace where only workers are scored."""
    },
    {
        "title": "SUCCESS-027 — Project owner griefing",
        "labels": ["type:security", "type:trust", "priority:p1", "area:economics"],
        "body": """### Attack Scenarios
Scope changes after hiring, unreasonable Challenges, repo deletion, refusing acknowledgment, and wage withholding.
Ensure settlement policies protect worker Nodes."""
    },
    {
        "title": "SUCCESS-028 — Node abandonment",
        "labels": ["type:product", "type:economics", "priority:p1", "area:economics"],
        "body": """### Policy
Define handling when a Node disappears or misses deadlines:
Unused reservation return, partial earned compensation, REP effect, and replacement."""
    },
    {
        "title": "SUCCESS-029 — Dispute economics",
        "labels": ["type:measurement", "type:economics", "priority:p1", "area:economics"],
        "body": """### Metric
Disputes per Engagement, time to resolution, human involvement, and amount at risk.
If disputes become common, the automated settlement hypothesis fails."""
    },
    {
        "title": "SUCCESS-030 — WORK necessity experiment",
        "labels": ["type:experiment", "type:tokenomics", "priority:p0", "area:economics"],
        "body": """### Hard Condition
Compare:
- **A**: USDC escrow + VWL accounting
- **B**: WORK accounting/token
If WORK adds negligible value beyond stablecoin escrow, keep it internal or remove transferable WORK."""
    },
    {
        "title": "SUCCESS-031 — Productive Lock Ratio",
        "labels": ["type:measurement", "type:tokenomics", "priority:p1", "area:economics"],
        "body": """### Formula
$$PLR = \\frac{\\text{ProjectLocked} + \\text{Reserved} + \\text{Vesting}}{\\text{Total WORK}}$$
Distinguish WORK connected to actual labor from idle/speculative supply."""
    },
    {
        "title": "SUCCESS-032 — Productive Trading Ratio",
        "labels": ["type:measurement", "type:tokenomics", "priority:p1", "area:market"],
        "body": """### Formula
$$PTR = \\frac{\\text{productive WORK acquisition/use}}{\\text{total WORK transaction volume}}$$
Public trading is not successful merely because volume is high."""
    },
    {
        "title": "SUCCESS-033 — Prevent token-first product failure",
        "labels": ["type:tokenomics", "type:product", "priority:p0", "area:economics"],
        "body": """### Launch Guardrail
VWL SHALL NOT initially market token appreciation, fixed scarcity, yield, staking APY, or 'get in early'.
Primary narrative: verified work, portable reputation, labor discovery, funded Engagements."""
    },
    {
        "title": "SUCCESS-034 — Stable WORK versus floating WORK experiment",
        "labels": ["type:experiment", "type:tokenomics", "priority:p1", "area:economics"],
        "body": """### Compare
- **Model A**: Permanent stable labor accounting (1 USD = 1,000 WORK)
- **Model B**: Transferable non-public WORK
- **Model C**: Floating WORK/USDC market"""
    },
    {
        "title": "SUCCESS-035 — Protocol fee willingness",
        "labels": ["type:experiment", "type:economics", "priority:p2", "area:economics"],
        "body": """### Experiment
Test protocol fees of 0.5%, 1%, 2%, and 3%. Measure conversion, off-protocol settlement, and repeat funding."""
    },
    {
        "title": "SUCCESS-036 — Fee bypass analysis",
        "labels": ["type:research", "type:economics", "priority:p1", "area:economics"],
        "body": """### Scenario
Project discovers Node through VWL and settles outside. Determine what settlement must provide to justify staying inside."""
    },
    {
        "title": "SUCCESS-037 — Alternative business models",
        "labels": ["type:research", "type:product", "priority:p2", "area:economics"],
        "body": """### Revenue Independent of Token Appreciation
Protocol settlement fee, enterprise VWL, verification products, certification, and Node fleet tooling."""
    },
    {
        "title": "SUCCESS-038 — Trust UX",
        "labels": ["type:product", "type:trust", "priority:p1", "area:rep"],
        "body": """### Research
What makes a Project owner trust an unknown Node?
Compare: REP number only, REP + confidence, REP + comparable Work Units, REP + repairs, and full evidence drill-down."""
    },
    {
        "title": "SUCCESS-039 — Evidence transparency UX",
        "labels": ["type:product", "type:trust", "priority:p1", "area:rep"],
        "body": """### Drill-Down
From any REP score, user must be able to navigate:
`REP -> evidence category -> Project -> Work Unit -> Proof -> Decision -> outcome`."""
    },
    {
        "title": "SUCCESS-040 — Node identity UX",
        "labels": ["type:product", "type:identity", "priority:p2", "area:identity"],
        "body": """### Mental Model
Users must understand: Node != model, Node != one process, Node != necessarily AI."""
    },
    {
        "title": "SUCCESS-041 — Project identity UX",
        "labels": ["type:product", "type:identity", "priority:p2", "area:core"],
        "body": """### Mental Model
Repository = VWL Project, Project account = treasury, Milestone = budget partition, Role = labor demand, Engagement = contract."""
    },
    {
        "title": "SUCCESS-042 — First-run onboarding",
        "labels": ["type:product", "type:adoption", "priority:p0", "area:cli"],
        "body": """### Target
Node: `install -> Node created -> GitHub connected -> first market view` (< 5 minutes).
Project: `install -> project init -> first Role published` (< 5 minutes)."""
    },
    {
        "title": "SUCCESS-043 — GitHub badge experiment",
        "labels": ["type:adoption", "type:distribution", "priority:p2", "area:github"],
        "body": """### Badges
VWL Verified Project, Verified Work Units, Replay Pass, Active Nodes.
Measure organic discovery generated by badges."""
    },
    {
        "title": "SUCCESS-044 — GitHub Pages Observatory",
        "labels": ["type:product", "type:trust", "priority:p1", "area:github"],
        "body": """### Observatory
Static GitHub Pages generating reproducible protocol docs, network stats, contract addresses, and economic metrics."""
    },
    {
        "title": "SUCCESS-045 — Distribution experiment",
        "labels": ["type:experiment", "type:distribution", "priority:p1", "area:market"],
        "body": """### Channels
Compare GitHub discovery, README badges, Hacker News, Reddit, OpenCode ecosystem, and developer communities."""
    },
    {
        "title": "SUCCESS-046 — Funded open-source launch program",
        "labels": ["type:adoption", "type:economics", "priority:p1", "area:market"],
        "body": """### Program
Allocate launch budget directly to real Projects, real funded work, and real verification rather than ads."""
    },
    {
        "title": "SUCCESS-047 — First 10 reference Projects",
        "labels": ["type:adoption", "type:product", "priority:p0", "area:market"],
        "body": """### Criteria
Good test coverage, clear ownership, real backlog, measurable outcomes, reasonable Task decomposition, and willing maintainers."""
    },
    {
        "title": "SUCCESS-048 — Real-world verifier benchmark",
        "labels": ["type:experiment", "type:verification", "priority:p1", "area:core"],
        "body": """### Benchmark
Build corpus of genuine defects (logic, integration, concurrency, security, performance, spec mismatch, composition) to test verifier Nodes."""
    },
    {
        "title": "SUCCESS-049 — Long-term outcome observation",
        "labels": ["type:measurement", "type:rep", "priority:p1", "area:rep"],
        "body": """### Windows
Track regressions, repairs, reverts, and production failures at 7d, 30d, 90d, and 180d intervals."""
    },
    {
        "title": "SUCCESS-050 — Causal attribution quality",
        "labels": ["type:research", "type:rep", "priority:p1", "area:rep"],
        "body": """### Attribution
Test whether VWL can reliably connect later Issues to specific past Work Units before applying economic penalties."""
    },
    {
        "title": "SUCCESS-051 — Marketing domain feasibility",
        "labels": ["type:research", "type:domain", "priority:p2", "area:core"],
        "body": """### Generalization
Research whether VWL core generalizes from code to marketing (campaign configuration, brand compliance, conversion, CAC, attribution)."""
    },
    {
        "title": "SUCCESS-052 — Domain Pack architecture",
        "labels": ["type:architecture", "type:domain", "priority:p2", "area:core"],
        "body": """### Architecture
Domain Pack: practitioner roles, Skills, MCP capabilities, verifier types, evidence types, outcome metrics, and REP dimensions. Software is Domain Pack #1."""
    },
    {
        "title": "SUCCESS-053 — Practitioner/MCP distinction",
        "labels": ["type:trust", "type:domain", "priority:p1", "area:core"],
        "body": """### Formal Invariant
MCP = access/capability; Skill = procedure; REP = evidence of valuable outcomes. Having an MCP must never itself confer REP."""
    },
    {
        "title": "SUCCESS-054 — Human/AI hybrid Nodes",
        "labels": ["type:experiment", "type:identity", "priority:p1", "area:identity"],
        "body": """### Hybrid Node Testing
Test Nodes operated jointly by human and AI. Verify that human authorship is recorded but does not bypass independent protocol verification."""
    },
]

def create_success_issues(existing_titles):
    print("Creating Success-Critical Issues (SUCCESS-001 to SUCCESS-054)...")
    for s in SUCCESS_ISSUES:
        title = s["title"]
        if title not in existing_titles:
            iss = create_issue(title, s["body"], s["labels"])
            if iss:
                print(f"  Created #{iss['number']}: {title[:40]}...")
                existing_titles[title] = iss["number"]
            time.sleep(0.3)

# -------------------------------------------------------------
# 7. Project Board Synchronization & Status Assignment
# -------------------------------------------------------------
def sync_project_board_and_status():
    print(f"Fetching all issues to sync with Project Board ({PROJECT_ID})...")
    issues = get_all_repo_issues()

    # 1. Fetch all current items on project board with cursor pagination
    query = """
    query GetProjectItems($projectId: ID!, $cursor: String) {
      node(id: $projectId) {
        ... on ProjectV2 {
          items(first: 100, after: $cursor) {
            pageInfo {
              hasNextPage
              endCursor
            }
            nodes {
              id
              content {
                ... on Issue {
                  number
                  title
                }
              }
            }
          }
        }
      }
    }
    """
    existing_items = {}
    cursor = None
    while True:
        res = graphql_query(query, {"projectId": PROJECT_ID, "cursor": cursor})
        items_data = res.get("data", {}).get("node", {}).get("items", {})
        for node in items_data.get("nodes", []):
            content = node.get("content")
            if content and "number" in content:
                existing_items[content["number"]] = node["id"]
        page_info = items_data.get("pageInfo", {})
        if page_info.get("hasNextPage"):
            cursor = page_info.get("endCursor")
        else:
            break

    add_mutation = """
    mutation AddItem($projectId: ID!, $contentId: ID!) {
      addProjectV2ItemById(input: { projectId: $projectId, contentId: $contentId }) {
        item {
          id
        }
      }
    }
    """

    set_status_mutation = """
    mutation SetStatus($projectId: ID!, $itemId: ID!, $fieldId: ID!, $optionId: String!) {
      updateProjectV2ItemFieldValue(input: {
        projectId: $projectId
        itemId: $itemId
        fieldId: $fieldId
        value: { singleSelectOptionId: $optionId }
      }) {
        projectV2Item {
          id
        }
      }
    }
    """

    print(f"Total repository issues: {len(issues)}. Processing board cards and statuses...")
    for iss in issues:
        if "pull_request" in iss:
            continue
        num = iss["number"]
        title = iss["title"]
        node_id = iss["node_id"]

        # Ensure on project board
        item_id = existing_items.get(num)
        if not item_id:
            try:
                res = graphql_query(add_mutation, {"projectId": PROJECT_ID, "contentId": node_id})
                if res.get("data", {}).get("addProjectV2ItemById"):
                    item_id = res["data"]["addProjectV2ItemById"]["item"]["id"]
                    existing_items[num] = item_id
            except Exception as e:
                print(f"  Error adding #{num} to project: {e}")
                continue

        # Determine target status:
        # Issue #1 (G0-001) is READY
        # Experiments -> EXPERIMENT
        # Governance / Hypotheses / Kill -> BACKLOG
        # Success issues -> BACKLOG or EXPERIMENT
        # Blocked issues -> BLOCKED
        target_status = "BLOCKED"
        if num == 1 or "G0-001" in title:
            target_status = "READY"
        elif "SUCCESS" in title:
            target_status = "EXPERIMENT" if "experiment" in title.lower() or "test" in title.lower() else "BACKLOG"
        elif title in ["VWL Hypotheses", "Kill Criteria", "Metrics"]:
            target_status = "BACKLOG"
        elif "experiment" in title.lower():
            target_status = "EXPERIMENT"
        elif "Stop Decision" in title:
            target_status = "STOP"

        option_id = STATUS_OPTIONS.get(target_status)
        if item_id and option_id:
            try:
                graphql_query(set_status_mutation, {
                    "projectId": PROJECT_ID,
                    "itemId": item_id,
                    "fieldId": STATUS_FIELD_ID,
                    "optionId": option_id,
                })
            except Exception as e:
                print(f"  Error setting status for #{num}: {e}")
            time.sleep(0.15)

    print("✓ Project board synchronization and status assignment complete.")

# -------------------------------------------------------------
# Main Execution
# -------------------------------------------------------------
def main():
    print("=== VWL Experimental Ledger Bootstrap ===")
    reconcile_labels()
    reconcile_milestones()
    existing_titles = get_existing_issue_titles()
    create_governance_issues(existing_titles)
    create_roadmap_epics(existing_titles)
    create_success_issues(existing_titles)
    sync_project_board_and_status()
    print("=== All bootstrap steps successfully completed ===")

if __name__ == "__main__":
    main()
