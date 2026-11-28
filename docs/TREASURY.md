# Virtual Work Ledger (VWL) — Team Protocol Fee Treasury & USDC Escrow

## 1. Overview
The Virtual Work Ledger protocol incorporates an automated protocol fee mechanism to ensure long-term self-sustainability, security auditing, and infrastructure maintenance. Whenever a candidate WorkUnit is verified and accepted by the verification engine, economic settlement disburses accrued value according to strict protocol-level allocation invariants.

## 2. Settlement Fee Distribution Invariant
For every settled and confirmed WorkUnit with a allocated bounty (in WORK or USDC):
- **Developer Implementation**: 80% (released immediately or according to vesting curve).
- **Independent Verifier Pool**: 15% (distributed across specialist verifiers providing passing proofs).
- **Team Protocol Treasury**: 5% (transferred to the protocol fee treasury wallet).

```
Total Bounded Reward
       │
       ├── 80% ──► Developer Node Wallet (Vesting / Liquid WORK)
       │
       ├── 15% ──► Verifier Node Pool (Specialist Proof Rewards)
       │
       └──  5% ──► Team Protocol Treasury Wallet (Operations & Security)
```

## 3. Team Protocol Treasury Wallet Configuration
The protocol treasury wallet receives the 5% network maintenance fee. It is configured declaratively in `.vwl/protocol.yaml` or injected via the `VWL_TREASURY_ADDRESS` environment variable.

### Configuration Format (`.vwl/protocol.yaml`)
```yaml
version: "1.0"
network: "testnet"
treasury:
  wallet_address: "0x0000000000000000000000000000000000000001"
  usdc_token_contract: "0x0000000000000000000000000000000000000002"
  work_token_contract: "0x0000000000000000000000000000000000000003"
  protocol_fee_bps: 500  # 5.00%
  min_payout_threshold_usdc: 50.00
```

### Environment Variable Override
```bash
export VWL_TREASURY_ADDRESS="0x0000000000000000000000000000000000000001"
export VWL_USDC_CONTRACT="0x0000000000000000000000000000000000000002"
```

## 4. USDC Escrow & Fiat Onramp Integration
Project maintainers fund their repositories by locking USDC or WORK tokens in a decentralized ProjectEscrow smart contract or local economic state machine.

### Escrow Invariants
1. **Collateral Lock**: Project maintainers reserve funding upfront before tasks enter the Work Frontier.
2. **Deterministic Release**: Funds cannot be released by human discretion; release requires cryptographic `Decision` and `Proof` signatures logged on the ledger.
3. **Dispute Resolution Clock**: If a verifier objects or an incident is filed, settlement pauses for 48 hours to evaluate regression evidence.

### Fiat Onramp (Stripe / MCP)
1. Maintainer runs:
   ```bash
   vwl economics deposit --project horia-ai/vwl --amount 2500000
   ```
2. Or interacts via MCP Stripe bridge:
   ```bash
   vwl mcp stripe deposit --project horia-ai/vwl --amount-usdc 2500
   ```
3. Deposited fiat is minted or converted to USDC in the ProjectEscrow contract.

## 5. Auditing & Fee Monitoring
The team can audit accumulated treasury balances at any time via CLI:
```bash
vwl economics status
vwl wallet
```
To view real-time protocol fee accrual and ledger distributions across all active projects:
```bash
vwl market scan --json
```
