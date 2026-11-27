# VWL DEX Integration & Liquidity Setup Guide (Gate 10)

> **Restricted Developer Documentation**  
> Interactive Token-Gated Web Portal: [https://horia-ai.github.io/vwl/dex-guide.html](https://horia-ai.github.io/vwl/dex-guide.html)

---

## 1. Selected DEX: Uniswap v3 (Ethereum Sepolia / Base Sepolia)

VWL standardizes on **Uniswap v3** as the primary respectable decentralized exchange venue for Gate 10 public liquidity evaluations:
- **Capital Efficiency**: Concentrated liquidity allows tight price band allocations ($0.00095 &mdash; $0.00105), minimizing slippage on labor compensation swaps.
- **Security & Non-Custodial Pool**: Audited immutable core contracts guarantee no admin backdoors or custodial hold over pool reserves.
- **Manipulation Resistance**: Geometric time-weighted average price (TWAP) oracles prevent single-block flash loan arbitrage.

---

## 2. Denomination & Price Ratio Invariant

In Gate 5 & Gate 9, the baseline conversion is established as:
$$\text{1.00 USD} = 1,000\ \text{WORK} \iff 1\ \text{WORK} = 0.001\ \text{USD}$$

When pairing against **USDC** (6 decimals) and **WORK** (18 decimals):
- Target Price: $P = \frac{\text{USDC}}{\text{WORK}} = \frac{10^6 \times 10^{-3}}{10^{18}} = 10^{-15}$ in raw token units.
- $\sqrt{P} \cdot 2^{96} \approx 2,505,414,483,753,750,000$ (`sqrtPriceX96` target for initial pool tick).

---

## 3. Smart Contracts & Testing

The contracts are provided under `contracts/`:
- [`contracts/WorkToken.sol`](file:///Users/trifan/DEVELOPMENT/vwl/contracts/contracts/WorkToken.sol): ERC-20 Labor Asset with EIP-2612 Permit, total supply management, and escrow-restricted minting.
- [`contracts/MockUSDC.sol`](file:///Users/trifan/DEVELOPMENT/vwl/contracts/contracts/MockUSDC.sol): 6-decimal test stablecoin for testnets.
- [`test/DEXPoolIntegration.t.sol`](file:///Users/trifan/DEVELOPMENT/vwl/contracts/test/DEXPoolIntegration.t.sol): Comprehensive simulation testing swap execution, fee extraction (0.3% tier), and price conservation.

### Running Local Tests
```bash
cd contracts
forge test -vvv --match-contract DEXPoolIntegrationTest
```

---

## 4. Testnet Deployment Procedure

### Step 1: Deploy Contracts to Base Sepolia
```bash
export RPC_URL="https://sepolia.base.org"
export PRIVATE_KEY="<your_private_key>"

# Deploy MockUSDC
forge create contracts/MockUSDC.sol:MockUSDC \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY

# Deploy WorkToken with 100M initial supply
forge create contracts/WorkToken.sol:WorkToken \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args 0x0000000000000000000000000000000000000001 100000000000000000000000000
```

### Step 2: Initialize Uniswap v3 Pool via Cast
```bash
# UniswapV3Factory on Base Sepolia / Sepolia: 0x0227628f3F023674B2267BE08c790D01917f96b9
cast send 0x0227628f3F023674B2267BE08c790D01917f96b9 \
  "createPool(address,address,uint24)" \
  $WORK_ADDRESS $USDC_ADDRESS 3000 \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY
```

---

## 5. Gate 10 Go / No-Go Checklist

As defined in the specification, public liquidity is strictly optional and must satisfy:
- [ ] **Arbitrage Protection**: Fixed 1000:1 issuance cannot remain an arbitrageable public promise once WORK floats.
- [ ] **Security Audit**: Core contracts must complete formal verification with 0 high/critical issues.
- [ ] **Productive Transfer Ratio (PTR)**: Verifiable organic labor transactions must exceed speculative liquidity churn.
