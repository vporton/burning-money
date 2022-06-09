module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    await deploy('Token', {
      from: deployer,
      args: [trustedForwarder, beneficiant, "World Token", "WT"],
      log: true,
    });
  };
  module.exports.tags = ['Token'];