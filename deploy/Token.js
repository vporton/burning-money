const fs = require('fs');
const { ethers } = require('hardhat');
const { myDeploy, getAddress } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    const forwarder = getAddress(network.name, "BiconomyForwarder");
    const Token = await ethers.getContractFactory("Token");
    const token = await myDeploy(
        Token, network, deployer, "Token",
        [forwarder, addresses.beneficiant, "World Token", "WT"],
    );

    const feeManager = await ethers.getContractAt("CentralisedFeeManager", getAddress(network.name, "CentralisedFeeManager"))
    await feeManager.setTokenAllowed(getAddress(network.name, "Token"), true);
};
module.exports.tags = ['Token'];
module.exports.dependencies = ['TrustedForwarder', 'FeeManager'];