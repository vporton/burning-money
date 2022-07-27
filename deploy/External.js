const fs = require('fs');
const { ethers } = require('hardhat');
const { myDeploy, updateAddress } = require('../lib/default-deployer');

module.exports = async ({getNamedAccounts, deployments, network}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const [deployerSigner] = await ethers.getSigners();
    const networkName = hre.network.name;
    const addresses = JSON.parse(fs.readFileSync('addresses.json'))[network.name];
    if(addresses.collateralOracle) {
        updateAddress(network.name, "collateralOracle", addresses.collateralOracle);
    } else if(process.env['TEST']) {
        const TestPriceOracle = await ethers.getContractFactory("TestPriceOracle");
        const testPriceOracle = await myDeploy(
            TestPriceOracle, network, deployer, "collateralOracle", [],
        );

        const serverAccount = new ethers.Wallet('0x' + fs.readFileSync("server/ethereum.priv"));
        deployerSigner.sendTransaction({
            to: await serverAccount.getAddress(),
            value: ethers.utils.parseEther("0.5"),
        });
    }
};
module.exports.tags = ['External'];
