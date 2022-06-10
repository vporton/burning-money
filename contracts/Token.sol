// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "openzeppelin-solidity/contracts/token/ERC20/ERC20.sol";
import { IERC20 } from "openzeppelin-solidity/contracts/token/ERC20/IERC20.sol";
import { Ownable } from "openzeppelin-solidity/contracts/access/Ownable.sol";
import { ERC2771Context } from "openzeppelin-solidity/contracts/metatx/ERC2771Context.sol";
import { Context } from "openzeppelin-solidity/contracts/utils/Context.sol";
import { ABDKMath64x64 } from "abdk-libraries-solidity/ABDKMath64x64.sol";
import "@chainlink/contracts/src/v0.8/interfaces/FeedRegistryInterface.sol";
import "@chainlink/contracts/src/v0.8/Denominations.sol";

/**
 * @title Simple Interface to interact with Universal Client Contract
 * @notice Client Address 0x8ea35EdC1709ea0Ea2C86241C7D1C84Fd0dDeB11
 */
interface ChainlinkInterface {

  /**
   * @notice Creates a Chainlink request with the job specification ID,
   * @notice and sends it to the Oracle.
   * @notice _oracle The address of the Oracle contract fixed top
   * @notice _payment For this example the PAYMENT is set to zero
   * @param _jobId The job spec ID that we want to call in string format
   */
    function requestPrice(string calldata _jobId) external;

    function currentPrice() external view returns (uint);

}

contract Token is ERC20, ERC2771Context, Ownable {
    using ABDKMath64x64 for int128;

    ChainlinkInterface internal registry;
    address internal DotTokenId;
    mapping (address => address) public referrals;
    mapping (IERC20 => int128) public collaterals; // token => growth rate
    address beneficiant;

    constructor(
        ChainlinkInterface _registry, // 0x6f6371a780324b90aaf195a0d39c723c // DOT to USD // FIXME: Instead use GLMR
        address trustedForwarder_,
        address _beneficiant,
        string memory _name,
        string memory _symbol
    )
        ERC2771Context(trustedForwarder_) ERC20(_name, _symbol)
    {
        registry = _registry;
        beneficiant = _beneficiant;
    }

    function setCollateral(IERC20 _collateral, int128 _growthRate) public onlyOwner {
        collaterals[_collateral] = _growthRate;
    }

    function changeBeneficiant(address _beneficiant) public onlyOwner {
        beneficiant = _beneficiant;
    }

    function setReferral(address _user, address _referral) public {
        require(referrals[_user] == address(0));
        referrals[_user] = _referral;
    }

    function getCollateralPrice() internal view returns (uint256) {
        return registry.currentPrice();
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

    function buyForCollateral(address _account, IERC20 _collateral, uint256 _collateral_amount) public {
        int128 _growthRate = collaterals[_collateral];
        require(_growthRate != 0, "Collateral not supported");
        int128 _ourTokenAmount = _growthRate.mul(int128(uint128(block.timestamp))).exp_2();
        int128 _price = _ourTokenAmount.mul(10000_0000).div(int128(uint128(getCollateralPrice() * (1<<128))));
        _mint(_account, _collateral_amount * uint256(int256(_price)));
        _collateral.transfer(beneficiant, _collateral_amount);
    }

    function _msgSender() internal view virtual override(Context, ERC2771Context) returns (address) {
        return ERC2771Context._msgSender();
    }

    function _msgData() internal view virtual override(Context, ERC2771Context) returns (bytes calldata) {
        return ERC2771Context._msgData();
    }
}