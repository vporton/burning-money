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
    const day = 19205;
    // The formula is 2^(-bx+c)
    const b = 1/(2 * 365.25); // 2 times per 2 years
    const c = Math.log2(2**63) + b * day;
    console.log(b, c)
    console.log(b * 2**64, c * 2**64)
    const token = await myDeploy(
        Token, network, deployer, "Token",
        [
            BN.from('25252216391115060'),
            BN.from('1647113692435066400000'),
            forwarder,
            "CardToken", "CT",
        ],
    );
    console.log(`Token contract at ${await token.address}`)

    const feeManager = await ethers.getContractAt("CentralisedFeeManager", getAddress(network.name, "CentralisedFeeManager"))
    await feeManager.setTokenAllowed(getAddress(network.name, "Token"), true);
};
module.exports.tags = ['Token'];
module.exports.dependencies = ['External', 'TrustedForwarder', 'FeeManager'];