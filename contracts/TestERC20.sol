// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import { ERC20 } from "openzeppelin-solidity/contracts/token/ERC20/ERC20.sol";
import { IERC20 } from "openzeppelin-solidity/contracts/token/ERC20/IERC20.sol";

contract TestERC20 is ERC20 {
    // TODO: premint
    constructor(uint256 _amount)
        ERC20("Test Token", "TEST")
    {
        _mint(_msgSender(), _amount);
    }
}