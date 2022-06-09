module.exports = async ({getNamedAccounts, deployments}) => {
    const {deploy} = deployments;
    const {deployer} = await getNamedAccounts();
    const Forwarder = await ethers.getContractFactory("BiconomyForwarder");
    forwarder = await Forwarder.deploy(await accounts[0].getAddress());
    await forwarder.deployed();

    const domainData = {
        name: "World Token",
        version: "1",
        verifyingContract: forwarder.address,
        salt: ethers.utils.hexZeroPad(salt.toHexString(), 32)
    };
    await forwarder.registerDomainSeparator("World Token", "1");
    const domainSeparator = ethers.utils.keccak256(
      ethers.utils.defaultAbiCoder.encode(
        ["bytes32", "bytes32", "bytes32", "address", "bytes32"],
        [
          ethers.utils.id(
            "EIP712Domain(string name,string version,address verifyingContract,bytes32 salt)"
          ),
          ethers.utils.id(domainData.name),
          ethers.utils.id(domainData.version),
          domainData.verifyingContract, // FIXME
          domainData.salt,
        ]
      )
    );

});
module.exports.tags = ['TrustedForwarder'];