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

    int128 public growthRate;
    mapping (address => address) public referrals;
    mapping (uint => mapping(address => uint256)) public bids; // time => (address => bid)
    mapping (uint => uint256) public totalBids; // time => total bid

    constructor(
        int128 _growthRate,
        address _trustedForwarder,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(_trustedForwarder) ERC20(_name, _symbol)
    {
        growthRate = _growthRate;
        emit TokenCreated(_growthRate, _trustedForwarder, _name, _symbol);
    }

    function setReferral(address _referral) public {
        require(referrals[_msgSender()] == address(0));
        referrals[_msgSender()] = _referral;
        emit SetReferral(_msgSender(), _referral);
    }

    // FIXME: It seems that this can produce an unlimited amount of money.
    function _mint(address _account, uint256 _amount) internal override {
        ERC20._mint(_account, _amount);
        address _referral = referrals[_account];
        if (_referral != address(0)) {
            ERC20._mint(_referral, _amount / 4);  // 25% first level referral
            _referral = referrals[_referral];
            if (_referral != address(0)) {
                ERC20._mint(_referral, _amount / 10); // 10% second level referral
            }
        }
        emit OurMint(_msgSender(), _account, _amount);
    }

    /// `_time` must be a multiple of 24*3600, otherwise the bid is lost.
    /// Need to approve this contract for transfers of collateral before calling this function.
    function bidOn(uint _day, address _for) public payable {
        uint256 _collateralAmount = msg.value;
        uint _curDay = block.timestamp / (24*3600);
        require(_curDay < _day, "You bade too late");
        totalBids[_day] += msg.value; // Solidity 0.8 overflow protection
        unchecked { // Overflow checked by the previous statement.
            bids[_day][_for] += msg.value;
        }
        payable(0).transfer(msg.value);
        emit Bid(_msgSender(), _for, _day, _collateralAmount);
    }

    function withdrawalAmount(uint _day) public view returns(uint256) {
        int128 _ourTokenAmount = growthRate.mul(int128(uint128(_day))).exp_2();
        int128 _share = ABDKMath64x64.divu(bids[_day][_msgSender()], totalBids[_day]);
        return uint256(int256(_ourTokenAmount.mul(_share)));
    }

    // Some time in the future overflow will happen.
    function withdraw(uint _day, address _account) public {
        require(block.timestamp >= _day * (24*3600), "Too early to withdraw");
        uint256 _amount = withdrawalAmount(_day);
        _mint(_account, _amount);
        bids[_day][_account] = 0;
        emit Withdraw(_msgSender(), _day, _account, _amount);
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return ERC2771Context._msgSender();
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return ERC2771Context._msgData();
    }

    event TokenCreated(
        int128 growthRate,
        address trustedForwarder,
        string name,
        string symbol
    );
    event SetReferral(address sender, address referral);
    event OurMint(address sender, address account, uint256 amount);
    event Bid(address sender, address for_, uint day, uint256 amount);
    event Withdraw(address sender, uint day, address account, uint256 amount);
}