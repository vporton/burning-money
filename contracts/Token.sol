// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ERC2771Context } from "@openzeppelin/contracts/metatx/ERC2771Context.sol";
import { Context } from "@openzeppelin/contracts/utils/Context.sol";
// import { ABDKMath64x64 } from "abdk-libraries-solidity/ABDKMath64x64.sol";

contract Token is ERC20, Ownable, ERC2771Context {
    // using ABDKMath64x64 for int128;

    mapping (address => address) public referrals;

    constructor(address trustedForwarder_, string memory name_, string memory symbol_)
        ERC2771Context(trustedForwarder_) ERC20(name_, symbol_)
    { }

    function setReferral(address _user, address _referral) external {
        require(referrals[_user] == address(0));
        referrals[_user] = _referral;
    }

    function mint(address account, uint256 amount) external onlyOwner {
        _mint(account, amount);
        address _referral = referrals[account];
        _mint(_referral, amount / 4);  // 25% first level referral
        _referral = referrals[_referral];
        _mint(_referral, amount / 5); // 10% second level referral
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return msg.data;
    }
}