# Virtual Work Ledger (VWL) — End-to-End Dogfooding Runbook & Operations Manual

This runbook provides complete operational guidance for teams, maintainers, and autonomous agent swarms running the Virtual Work Ledger (VWL) protocol in production and dogfooding environments.

---

## Part 1: Automated Agent & Developer Workflows

### 1.1 Repository Bootstrap & Gate Initialization
When initializing a repository or entering an autonomous agent harness (OpenCode, Claude, Gemini, Copilot):

```bash
# 1. Bootstrap stack detection, configure static gates, install Git hook and agent skills
npx @horia-ai/vwl bootstrap --commit

# 2. Verify all non-bypassable pre-commit validator gates on the clean base
npx @horia-ai/vwl verify precommit
```

**Automated Guarantees**:
- Analyzes repository stack (Rust, TypeScript/Node.js, Python, Go) and writes `.vwl/gates.yaml`.
- Checks for missing dependencies (`package.json` without `node_modules`) and auto-runs `npm install`.
- Installs the non-bypassable Git hook at `.git/hooks/pre-commit` (`--no-verify` is strictly rejected).
- Injects specialist agent skills across `.vwl/skills`, `.opencode/skills`, `.claude/skills`, and `.gemini/skills`.

---

### 1.2 Dual-Path Spec Writing & Task-to-WorkUnit Partitioning

VWL enforces two deterministic paths for specification authoring:

#### Path A: Interactive New Feature Design (`vwl spec new`)
Adopt the **Adversarial Spec Skeptic** persona:
- Rejects subjective phrases (*"clean code"*, *"looks good"*, *"fast enough"*, *"well tested"*).
- Rejects conversational drift (*"as discussed"*, *"in our chat"*, *"like we agreed"*).
- Requires deterministic, executable acceptance commands (`cargo test`, `npm test`, etc.).

```bash
vwl spec new \
  --id FEAT-PAYOUT \
  --objective "Implement liquid WORK to USDC payout liquidation" \
  -a "cargo test -p vwl-cli test_cli_wallet_deposit_and_payout" \
  -a "vwl verify precommit"
```

#### Path B: Reproducible GitHub Issues (`vwl spec from-issue`)
Autonomous ingestion with self-review by `TaskSkeptic` (zero human ping-pong):

```bash
vwl spec from-issue 245 --repo horia-ai/vwl
```

- Automatically extracts code blocks with reproduction test commands (`cargo test`, `npm test`, etc.).
- **Markdown Checklist Decomposition**: If the issue body contains two or more checklist items (`- [ ] Item`), VWL automatically partitions the issue into sequential subtasks:
  - `ISSUE-245-1`: First atomic milestone
  - `ISSUE-245-2`: Requires `ISSUE-245-1`
  - `ISSUE-245-3`: Requires `ISSUE-245-2`
  Each subtask enters the Work Frontier in strict sequential DAG order!

#### Manual Task Splitting (`vwl spec split`)
To split an existing large task into sequential WorkUnits:

```bash
vwl spec split FEAT-AUTH --subtasks "Database schema migration, JWT auth middleware, Integration test suite"
```

---

### 1.3 Developer Node Execution (`vwl dev`)
The developer node executes tasks autonomously inside isolated worktrees:

```bash
# Target a specific GitHub issue:
vwl dev --issue 245

# Or target a specific task ID:
vwl dev --task ISSUE-245-1

# Or run in continuous watch mode (polls Work Frontier for ready tasks):
vwl dev --watch
```

**Developer Protocol Invariants**:
1. **Isolated Worktree**: Creates and executes in `.vwl/worktrees/<task-id>`.
2. **Strict TDD & Mutation Rigor**: Failing test first (Red), minimal implementation (Green), clean refactor (Refactor), and Stryker mutation verification (score >= 80%).
3. **Pre-commit Gate Check**: Verifies `.vwl/gates.yaml` before committing.
4. **WorkUnit Candidate Submission**:
   ```bash
   vwl wu submit <task-id> --candidate <head-sha>
   ```

---

### 1.4 Independent Verification Factory & Roll-Forward Auto-Repair (`vwl verifier`)
The verifier runs in strict isolation with **Zero Implementer Chat** (0 conversational tokens):

```bash
# Continuously verify unverified WorkUnit candidates in the ledger:
vwl verifier --watch

# Or verify a specific candidate commit:
vwl verifier <candidate-sha>
```

**Verification Outcomes**:
- **Confirmed**: All deterministic validator gates pass.
  - Appends cryptographic `Proof` (Type: Work) and `Decision` (Outcome: Confirmed) to ledger.
  - Releases task claim.
  - Triggers economic settlement (80% developer, 15% verifier pool, 5% protocol treasury).
  - Updates GitHub Pages analytics (`docs/data/ledger_report.json`).
- **Rejected (Roll-Forward Invariant)**:
  - Appends `Decision` (Outcome: Rejected).
  - Automatically files an `IncidentRecord` (`INC-<candidate-sha>`).
  - Automatically generates and admits a roll-forward repair task (`REPAIR-INC-<candidate-sha>`) with self-reviewed `TaskSkeptic` validation directly to the Work Frontier.
  - Developer nodes pick up the repair task immediately, ensuring continuous roll-forward progress without human blockers.

---

## Part 2: Team Multi-Sig Treasury & Manual Operations

### 2.1 Multi-Sig Safe Setup (Gnosis Safe on Base or Arbitrum)
To secure the protocol fee treasury and project escrow funds against single points of failure, teams must establish a decentralized Multi-Sig Safe.

#### Manual Safe Deployment Steps:
1. Navigate to [app.safe.global](https://app.safe.global/).
2. Connect owner wallets (e.g., MetaMask, Rabby, or Hardware Wallets like Ledger/Trezor).
3. Select target network (Recommended: **Base** or **Arbitrum One** for low transaction fees and high USDC liquidity).
4. Configure Multi-Sig Signers and Threshold:
   - **Recommended Configuration**: 2-of-3 or 3-of-5 threshold.
   - Example 2-of-3 Signers:
     - Signer 1 (Lead Maintainer / Engineering Lead)
     - Signer 2 (Security Lead / Infrastructure)
     - Signer 3 (Operational Cold Storage / Hardware Wallet)
5. Review transaction and deploy the Safe contract.
6. Note the deployed Safe address (e.g., `0xSafeContractAddress...`).

---

### 2.2 Protocol Treasury Configuration (`.vwl/protocol.yaml`)
Update `.vwl/protocol.yaml` with the deployed Safe address and token contracts:

```yaml
version: "1.0"
network: "base"
treasury:
  wallet_address: "0xSafeContractAddress..."       # Team Multi-Sig Safe
  usdc_token_contract: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"  # USDC on Base
  work_token_contract: "0xWorkTokenContractAddress..."
  protocol_fee_bps: 500                            # 5.00% protocol maintenance fee
  min_payout_threshold_usdc: 50.00
```

Alternatively, configure via environment variables:
```bash
export VWL_TREASURY_ADDRESS="0xSafeContractAddress..."
export VWL_USDC_CONTRACT="0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
```

---

### 2.3 Project Escrow Funding
Project maintainers fund their repositories so worker nodes receive bounties upon verification.

#### Option A: Automated CLI Deposit
```bash
# Fund project with $1,000 USD (mints 1,000,000 WORK escrow credits)
vwl wallet deposit --project horia-ai/vwl --amount-usd 1000
```

#### Option B: Manual Multi-Sig Safe Escrow Deposit
1. Log in to [app.safe.global](https://app.safe.global/) with signer wallet.
2. Transfer USDC from Safe treasury to the `ProjectEscrow` smart contract.
3. Call `depositFiatUSD(bytes32 projectId, uint256 amountUSD)` on the escrow contract.
4. Collect required threshold signatures (e.g., 2 of 3) and execute on-chain.

---

### 2.4 Node Payout Liquidation (WORK -> USDC)
When a developer or verifier node wishes to disburse liquid WORK into USDC:

```bash
# Check wallet balance and vesting schedule:
vwl wallet

# Disburse all available liquid balance:
vwl wallet payout

# Or disburse a specific amount:
vwl wallet payout --amount-usd 150
```

#### Manual Team Payout Approval (Batch Multi-Sig Settlement)
For non-custodial manual disbursement:
1. Export unliquidated payouts from the ledger:
   ```bash
   vwl report --json > /tmp/payouts.json
   ```
2. In the Safe Web UI, open the **Transaction Builder**.
3. Batch transfer USDC to recipient node wallet addresses based on `/tmp/payouts.json`.
4. Submit transaction for multi-sig review.
5. Signers verify cryptographic proof hashes from GitHub Pages (`docs/data/ledger_report.json`).
6. Sign and execute batch transaction.

---

## Part 3: Real-Time Tokenomics & Telemetry Monitoring

1. **Local CLI Inspection**:
   - `vwl wallet`: Inspect liquid WORK, vesting WORK, and active project escrows.
   - `vwl rep`: Inspect reputation score, acceptance rate, and tier status.
   - `vwl market list`: Discover high-paying projects and active funded bounties.

2. **Decentralized Web Dashboard**:
   - GitHub Pages publishes real-time analytics at `https://<owner>.github.io/<repo>/`.
   - Responsive mobile view provides instant visibility into:
     - Total WorkUnits verified and accepted.
     - Contributor effectiveness and token distribution.
     - Active project escrows and team treasury accrual.
