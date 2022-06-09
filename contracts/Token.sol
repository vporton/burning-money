// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ERC2771Context } from "@openzeppelin/contracts/metatx/ERC2771Context.sol";
import { Context } from "@openzeppelin/contracts/utils/Context.sol";
import { ABDKMath64x64 } from "abdk-libraries-solidity/ABDKMath64x64.sol";

contract Token is ERC20, ERC2771Context, Ownable {
    using ABDKMath64x64 for int128;

    mapping (address => address) public referrals;
    mapping (IERC20 => int128) public collaterals; // token => growth rate
    address beneficiant;
    bool disabledMint; // for more trust of users

    constructor(
        address trustedForwarder_,
        address _beneficiant,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(trustedForwarder_) ERC20(_name, _symbol)
    {
        beneficiant = _beneficiant;
    }

    function setCollateral(IERC20 _collateral, int128 _growthRate) public onlyOwner {
        collaterals[_collateral] = _growthRate;
    }

    function changeBeneficiant(address _beneficiant) public onlyOwner {
        beneficiant = _beneficiant;
    }

    function disableMint() public onlyOwner {
        disabledMint = true;
    }

    function setReferral(address _user, address _referral) public {
        require(referrals[_user] == address(0));
        referrals[_user] = _referral;
    }

    function mint(address account, uint256 amount) public onlyOwner {
        require(!disabledMint);
        _mint(account, amount);
        address _referral = referrals[account];
        _mint(_referral, amount / 4);  // 25% first level referral
        _referral = referrals[_referral];
        _mint(_referral, amount / 10); // 10% second level referral
    }

    function buyForCollateral(address _account, IERC20 _collateral, uint256 _collateral_amount) public {
        int128 _growthRate = collaterals[_collateral];
        require(_growthRate != 0, "Collateral not supported");
        int128 _ourTokenAmount = _growthRate.mul(int128(uint128(block.timestamp))).exp_2();
        _mint(_account, _collateral_amount * uint256(int256(_ourTokenAmount)));
        _collateral.transfer(beneficiant, _collateral_amount);
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return ERC2771Context._msgSender();
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return ERC2771Context._msgData();
    }
}