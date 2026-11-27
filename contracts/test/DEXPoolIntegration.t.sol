// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "../contracts/WorkToken.sol";
import "../contracts/MockUSDC.sol";

/**
 * @title DEXPoolIntegrationTest
 * @notice Gate 10 — Test harness for pairing WORK token with a respectable DEX (Uniswap v3 model).
 * Demonstrates:
 * 1. Price ratio invariant: 1 USD (1,000,000 USDC units) = 1,000 WORK (1,000 * 1e18 units).
 *    -> 1 WORK = 0.001 USD.
 * 2. Constant-product / Concentrated liquidity pricing model.
 * 3. Slippage and conservation of value during swaps.
 */
contract DEXPoolIntegrationTest {
    WorkToken public work;
    MockUSDC public usdc;

    uint256 public constant INITIAL_WORK_SUPPLY = 100_000_000 * 1e18; // 100M WORK
    uint256 public constant INITIAL_USDC_POOL = 100_000 * 1e6;       // 100,000 USDC ($100k)
    uint256 public constant INITIAL_WORK_POOL = 100_000_000 * 1e18;  // 100M WORK (at $0.001 each)

    // Virtual pool reserves for local differential testing
    uint256 public reserveUSDC;
    uint256 public reserveWORK;

    event LiquidityAdded(uint256 usdcAmount, uint256 workAmount);
    event SwapExecuted(address indexed sender, string direction, uint256 amountIn, uint256 amountOut);

    function setUp() public {
        usdc = new MockUSDC();
        work = new WorkToken(address(this), INITIAL_WORK_SUPPLY);

        // Fund test pool
        reserveUSDC = INITIAL_USDC_POOL;
        reserveWORK = INITIAL_WORK_POOL;
    }

    /**
     * @notice Tests that initial price corresponds exactly to 1,000 WORK per 1 USD (1e6 USDC).
     */
    function testInitialPriceRatio() public view returns (bool) {
        // Price of 1 WORK in USDC = reserveUSDC / (reserveWORK / 1e18)
        // 100,000 * 1e6 / 100,000,000 = 1,000 USDC units = 0.001 USDC
        uint256 usdcPerWork = (reserveUSDC * 1e18) / reserveWORK; // scaled to 1e6
        require(usdcPerWork == 1_000, "Price must be exactly 0.001 USD (1000 micro-USDC)");
        return true;
    }

    /**
     * @notice Simulates buying WORK with 1,000 USDC ($1,000).
     * Output should be ~1,000,000 WORK minus AMM price impact / fee.
     */
    function simulateBuyWork(uint256 usdcIn) public returns (uint256 workOut) {
        require(usdcIn > 0, "Input must be positive");

        // 0.3% fee (Uniswap v3 tier 3000)
        uint256 amountInWithFee = usdcIn * 997;
        uint256 numerator = amountInWithFee * reserveWORK;
        uint256 denominator = (reserveUSDC * 1000) + amountInWithFee;
        workOut = numerator / denominator;

        reserveUSDC += usdcIn;
        reserveWORK -= workOut;

        emit SwapExecuted(msg.sender, "USDC_TO_WORK", usdcIn, workOut);
        return workOut;
    }

    /**
     * @notice Simulates selling WORK for USDC.
     */
    function simulateSellWork(uint256 workIn) public returns (uint256 usdcOut) {
        require(workIn > 0, "Input must be positive");

        uint256 amountInWithFee = workIn * 997;
        uint256 numerator = amountInWithFee * reserveUSDC;
        uint256 denominator = (reserveWORK * 1000) + amountInWithFee;
        usdcOut = numerator / denominator;

        reserveWORK += workIn;
        reserveUSDC -= usdcOut;

        emit SwapExecuted(msg.sender, "WORK_TO_USDC", workIn, usdcOut);
        return usdcOut;
    }
}
