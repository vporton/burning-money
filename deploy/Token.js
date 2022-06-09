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


    const ERC20ForwarderProxy = await hre.ethers.getContractFactory("ERC20ForwarderProxy");
    erc20ForwarderProxy = await ERC20ForwarderProxy.deploy(
        erc20Forwarder.address,
        await accounts[2].getAddress(), // FIXME
        await accounts[0].getAddress() // FIXME
    );
    await erc20ForwarderProxy.deployed();
  
    proxy = await ethers.getContractAt(
        "contracts/6/forwarder/ERC20Forwarder.sol:ERC20Forwarder",
        erc20ForwarderProxy.address
    );
    await proxy.initialize(
        await accounts[0].getAddress(),
        mockFeeManager.address,
        forwarder.address
    );

    await proxy.setTransferHandlerGas(USDT.address, 41672);

    await USDT.approve(erc20ForwarderProxy.address, ethers.utils.parseEther("1000"));
  };
  module.exports.tags = ['Token'];