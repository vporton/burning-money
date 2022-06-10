const fs = require('fs');

module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[networkName];
    const forwarder = await deployments.get("BiconomyForwarder");
    await deploy('Token', {
        from: deployer,
        args: [forwarder, addresses.beneficiant, "World Token", "WT"],
        log: true,
    });
  };
  module.exports.tags = ['Token'];
  module.exports.dependencies = ['TrustedForwarder'];