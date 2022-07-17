// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";

contract TestPriceOracle is AggregatorV3Interface {
    function decimals() external pure returns (uint8) {
        return 10;
    }

    function description() external pure returns (string memory) {
        return "Test price oracle";
    }

    function version() external pure returns (uint256) {
        return 0; // correct?
    }

    // getRoundData and latestRoundData should both raise "No data present"
    // if they do not have data to report, instead of returning unset values
    // which could be misinterpreted as actual reported values.
    function getRoundData(uint80 _roundId)
    external
    pure
    returns (
        uint80 roundId,
        int256 answer,
        uint256 startedAt,
        uint256 updatedAt,
        uint80 answeredInRound
    ) {
        return (1, 10 * 10**10, 1, 1, 1); // duplicate code
    }

    function latestRoundData()
    external
    pure
    returns (
        uint80 roundId,
        int256 answer,
        uint256 startedAt,
        uint256 updatedAt,
        uint80 answeredInRound
    ) {
        return (1, 10 * 10**10, 1, 1, 1); // duplicate code
    }
}
