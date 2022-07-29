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
    int128 public shift;
    mapping (uint64 => mapping(address => uint256)) public bids; // time => (address => bid)
    mapping (uint64 => uint256) public totalBids; // time => total bid

    constructor(
        int128 _growthRate,
        int128 _shift,
        address _trustedForwarder,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(_trustedForwarder) ERC20(_name, _symbol)
    {
        growthRate = _growthRate;
        shift = _shift;
        emit TokenCreated(_growthRate, _shift, _trustedForwarder, _name, _symbol);
    }

    /// `_time` must be a multiple of 24*3600, otherwise the bid is lost.
    /// Need to approve this contract for transfers of collateral before calling this function.
    function bidOn(uint64 _day, address _for) public payable {
        uint256 _collateralAmount = msg.value;
        uint64 _curDay = uint64(block.timestamp / (24*3600));
        require(_curDay < _day, "You bade too late");
        totalBids[_day] += msg.value; // Solidity 0.8 overflow protection
        unchecked { // Overflow checked by the previous statement.
            bids[_day][_for] += msg.value;
        }
        payable(0).transfer(msg.value);
        emit Bid(_msgSender(), _for, _day, _collateralAmount);
    }

    function withdrawalAmount(uint64 _day) public view returns(uint256) {
        int128 _ourTokenAmount = growthRate.mul(int128(uint128(_day))).add(shift).exp_2();
        int128 _share = ABDKMath64x64.divu(bids[_day][_msgSender()], totalBids[_day]);
        return uint256(int256(_ourTokenAmount.mul(_share)));
    }

    // Some time in the future overflow will happen.
    function withdraw(uint64 _day, address _account) public {
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
        int128 shift,
        address trustedForwarder,
        string name,
        string symbol
    );
    event SetReferral(address sender, address referral);
    event OurMint(address sender, address account, uint256 amount);
    event Bid(address sender, address for_, uint64 day, uint256 amount);
    event Withdraw(address sender, uint day, address account, uint256 amount);
}