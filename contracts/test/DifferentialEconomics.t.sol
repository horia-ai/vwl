// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "../contracts/EngagementEscrow.sol";
import "../contracts/WorkToken.sol";
import "../contracts/NodeRegistry.sol";

/**
 * @title DifferentialEconomicsTest
 * @notice Gate 7 (WU-G7-001, WU-G7-002, WU-G7-007)
 * Differential model testing between Rust reference implementation (EconomicLedger)
 * and EVM Solidity smart contracts (EngagementEscrow, WorkToken, NodeRegistry).
 */
contract DifferentialEconomicsTest {
    EngagementEscrow public escrow;
    WorkToken public work;
    NodeRegistry public registry;

    bytes32 public constant PROJECT_ID = keccak256("horia-ai/vwl");
    bytes32 public constant MILESTONE_ID = keccak256("G5 - Local Economics");
    bytes32 public constant NODE_ID = keccak256("node-gemini-dev");
    bytes32 public constant ENGAGEMENT_ID = keccak256("ENG-TASK-001");

    uint256 public constant WORK_PER_USD = 1_000;
    uint256 public constant DEPOSIT_USD = 1_000; // $1,000
    uint256 public constant EXPECTED_WORK_MINTED = 1_000_000; // 1,000,000 WORK
    uint256 public constant RESERVED_WORK = 250_000; // 250,000 WORK

    function setUp() public {
        escrow = new EngagementEscrow();
        work = new WorkToken(address(escrow), 0);
        registry = new NodeRegistry();
    }

    /**
     * @notice Test Step 1: Fiat Deposit & Conservation Invariant (Matching Rust test_fiat_deposit_and_conservation)
     */
    function testDifferentialDepositAndConservation() public {
        uint256 workAmount = DEPOSIT_USD * WORK_PER_USD;
        escrow.fundProject(PROJECT_ID, workAmount);

        require(escrow.projectBalances(PROJECT_ID) == EXPECTED_WORK_MINTED, "Project balance mismatch");
        require(escrow.totalDeposited() == EXPECTED_WORK_MINTED, "Total deposited mismatch");

        // Verify conservation
        bool holds = escrow.checkConservation(PROJECT_ID, MILESTONE_ID, ENGAGEMENT_ID);
        require(holds, "Conservation invariant must hold after deposit");
    }

    /**
     * @notice Test Step 2 & 3: Atomic Reservation & Settlement Sequence
     * (Matching Rust test_engagement_lifecycle_and_conservation_invariant)
     */
    function testDifferentialLifecycleAndStateTransitions() public {
        // 1. Deposit $1,000 -> 1,000,000 WORK
        uint256 workAmount = DEPOSIT_USD * WORK_PER_USD;
        escrow.fundProject(PROJECT_ID, workAmount);

        // 2. Allocate to Milestone: 500,000 WORK
        uint256 milestoneAmount = 500_000;
        escrow.allocateMilestone(PROJECT_ID, MILESTONE_ID, milestoneAmount);
        require(escrow.projectBalances(PROJECT_ID) == 500_000, "Remaining project balance mismatch");
        require(escrow.milestoneBudgets(MILESTONE_ID) == 500_000, "Milestone budget mismatch");

        // Conservation check
        require(escrow.checkConservation(PROJECT_ID, MILESTONE_ID, ENGAGEMENT_ID), "Invariant after milestone allocation");

        // 3. Reserve Engagement: 250,000 WORK
        escrow.reserveEngagement(ENGAGEMENT_ID, PROJECT_ID, MILESTONE_ID, NODE_ID, RESERVED_WORK);
        require(escrow.milestoneBudgets(MILESTONE_ID) == 250_000, "Milestone budget after reserve mismatch");

        (
            bytes32 pId,
            bytes32 mId,
            bytes32 nId,
            uint256 resAmt,
            uint256 setAmt,
            bool active,
            bool completed
        ) = escrow.engagements(ENGAGEMENT_ID);

        require(pId == PROJECT_ID, "Engagement project mismatch");
        require(mId == MILESTONE_ID, "Engagement milestone mismatch");
        require(nId == NODE_ID, "Engagement node mismatch");
        require(resAmt == RESERVED_WORK, "Reserved amount mismatch");
        require(setAmt == 0, "Initial settled amount must be 0");
        require(active && !completed, "Engagement must be active and uncompleted");

        // Conservation check
        require(escrow.checkConservation(PROJECT_ID, MILESTONE_ID, ENGAGEMENT_ID), "Invariant after reservation");

        // 4. Settle Engagement upon confirmed verification
        uint256 nonce = 1;
        escrow.settleEngagement(ENGAGEMENT_ID, nonce);

        (, , , uint256 postRes, uint256 postSet, bool postActive, bool postCompleted) = escrow.engagements(ENGAGEMENT_ID);
        require(postRes == 0, "Post-settlement reserved must be 0");
        require(postSet == RESERVED_WORK, "Post-settlement settled must equal reserved amount");
        require(postActive && postCompleted, "Engagement must be active and completed");
        require(escrow.totalDisbursed() == RESERVED_WORK, "Total disbursed mismatch");

        // Final conservation check
        require(escrow.checkConservation(PROJECT_ID, MILESTONE_ID, ENGAGEMENT_ID), "Invariant after settlement");
    }

    /**
     * @notice Test Revert on Over-Draft / Unauthorized Inflation
     */
    function testRevertOnInsufficientBudget() public {
        // Fund only 10,000 WORK
        escrow.fundProject(PROJECT_ID, 10_000);

        // Attempting to allocate 20,000 WORK must revert
        try escrow.allocateMilestone(PROJECT_ID, MILESTONE_ID, 20_000) {
            revert("Overdraft allocation should have reverted");
        } catch {
            // Expected
        }
    }

    /**
     * @notice Test Revert on Duplicate Reservation
     */
    function testRevertOnDuplicateReservation() public {
        escrow.fundProject(PROJECT_ID, 500_000);
        escrow.allocateMilestone(PROJECT_ID, MILESTONE_ID, 500_000);

        escrow.reserveEngagement(ENGAGEMENT_ID, PROJECT_ID, MILESTONE_ID, NODE_ID, 100_000);

        // Attempt duplicate reservation with same engagement ID must revert
        try escrow.reserveEngagement(ENGAGEMENT_ID, PROJECT_ID, MILESTONE_ID, NODE_ID, 100_000) {
            revert("Duplicate engagement reservation should have reverted");
        } catch {
            // Expected
        }
    }

    /**
     * @notice Test Revert on Double Settlement
     */
    function testRevertOnDoubleSettlement() public {
        escrow.fundProject(PROJECT_ID, 500_000);
        escrow.allocateMilestone(PROJECT_ID, MILESTONE_ID, 500_000);
        escrow.reserveEngagement(ENGAGEMENT_ID, PROJECT_ID, MILESTONE_ID, NODE_ID, 100_000);

        escrow.settleEngagement(ENGAGEMENT_ID, 1);

        // Second settlement attempt must revert
        try escrow.settleEngagement(ENGAGEMENT_ID, 2) {
            revert("Double settlement should have reverted");
        } catch {
            // Expected
        }
    }
}
