const fs = require('fs');
const { ethers } = require('hardhat');
const { myDeploy, getAddress } = require('../lib/default-deployer');
const BN = ethers.BigNumber;

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    const forwarder = getAddress(network.name, "BiconomyForwarder");
    const Token = await ethers.getContractFactory("Token");
    const token = await myDeploy(
        Token, network, deployer, "Token",
        [
            getAddress(networkName, 'collateral'),
            Math.floor(BN.from(1).div(BN.from(2 * 24*3600*365.25)).mul(BN.from(2).pow(BN.from(64)))), // 2 times per 2 years
            forwarder,
            addresses.beneficiant,
            "World Token", "WT",
        ],
    );

    const feeManager = await ethers.getContractAt("CentralisedFeeManager", getAddress(network.name, "CentralisedFeeManager"))
    await feeManager.setTokenAllowed(getAddress(network.name, "Token"), true);
};
module.exports.tags = ['Token'];
module.exports.dependencies = ['External', 'TrustedForwarder', 'FeeManager'];