// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "openzeppelin-solidity/contracts/token/ERC20/ERC20.sol";
import { IERC20 } from "openzeppelin-solidity/contracts/token/ERC20/IERC20.sol";
import { Ownable } from "openzeppelin-solidity/contracts/access/Ownable.sol";
import { ERC2771Context } from "openzeppelin-solidity/contracts/metatx/ERC2771Context.sol";
import { Context } from "openzeppelin-solidity/contracts/utils/Context.sol";
import { ABDKMath64x64 } from "abdk-libraries-solidity/ABDKMath64x64.sol";

contract Token is ERC20, ERC2771Context, Ownable {
    using ABDKMath64x64 for int128;

    // TODO: Events.

    IERC20 public collateral;
    int128 public growthRate;
    mapping (address => address) public referrals;
    address public beneficiant;
    mapping (uint => mapping(address => uint256)) public bids; // time => (address => bid)
    mapping (uint => uint256) public totalBids; // address => total bid

    // TODO: premint
    constructor(
        IERC20 _collateral,
        int128 _growthRate,
        address _trustedForwarder,
        address _beneficiant,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(_trustedForwarder) ERC20(_name, _symbol)
    {
        collateral = _collateral;
        growthRate = _growthRate;
        beneficiant = _beneficiant;
    }

    function changeBeneficiant(address _beneficiant) public onlyOwner {
        beneficiant = _beneficiant;
    }

    function setReferral(address _user, address _referral) public {
        require(referrals[_user] == address(0));
        referrals[_user] = _referral;
    }

    function _mint(address account, uint256 amount) internal override {
        ERC20._mint(account, amount);
        address _referral = referrals[account];
        if (_referral != address(0)) {
            ERC20._mint(_referral, amount / 4);  // 25% first level referral
            _referral = referrals[_referral];
            if (_referral != address(0)) {
                ERC20._mint(_referral, amount / 10); // 10% second level referral
            }
        }
    }

     /// `_time` must be a multiple of 24*3600, otherwise the bid is lost.
    /// Need to approve this contract for transfers of collateral before calling this function.
    function bidOn(uint _time, uint256 _collateralAmount) public {
        require(block.timestamp < _time);
        totalBids[_time] += _collateralAmount; // Solidity 0.8 overflow protection
        bids[_time][_msgSender()] += _collateralAmount;
        collateral.transferFrom(_msgSender(), beneficiant, _collateralAmount);
    }

    // Some time in the future overflow will happen.
    function withdraw(uint _time, address _account) public {
        require(block.timestamp >= _time);
        require(_time % (24*3600) == 0);

        int128 _ourTokenAmount = growthRate.mul(int128(uint128(_time))).exp_2();
        int128 _share = ABDKMath64x64.divu(bids[_time][_msgSender()], totalBids[_time]);
        _mint(_account, uint256(int256(_ourTokenAmount.mul(_share))));
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return ERC2771Context._msgSender();
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return ERC2771Context._msgData();
    }
}