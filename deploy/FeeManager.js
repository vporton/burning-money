const fs = require('fs');
const { myDeploy } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    const FeeManager = await ethers.getContractFactory("CentralisedFeeManager");
    const feeManager = await myDeploy(
        FeeManager, network, deployer, "CentralisedFeeManager",
        [
            deployer,
            0,
        ],
    );
};
module.exports.tags = ['FeeManager'];