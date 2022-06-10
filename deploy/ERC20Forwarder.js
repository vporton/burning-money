const fs = require('fs');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];

    const ERC20Forwarder = await ethers.getContractFactory("ERC20Forwarder");
    erc20Forwarder = await ERC20Forwarder.deploy(deployer);
    await erc20Forwarder.deployed();
    
    const ERC20ForwarderProxy = await hre.ethers.getContractFactory("ERC20ForwarderProxy");
    erc20ForwarderProxy = await ERC20ForwarderProxy.deploy(
        erc20Forwarder.address,
        deployer, // admin
        deployer, // owner
    );
    await erc20ForwarderProxy.deployed();
  
    const forwarder = await deployments.getArtifact("BiconomyForwarder");
    const feeManager = await deployments.getArtifact("CentralisedFeeManager");
    proxy = await ethers.getContractAt(
        "ERC20Forwarder",
        erc20ForwarderProxy.address
    );
    await proxy.initialize(
        deployer,
        feeManager.address,
        forwarder.address
    );

    Collateral = await ethers.getContractAt(
        "contracts/6/token/erc20/IERC20.sol:IERC20",
        addresses.collateral
      );
    await proxy.setTransferHandlerGas(addresses.collateral, 41672); // USDT // FIXME: wrapped USDT
    await Collateral.approve(erc20ForwarderProxy.address, ethers.utils.parseEther("1000"));
  };
  module.exports.tags = ['ERC20Forwarder'];
  module.exports.dependencies = ['TrustedForwarder', 'FeeManager'];