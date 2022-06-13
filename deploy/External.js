const fs = require('fs');
const { ethers } = require('hardhat');
const { myDeploy, getAddress } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    if(addresses.collateral) {
        updateAddress(network.name, "collateral", addresses.collateral);
    } else if(process.env['TEST']) {
        const TestErc20 = await ethers.getContractFactory("TestERC20");
        const testErc20 = await myDeploy(
            TestErc20, network, deployer, "collateral",
            [ethers.utils.parseEther('10000')],
        );    
    }
};
module.exports.tags = ['External'];
