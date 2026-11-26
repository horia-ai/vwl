// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title NodeRegistry
 * @notice Gate 7 — Node credential / operational key anchoring for VWL.
 * Authority over code correctness and REP remains in the off-chain cryptographic ledger.
 */
contract NodeRegistry {
    struct Node {
        address controller;
        bytes32 operationalPubKey;
        uint256 registeredAt;
        bool active;
    }

    mapping(bytes32 => Node) public nodes; // nodeId (hash) => Node

    event NodeRegistered(bytes32 indexed nodeId, address indexed controller, bytes32 operationalPubKey);
    event NodeKeyRotated(bytes32 indexed nodeId, bytes32 newOperationalPubKey);

    error NodeAlreadyExists(bytes32 nodeId);
    error NodeNotFound(bytes32 nodeId);
    error Unauthorized(address sender);

    function registerNode(bytes32 nodeId, bytes32 operationalPubKey) external {
        if (nodes[nodeId].active) revert NodeAlreadyExists(nodeId);
        nodes[nodeId] = Node({
            controller: msg.sender,
            operationalPubKey: operationalPubKey,
            registeredAt: block.timestamp,
            active: true
        });
        emit NodeRegistered(nodeId, msg.sender, operationalPubKey);
    }

    function rotateKey(bytes32 nodeId, bytes32 newOperationalPubKey) external {
        Node storage node = nodes[nodeId];
        if (!node.active) revert NodeNotFound(nodeId);
        if (node.controller != msg.sender) revert Unauthorized(msg.sender);
        node.operationalPubKey = newOperationalPubKey;
        emit NodeKeyRotated(nodeId, newOperationalPubKey);
    }
}
