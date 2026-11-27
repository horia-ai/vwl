// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title WorkToken (WORK)
 * @notice Gate 7 & Gate 10 — ERC-20 Labor Asset for the Virtual Work Ledger (VWL).
 * Designed for programmatic settlement and pairing with Uniswap v3 / reputable DEX pools.
 * Includes EIP-2612 permit for gasless approvals.
 */
contract WorkToken {
    string public constant name = "Virtual Work Token";
    string public constant symbol = "WORK";
    uint8 public constant decimals = 18;

    uint256 public totalSupply;
    address public immutable escrowSettlement;
    address public owner;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    mapping(address => uint256) public nonces;

    // EIP-712 Permit
    bytes32 public immutable DOMAIN_SEPARATOR;
    bytes32 public constant PERMIT_TYPEHASH =
        keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event EscrowMint(address indexed recipient, uint256 amount);

    error Unauthorized();
    error InsufficientBalance();
    error InsufficientAllowance();
    error ExpiredDeadline();
    error InvalidSigner();

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor(address _escrowSettlement, uint256 initialSupply) {
        owner = msg.sender;
        escrowSettlement = _escrowSettlement;

        DOMAIN_SEPARATOR = keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                keccak256(bytes("1")),
                block.chainid,
                address(this)
            )
        );

        if (initialSupply > 0) {
            totalSupply = initialSupply;
            balanceOf[msg.sender] = initialSupply;
            emit Transfer(address(0), msg.sender, initialSupply);
        }
    }

    function transfer(address to, uint256 value) external returns (bool) {
        return _transfer(msg.sender, to, value);
    }

    function approve(address spender, uint256 value) external returns (bool) {
        allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }

    function transferFrom(address from, address to, uint256 value) external returns (bool) {
        uint256 currentAllowance = allowance[from][msg.sender];
        if (currentAllowance != type(uint256).max) {
            if (currentAllowance < value) revert InsufficientAllowance();
            allowance[from][msg.sender] = currentAllowance - value;
        }
        return _transfer(from, to, value);
    }

    function _transfer(address from, address to, uint256 value) internal returns (bool) {
        if (to == address(0)) revert Unauthorized();
        if (balanceOf[from] < value) revert InsufficientBalance();

        balanceOf[from] -= value;
        balanceOf[to] += value;
        emit Transfer(from, to, value);
        return true;
    }

    /**
     * @notice Escrow-controlled minting upon confirmed Work Unit settlement attestations (Gate 7).
     */
    function mintFromSettlement(address recipient, uint256 amount) external {
        if (msg.sender != escrowSettlement && msg.sender != owner) revert Unauthorized();
        totalSupply += amount;
        balanceOf[recipient] += amount;
        emit Transfer(address(0), recipient, amount);
        emit EscrowMint(recipient, amount);
    }

    /**
     * @notice EIP-2612 gasless approval
     */
    function permit(
        address ownerAddress,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        if (block.timestamp > deadline) revert ExpiredDeadline();

        bytes32 structHash = keccak256(
            abi.encode(PERMIT_TYPEHASH, ownerAddress, spender, value, nonces[ownerAddress]++, deadline)
        );

        bytes32 digest = keccak256(abi.encodePacked("\x19\x01", DOMAIN_SEPARATOR, structHash));
        address signer = ecrecover(digest, v, r, s);

        if (signer == address(0) || signer != ownerAddress) revert InvalidSigner();

        allowance[ownerAddress][spender] = value;
        emit Approval(ownerAddress, spender, value);
    }
}
