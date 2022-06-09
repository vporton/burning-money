const fs = require('fs');

module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = fs.readFileSync('addresses.json')[networkName];
    await deploy('Token', {
        from: deployer,
        args: [addresses.TrustedForwarder, addresses.beneficiant, "World Token", "WT"],
        log: true,
    });
  };
  module.exports.tags = ['Token'];