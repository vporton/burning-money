const fs = require('fs');

module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[networkName];
    const FeeManager = await ethers.getContractFactory("CentralisedFeeManager");
    feeManager = await FeeManager.deploy(addresses.collateral, 0);
    await feeManager.deployed();
    await feeManager.setTokenAllowed(addresses.collateral, true);

};
module.exports.tags = ['FeeManager'];