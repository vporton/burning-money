// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ERC2771Context } from "@openzeppelin/contracts/metatx/ERC2771Context.sol";
import { Context } from "@openzeppelin/contracts/utils/Context.sol";
import { ABDKMath64x64 } from "abdk-libraries-solidity/ABDKMath64x64.sol";

contract Token is ERC20, Ownable, ERC2771Context {
    using ABDKMath64x64 for int128;

    mapping (address => address) public referrals;
    IERC20 collateral;
    address beneficiant;
    int128 growthRate;

    constructor(
        address _trustedForwarder,
        IERC20 _collateral,
        address _beneficiant,
        int128 _growthRate,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(_trustedForwarder) ERC20(_name, _symbol)
    {
        collateral = _collateral;
        beneficiant = _beneficiant;
        growthRate = _growthRate;
    }

    function changeBeneficiant(address _beneficiant) public onlyOwner {
        beneficiant = _beneficiant;
    }

    function setReferral(address _user, address _referral) public {
        require(referrals[_user] == address(0));
        referrals[_user] = _referral;
    }

    function mint(address account, uint256 amount) public onlyOwner {
        _mint(account, amount);
        address _referral = referrals[account];
        _mint(_referral, amount / 4);  // 25% first level referral
        _referral = referrals[_referral];
        _mint(_referral, amount / 10); // 10% second level referral
    }

    function buyForCollateral(address _account, uint256 _collateral_amount) public {
        int128 ourTokenAmount = growthRate.mul(int128(uint128(block.timestamp))).exp_2();
        _mint(_account, _collateral_amount * uint256(int256(ourTokenAmount)));
        collateral.transfer(beneficiant, _collateral_amount);
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return msg.sender;
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return msg.data;
    }
}