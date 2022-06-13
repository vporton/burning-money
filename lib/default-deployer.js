const fs = require('fs');

function updateAddress(networkName, property, value) {
    const addressesFileName = `dist/deployed-addresses.json`;
    let json;
    try {
        const text = fs.readFileSync(addressesFileName);
        json = JSON.parse(text);
    }
    catch(_) {
        json = {};  
    }

    if (networkName == 'ganache' || networkName == 'localhost' || networkName == 'local' || networkName === 'hardhat') {
        networkName = 'local';
    }
    if (!json[networkName]) {
        json[networkName] = {};
    }

    try {
        fs.mkdirSync('dist');
    }
    catch(e) { }
    json[networkName][property] = value;
    fs.writeFileSync(addressesFileName, JSON.stringify(json, null, 4));
}

function getAddress(network, property) {
    if (network == 'ganache' || network == 'localhost' || network == 'local' || network === 'hardhat') {
        network = 'local';
    }
    let json;
    try {
        const text = fs.readFileSync(`dist/deployed-addresses.json`);
        json = JSON.parse(text);
    }
    catch(_) {
        json = {};  
    }
    if (!json[network]) {
        return null;
    }
    return json[network][property];
}

async function myDeploy(factory, network, deployer, contractName, args) {
    const contract = await factory.deploy(...args);
    const deployResult = await contract.deployed();
    updateAddress(network.name, contractName, deployResult.address);
    return deployResult;
}

module.exports = {
    updateAddress,
    getAddress,
    myDeploy,
}
