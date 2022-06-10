const fs = require('fs');

module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[networkName];

    const ERC20Forwarder = await ethers.getContractFactory("ERC20Forwarder");
    erc20Forwarder = await ERC20Forwarder.deploy(
        await accounts[0].getAddress()
    );
    await erc20Forwarder.deployed();
    
    const ERC20ForwarderProxy = await hre.ethers.getContractFactory("ERC20ForwarderProxy");
    erc20ForwarderProxy = await ERC20ForwarderProxy.deploy(
        erc20Forwarder.address,
        await accounts[0].getAddress(), // admin
        await accounts[0].getAddress(), // owner
    );
    await erc20ForwarderProxy.deployed();
  
    const forwarder = await deployments.get("BiconomyForwarder");
    proxy = await ethers.getContractAt(
        "contracts/6/forwarder/ERC20Forwarder.sol:ERC20Forwarder",
        erc20ForwarderProxy.address
    );
    await proxy.initialize(
        await accounts[0].getAddress(),
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
  module.exports.dependencies = ['TrustedForwarder'];