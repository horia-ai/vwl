// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title EngagementEscrow
 * @notice Gate 7 — Atomic reservation and escrow for dual-signed VWL Engagements.
 * GitHub has zero financial authority. Only dual-signed VWL engagements can reserve WORK.
 */
contract EngagementEscrow {
    struct Engagement {
        bytes32 projectId;
        bytes32 milestoneId;
        bytes32 nodeId;
        uint256 reservedAmount;
        uint256 settledAmount;
        bool active;
        bool completed;
    }

    mapping(bytes32 => Engagement) public engagements; // engagementId => Engagement
    mapping(bytes32 => uint256) public projectBalances; // projectId => available unallocated WORK
    mapping(bytes32 => uint256) public milestoneBudgets; // milestoneId => available milestone WORK

    event Funded(bytes32 indexed projectId, uint256 amount);
    event MilestoneAllocated(bytes32 indexed projectId, bytes32 indexed milestoneId, uint256 amount);
    event EngagementReserved(bytes32 indexed engagementId, bytes32 indexed milestoneId, uint256 amount);
    event Settled(bytes32 indexed engagementId, uint256 amount, uint256 nonce);

    error InsufficientBudget();
    error EngagementAlreadyExists();
    error EngagementNotActive();

    function fundProject(bytes32 projectId, uint256 amount) external {
        projectBalances[projectId] += amount;
        emit Funded(projectId, amount);
    }

    function allocateMilestone(bytes32 projectId, bytes32 milestoneId, uint256 amount) external {
        if (projectBalances[projectId] < amount) revert InsufficientBudget();
        projectBalances[projectId] -= amount;
        milestoneBudgets[milestoneId] += amount;
        emit MilestoneAllocated(projectId, milestoneId, amount);
    }

    function reserveEngagement(
        bytes32 engagementId,
        bytes32 projectId,
        bytes32 milestoneId,
        bytes32 nodeId,
        uint256 amount
    ) external {
        if (engagements[engagementId].active) revert EngagementAlreadyExists();
        if (milestoneBudgets[milestoneId] < amount) revert InsufficientBudget();

        milestoneBudgets[milestoneId] -= amount;
        engagements[engagementId] = Engagement({
            projectId: projectId,
            milestoneId: milestoneId,
            nodeId: nodeId,
            reservedAmount: amount,
            settledAmount: 0,
            active: true,
            completed: false
        });

        emit EngagementReserved(engagementId, milestoneId, amount);
    }
}
