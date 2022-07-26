const fs = require('fs');
const { ethers } = require('hardhat');
const { myDeploy, updateAddress } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    if(addresses.collateralOracle) {
        updateAddress(network.name, "collateralOracle", addresses.collateralOracle);
    } else if(process.env['TEST']) {
        const TestPriceOracle = await ethers.getContractFactory("TestPriceOracle");
        const testPriceOracle = await myDeploy(
            TestPriceOracle, network, deployer, "collateralOracle", [],
        );    
    }
};
module.exports.tags = ['External'];
