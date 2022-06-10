const fs = require('fs');

function updateAddress(json, network, property, value) {
    if (network == 'ganache' || network == 'localhost' || network == 'local' || network === 'hardhat') {
        network = 'local';
    }
    if (!json[network]) {
        json[network] = {};
    }
    json[network][property] = value;
}

function getAddress(network, property) {
    if (network == 'ganache' || network == 'localhost' || network == 'local' || network === 'hardhat') {
        network = 'local';
    }
    let json;
    try {
        const text = fs.readFileSync(`deployed-addresses.json`);
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

    { // block
        const addressesFileName = `deployed-addresses.json`;
        let json;
        try {
            const text = fs.readFileSync(addressesFileName);
            json = JSON.parse(text);
        }
        catch(_) {
            json = {};  
        }
        updateAddress(json, network.name, contractName, deployResult.address);
        fs.writeFileSync(addressesFileName, JSON.stringify(json, null, 4));
    }

    return deployResult;
}

module.exports = {
    updateAddress,
    getAddress,
    myDeploy,
}
