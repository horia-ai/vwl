import os
import json
import time
import urllib.request
import urllib.error

TOKEN = os.environ.get("GITHUB_TOKEN") or os.environ.get("ATRIFAN_GITHUB_PAT_TOKEN")
REPO = "horia-ai/vwl"
API_BASE = f"https://api.github.com/repos/{REPO}"

HEADERS = {
    "Authorization": f"token {TOKEN}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "VWL-Bootstrap"
}

def request(method, endpoint, data=None):
    url = f"{API_BASE}/{endpoint}" if not endpoint.startswith("http") else endpoint
    body = json.dumps(data).encode("utf-8") if data else None
    req = urllib.request.Request(url, data=body, headers=HEADERS, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode("utf-8")
        print(f"HTTPError on {method} {url}: {e.code} - {err_msg}")
        raise

labels = [
    {"name": "gate:0", "color": "0E8A16", "description": "Gate 0 - Minimum Falsifiable VWL"},
    {"name": "gate:1", "color": "1D76DB", "description": "Gate 1 - Node and Project Identity"},
    {"name": "gate:2", "color": "5319E7", "description": "Gate 2 - GitHub Discovery & Market"},
    {"name": "gate:3", "color": "0052CC", "description": "Gate 3 - CLI and TUI"},
    {"name": "gate:4", "color": "FBCA04", "description": "Gate 4 - Reputation and Evidence"},
    {"name": "gate:5", "color": "F9D0C4", "description": "Gate 5 - Local Economic State Machine"},
    {"name": "gate:6", "color": "D93F0B", "description": "Gate 6 - Economic Simulator"},
    {"name": "gate:7", "color": "B60205", "description": "Gate 7 - Testnet Contracts"},
    {"name": "gate:8", "color": "7057FF", "description": "Gate 8 - Wallet and Chain TUI"},
    {"name": "gate:9", "color": "006B75", "description": "Gate 9 - Fiat Onramp & Stripe"},
    {"name": "gate:10", "color": "C2E0C6", "description": "Gate 10 - Productive Transfer & DEX"},
    {"name": "type:work-unit", "color": "BFD4F2", "description": "Bounded Work Unit"},
    {"name": "type:gate-decision", "color": "E99695", "description": "Gate Exit Decision"},
    {"name": "type:infrastructure", "color": "C5DEF5", "description": "CI/CD & Repo Tooling"},
]

print("Ensuring labels exist...")
existing_labels = {l["name"]: l for l in request("GET", "labels?per_page=100")}
for l in labels:
    if l["name"] not in existing_labels:
        request("POST", "labels", l)
        print(f"Created label {l['name']}")

milestones_data = [
    {"title": "Gate 0 — Minimum Falsifiable VWL", "description": "Prove core hypotheses H-CORE, H-INDEPENDENCE, H-COMPOSITION, H-PORTABILITY locally using small Git-native Work Units, replay, and independent verifiers. No network/blockchain/tokens."},
    {"title": "Gate 1 — Node and Project Identity", "description": "Decentralized node signing keys, node credentials, signed work units/proofs, and project membership without central identity service."},
    {"title": "Gate 2 — GitHub-Only Discovery and Market Objects", "description": "GitHub adapter projection, project discovery convention, signed Role and Offer DTOs, and local market scanner."},
    {"title": "Gate 3 — CLI/TUI Shell & Workflows", "description": "Terminal user interface for Home, Market, Projects, Node, and Inbox; candidate inspection and owner selection."},
    {"title": "Gate 4 — REP (Reputation & Evidence Scoring)", "description": "Raw evidence feature extraction, domain REP scoring v0, baseline predictive comparison, and portable archived evidence bundles."},
    {"title": "Gate 5 — Local Economic State Machine", "description": "Simulated accounts, WORK state conservation, milestone budgets, atomic reservation, vesting schedules, and economic replay."},
    {"title": "Gate 6 — Economic Simulator", "description": "Agent-based simulation of labor markets, quality differentiation, Sybil/collusion resistance, and annual circulation dynamics."},
    {"title": "Gate 7 — Testnet Contracts", "description": "Solidity implementation of NodeRegistry, ProjectRegistry, Engagement reservation, VestingSettlement, CheckpointRegistry with differential testing against Rust model."},
    {"title": "Gate 8 — Wallet/Chain TUI", "description": "TUI project/node wallet views, direct contract interaction, funded acceptance UX."},
    {"title": "Gate 9 — Optional Fiat Onramp & Stripe Integration", "description": "Funding provider abstraction, Stripe/fiat onramp POC, and WORK purchase transactions."},
    {"title": "Gate 10 — Productive Transfer & Public Liquidity DEX", "description": "Node/Project WORK transfers, order-book and AMM DEX comparative simulations, and public market go/no-go review."}
]

print("Ensuring milestones exist...")
existing_ms = {m["title"]: m["number"] for m in request("GET", "milestones?state=all")}
milestone_map = {}
for m in milestones_data:
    if m["title"] not in existing_ms:
        created = request("POST", "milestones", m)
        milestone_map[m["title"]] = created["number"]
        print(f"Created milestone: {m['title']} (#{created['number']})")
    else:
        milestone_map[m["title"]] = existing_ms[m["title"]]

# Define Work Unit issues
issues = [
    # Gate 0
    {
        "title": "WU-G0-001 — Rust workspace and CLI shell",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
VWL can remain a small local executable with no service/database dependency.

### Objective
Create minimal Rust workspace and `vwl` CLI.

### Acceptance Criteria
- [ ] `cargo test` passes cleanly
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` zero warnings
- [ ] `vwl --help` displays commands
- [ ] Strictly zero network service, zero database dependencies

### Verification
Run `cargo test && cargo clippy` from clean checkout. Verify dependency tree contains no web servers, databases, or cloud SDKs.
"""
    },
    {
        "title": "WU-G0-002 — Repository identity and canonical base",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
VWL can deterministically identify repository, HEAD, tree and canonical base.

### Objective
Implement Git repository inspection.

### Acceptance Criteria
- [ ] Repo root detected deterministically
- [ ] HEAD commit SHA deterministic
- [ ] Tree SHA deterministic
- [ ] Canonical base stored and read from local metadata
- [ ] Non-Git path cleanly rejected with appropriate error
- [ ] Command `vwl status` prints repository + HEAD + canonical base
"""
    },
    {
        "title": "WU-G0-003 — Immutable Task DTO",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
Bounded intent can be serialized/digested deterministically.

### Objective
Implement Task DTO (`id`, `objective`, `acceptance`, `base`).

### Acceptance Criteria
- [ ] Round-trip YAML serialization is stable
- [ ] Digest calculation (SHA-256) is deterministic
- [ ] Field-order normalization prevents digest mismatch
- [ ] Invalid or missing ID rejected
- [ ] Invalid base commit rejected
- [ ] Command `vwl task add task.yaml` validates and ingests task
"""
    },
    {
        "title": "WU-G0-004 — WorkUnit DTO",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
Candidate work can be represented by Task + Git commit + canonical base.

### Acceptance Criteria
Reject unless:
- [ ] Task exists
- [ ] Commit exists in git object graph
- [ ] Base exists
- [ ] Commit descends directly from expected base commit
"""
    },
    {
        "title": "WU-G0-005 — Proof DTO",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
Verifier evidence can be represented independently of implementer state.

### DTO
`subject` (<commit>), `type` (work|impact|structural), `verdict` (pass|fail|inconclusive), `evidence`.

### Acceptance Criteria
- [ ] Deterministic serialization and content digest
- [ ] Invalid verdict string rejected
- [ ] Subject commit must exist
"""
    },
    {
        "title": "WU-G0-006 — Deterministic Decision",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
AI does not need authority over final state transition; deterministic logic evaluates proofs against policy.

### Acceptance Criteria
- [ ] All required PASS proofs → `CONFIRMED`
- [ ] Any required FAIL proof → `REJECTED`
- [ ] Inconclusive or missing proof → `PENDING`
- [ ] Exhaustive truth-table property tests
- [ ] Strictly zero LLM / model calls inside Decision evaluation
"""
    },
    {
        "title": "WU-G0-007 — Append-only local ledger",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Hypothesis
Gate-0 state can be derived from durable objects rather than runtime memory.

### Objective
Persist Task, WorkUnit, Proof, Decision and CanonicalState records append-only in `.vwl/ledger/`.

### Acceptance Criteria
- [ ] Duplicate identical object ingestion is idempotent
- [ ] Same ID with conflicting content is strictly rejected
- [ ] Corrupt object detected and flagged
- [ ] Process restart loses zero authoritative state
"""
    },
    {
        "title": "WU-G0-008 — Replay",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Critical Hypothesis
A fresh process can reconstruct authoritative state from Git + ledger.

### Command
`vwl replay`

### Acceptance Criteria
- [ ] Delete caches/runtime state and reconstruct Tasks, WorkUnits, Proofs, Decisions, Canonical state
- [ ] Repeated replay produces byte/logically identical derived state
- [ ] HARD STOP if any replay divergence occurs
"""
    },
    {
        "title": "WU-G0-009 — Task worktree assignment",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
`vwl work <task_id>` creates an isolated Git worktree based on the task base commit.

### Acceptance Criteria
- [ ] Parallel worktrees do not mutate canonical branch or each other
- [ ] Correct base checkout and environment isolation
"""
    },
    {
        "title": "WU-G0-010 — Developer worker wrapper (OpenCode harness)",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Command `vwl worker developer <task_id> -- opencode` sets up bounded context and executes developer agent.

### Acceptance Criteria
- [ ] Fresh worker receives only Task, acceptance, base commit, worktree path (no prior conversational history)
- [ ] Captures resulting Git commit SHA cleanly
"""
    },
    {
        "title": "WU-G0-011 — Independent verifier wrapper",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Command `vwl worker verifier <commit> -- <harness>`.

### Acceptance Criteria
- [ ] Verifier receives only Task, acceptance, base commit, candidate commit/diff
- [ ] Zero implementer conversation delivered
- [ ] Outputs structured Proof / Challenge
"""
    },
    {
        "title": "WU-G0-012 — Real parallel Work Unit experiment",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Execute two independent tasks A and B branched concurrently from base commit C0.

### Acceptance Criteria
- [ ] Both workers run concurrently in separate worktrees
- [ ] Verifiers independently verify both
- [ ] Metrics logged: implementation time, tokens, verification time
"""
    },
    {
        "title": "WU-G0-013 — Cutover and composition",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Compose confirmed compatible effects (C0 + ΔA + ΔB → C1).

### Acceptance Criteria
- [ ] Composition manifest recorded
- [ ] Both intended effects present in canonical branch
- [ ] Project test suite passes on C1
- [ ] `vwl replay` reproduces C1 exactly
"""
    },
    {
        "title": "WU-G0-014 — Seeded bad Work Unit experiment",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Seed a plausible defective candidate where self-checks pass but independent verifier discovers bug.

### Acceptance Criteria
- [ ] Developer self-check: PASS
- [ ] Verifier check: FAIL
- [ ] Decision: REJECTED
- [ ] Canonical branch remains unchanged
"""
    },
    {
        "title": "WU-G0-015 — Repair from retained Challenge",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Fresh developer repairs rejected candidate given only the reproducer Challenge.

### Acceptance Criteria
- [ ] Clean repair produced without human or implementer conversation
- [ ] Independent verification PASS → confirmed cutover
"""
    },
    {
        "title": "WU-G0-016 — Semantic composition conflict",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Test two independently valid changes that collide semantically.

### Acceptance Criteria
- [ ] System detects COMPOSITION_CONFLICT
- [ ] Canonical branch remains unchanged
- [ ] HARD STOP if incompatible effects silently canonicalize
"""
    },
    {
        "title": "WU-G0-017 — Spec drift revalidation",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Handle case where spec evolves while candidate is in flight.

### Acceptance Criteria
- [ ] Spec change triggers automatic REVALIDATE status
- [ ] Stale proofs cannot confirm candidate against drifted intent
"""
    },
    {
        "title": "WU-G0-018 — Human takeover and provenance",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Agent starts task; human finishes/commits.

### Acceptance Criteria
- [ ] Actor provenance recorded (agent vs human)
- [ ] Verification and decision rules apply identically
"""
    },
    {
        "title": "WU-G0-019 — Fresh-machine reconstruction",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:work-unit"],
        "body": """### Objective
Clone repository to a pristine environment with zero cache or daemon.

### Acceptance Criteria
- [ ] `vwl replay` reconstructs all historical tasks, passed/failed commits, challenges, repairs, and exact canonical state
"""
    },
    {
        "title": "Gate 0 Exit Decision & Formal Audit",
        "milestone": "Gate 0 — Minimum Falsifiable VWL",
        "labels": ["gate:0", "type:gate-decision"],
        "body": """### Acceptance Criteria for Gate 0 Exit
- [ ] Replay divergence: 0
- [ ] Canonical reproducibility: 100%
- [ ] Seeded bad WU caught by verifier
- [ ] Semantic conflict blocked
- [ ] Fresh machine reconstruction verified
"""
    },
    # Gate 1
    {
        "title": "WU-G1-001 — Operational signing keys (Ed25519)",
        "milestone": "Gate 1 — Node and Project Identity",
        "labels": ["gate:1", "type:work-unit"],
        "body": "Implement local Ed25519 signing key generation, import, and rotation. Sign/verify arbitrary object digests without blockchain."
    },
    {
        "title": "WU-G1-002 — Node credential DTO",
        "milestone": "Gate 1 — Node and Project Identity",
        "labels": ["gate:1", "type:work-unit"],
        "body": "Persistent local node credential containing Node ID, controller ref, public keys, creation epoch, and evidence roots."
    },
    {
        "title": "WU-G1-003 — Signed WorkUnits and Proofs",
        "milestone": "Gate 1 — Node and Project Identity",
        "labels": ["gate:1", "type:work-unit"],
        "body": "Bind Node IDs and signatures to WorkUnit and Proof DTOs. Replay verifies signatures; tampered objects invalidate derived state."
    },
    {
        "title": "WU-G1-004 — Project credential DTO",
        "milestone": "Gate 1 — Node and Project Identity",
        "labels": ["gate:1", "type:work-unit"],
        "body": "Bind Project ID, repository identity, controller public key, and canonical ledger reference."
    },
    {
        "title": "WU-G1-005 — Project membership authorization",
        "milestone": "Gate 1 — Node and Project Identity",
        "labels": ["gate:1", "type:work-unit"],
        "body": "Signed project membership objects authorizing node roles/capabilities. Reject unauthorized node actions."
    },
    # Gate 2
    {
        "title": "WU-G2-001 — GitHub adapter read-only",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "Inspect repo metadata, topics, issues, labels, and comments via GitHub API without writes."
    },
    {
        "title": "WU-G2-002 — Project discovery convention (.vwl/project.yaml)",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "Recognize `topic:vwl-project` and `.vwl/project.yaml`. Local scanner discovers and verifies project identity."
    },
    {
        "title": "WU-G2-003 — Role DTO & schema",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "Signed Role specification: Project ID, optional milestone, role type, requirements, offered WORK amount."
    },
    {
        "title": "WU-G2-004 — Role GitHub projection",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "`vwl role publish` maps signed Role object to GitHub Issue with `vwl-role` labels. Sync detects drift."
    },
    {
        "title": "WU-G2-005 — Offer DTO & schema",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "Signed Node Offer / Counter: Role ID, Node ID, WORK ask, expiry timestamp, and unique nonce."
    },
    {
        "title": "WU-G2-006 — Offer GitHub projection",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "Project offers appear as issue comments. Owner reconstructs candidates from signed comment projections."
    },
    {
        "title": "WU-G2-007 — Local market scanner (vwl market)",
        "milestone": "Gate 2 — GitHub-Only Discovery and Market Objects",
        "labels": ["gate:2", "type:work-unit"],
        "body": "`vwl market` scans GitHub topics and builds local market view with non-authoritative caching."
    },
    # Gate 3
    {
        "title": "WU-G3-001 — TUI shell (Ratatui)",
        "milestone": "Gate 3 — CLI/TUI Shell & Workflows",
        "labels": ["gate:3", "type:work-unit"],
        "body": "Terminal UI with 5 main views: Home, Market, Projects, Node, and Inbox."
    },
    {
        "title": "WU-G3-002 — Market interaction in TUI",
        "milestone": "Gate 3 — CLI/TUI Shell & Workflows",
        "labels": ["gate:3", "type:work-unit"],
        "body": "Inspect roles, apply, submit counter-offer, inspect candidates directly from TUI."
    },
    {
        "title": "WU-G3-003 — Project-owner flow in TUI",
        "milestone": "Gate 3 — CLI/TUI Shell & Workflows",
        "labels": ["gate:3", "type:work-unit"],
        "body": "Owner view: create project, publish role, review candidates, select candidate into proposed Engagement."
    },
    # Gate 4
    {
        "title": "WU-G4-001 — Evidence feature extraction",
        "milestone": "Gate 4 — REP (Reputation & Evidence Scoring)",
        "labels": ["gate:4", "type:work-unit"],
        "body": "Extract raw features from VWL history: confirmed WUs, rejected WUs, first-pass success, repairs, unique catches, difficulty."
    },
    {
        "title": "WU-G4-002 — REP algorithm v0",
        "milestone": "Gate 4 — REP (Reputation & Evidence Scoring)",
        "labels": ["gate:4", "type:work-unit"],
        "body": "Deterministic domain score with mature and provisional components, evidence root, and confidence interval."
    },
    {
        "title": "WU-G4-003 — REP baseline comparison benchmark",
        "milestone": "Gate 4 — REP (Reputation & Evidence Scoring)",
        "labels": ["gate:4", "type:work-unit"],
        "body": "Benchmark REP predictive accuracy against raw WU count and commit counts. Stop if REP adds no predictive lift."
    },
    {
        "title": "WU-G4-004 — Archived & private evidence bundles (vwl archive)",
        "milestone": "Gate 4 — REP (Reputation & Evidence Scoring)",
        "labels": ["gate:4", "type:work-unit"],
        "body": "Content-addressed evidence bundles for private or deleted repos (LIVE, ARCHIVED, ATTESTED, UNVERIFIABLE)."
    },
    # Gate 5
    {
        "title": "WU-G5-001 — Simulated accounts & WORK denomination",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Local double-entry accounting model (1 simulated stablecoin = 1,000 WORK)."
    },
    {
        "title": "WU-G5-002 — WORK state conservation invariants",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Conservation property tests across LIQUID, PROJECT_LOCKED, RESERVED, EARNED, VESTING states."
    },
    {
        "title": "WU-G5-005 — Dual-signed Engagement DTO",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Dual-signed agreement binding Project, Milestone, Role, Node, scope digest, WORK cap, and vesting policy."
    },
    {
        "title": "WU-G5-006 — Atomic reservation & team derivation",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Atomic budget reservation on engagement activation; derive active project team from engagements."
    },
    {
        "title": "WU-G5-008 — Earn transition & vesting schedule",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Move RESERVED to EARNED/VESTING via idempotent VWL settlement attestations."
    },
    {
        "title": "WU-G5-013 — Economic replay verification",
        "milestone": "Gate 5 — Local Economic State Machine",
        "labels": ["gate:5", "type:work-unit"],
        "body": "Wipe state and replay all signed economic objects; assert byte-exact match of balances and vesting."
    },
    # Gate 6
    {
        "title": "WU-G6-001 — Agent-based simulator skeleton",
        "milestone": "Gate 6 — Economic Simulator",
        "labels": ["gate:6", "type:work-unit"],
        "body": "Discrete-event simulation of projects, roles, nodes, offers, work units, and vesting."
    },
    {
        "title": "WU-G6-004 — Sybil & collusion attack simulation",
        "milestone": "Gate 6 — Economic Simulator",
        "labels": ["gate:6", "type:work-unit"],
        "body": "Stress test against developer/verifier collusion, fake projects, and Sybil farming."
    },
    # Gate 7
    {
        "title": "WU-G7-001 — Contract invariant specification",
        "milestone": "Gate 7 — Testnet Contracts",
        "labels": ["gate:7", "type:work-unit"],
        "body": "Formal testable invariants: allocated <= balance, reserved <= available, idempotent nonce, GitHub has 0 financial authority."
    },
    {
        "title": "WU-G7-002 — Core Solidity contracts (NodeRegistry, ProjectRegistry, Escrow, Vesting)",
        "milestone": "Gate 7 — Testnet Contracts",
        "labels": ["gate:7", "type:work-unit"],
        "body": "Solidity implementation of registry, reservation escrow, and vesting settlement contracts."
    },
    {
        "title": "WU-G7-007 — Differential model testing (Rust vs Solidity)",
        "milestone": "Gate 7 — Testnet Contracts",
        "labels": ["gate:7", "type:work-unit"],
        "body": "Run identical transaction sequences against Rust reference model and EVM contracts; assert identical end states."
    },
    # Gate 8
    {
        "title": "WU-G8-001 — Wallet & on-chain TUI views",
        "milestone": "Gate 8 — Wallet/Chain TUI",
        "labels": ["gate:8", "type:work-unit"],
        "body": "TUI panels for project balances, node wallet, contract reservations, and transaction execution."
    },
    # Gate 9
    {
        "title": "WU-G9-001 — Funding provider interface & Stripe MCP POC",
        "milestone": "Gate 9 — Optional Fiat Onramp & Stripe",
        "labels": ["gate:9", "type:work-unit"],
        "body": "Abstract funding interface with Stripe MCP server / fiat-to-stablecoin adapter."
    },
    # Gate 10
    {
        "title": "WU-G10-001 — Transfer & public liquidity DEX evaluation",
        "milestone": "Gate 10 — Productive Transfer & DEX",
        "labels": ["gate:10", "type:work-unit"],
        "body": "Internal order-book and AMM comparative simulations; legal & security public market go/no-go gate."
    }
]

print("Fetching existing issues...")
existing_issues = {i["title"]: i["number"] for i in request("GET", "issues?state=all&per_page=100")}

print(f"Creating {len(issues)} issues...")
for issue in issues:
    if issue["title"] in existing_issues:
        print(f"Issue already exists: {issue['title']} (#{existing_issues[issue['title']]})")
        continue
    ms_title = issue.pop("milestone", None)
    payload = dict(issue)
    if ms_title and ms_title in milestone_map:
        payload["milestone"] = milestone_map[ms_title]
    res = request("POST", "issues", payload)
    print(f"Created issue: {res['title']} (#{res['number']})")
    time.sleep(0.5)

print("All milestones and issues successfully created!")
