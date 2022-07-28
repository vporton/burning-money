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
        console.log(`Price oracle ${testPriceOracle.addresses}`)

        const serverAccount = new ethers.Wallet('0x' + fs.readFileSync("server/ethereum.priv"));
        const serverAddress = await serverAccount.getAddress();
        console.log("Server address: ", serverAddress);
        await deployerSigner.sendTransaction({
            to: serverAddress,
            value: ethers.utils.parseEther("0.5"),
        });
        console.log(`Server account funding ${await ethers.provider.getBalance(serverAddress)}`)
    }
};
module.exports.tags = ['External'];
