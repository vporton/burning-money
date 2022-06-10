const fs = require('fs');
const { myDeploy, getAddress } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];

    const ERC20Forwarder = await ethers.getContractFactory("ERC20Forwarder");
    const erc20Forwarder = await myDeploy(ERC20Forwarder, network, deployer, "ERC20Forwarder", [deployer]);
    
    const ERC20ForwarderProxy = await hre.ethers.getContractFactory("ERC20ForwarderProxy");
    const erc20ForwarderProxy = await myDeploy(
        ERC20ForwarderProxy, network, deployer, "ERC20ForwarderProxy",
        [
            erc20Forwarder.address,
            deployer, // admin
            deployer, // owner
        ],
    );
  
    const forwarder = getAddress(network.name, "BiconomyForwarder");
    const feeManager = getAddress(network.name, "CentralisedFeeManager");
    proxy = await ethers.getContractAt(
        "ERC20Forwarder",
        erc20ForwarderProxy.address
    );
    await proxy.initialize(
        deployer,
        feeManager,
        forwarder,
    );

    const token = getAddress(network.name, "Token");
    await proxy.setTransferHandlerGas(token, 41672); // FIXME
    await token.approve(erc20ForwarderProxy.address, ethers.utils.parseEther("1000"));
  };
  module.exports.tags = ['ERC20Forwarder'];
  module.exports.dependencies = ['TrustedForwarder', 'FeeManager', 'Token'];