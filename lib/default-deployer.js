const fs = require('fs');

function updateAddress(json, network, property, value) {
    if (network == 'ganache' || network === 'development') {
        network = 'local';
    }
    if (!json[network]) {
        json[network] = {};
    }
    json[network][property] = value;
}

function getAddress(network, property) {
    if (network == 'ganache' || network == 'localhost' || network === 'hardhat') {
        network = 'local';
    }
    let json;
    try {
        const text = fs.readFileSync(addressesFileName);
        json = JSON.parse(text);
    }
    catch(_) {
        json = {};  
    }
    if (!json[network]) {
        return {};
    }
    return json[network][property];
}

async function myDeploy(factory, network, accounts, contractName, ...args) {
    await factory.deploy(args, {overwrite: false, from: accounts[0]}); // `from` for Truffle 5.1.58 bug
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
        updateAddress(json, network.name, contractName, {
            address: deployResult.address,
        });
        fs.writeFileSync(addressesFileName, JSON.stringify(json, null, 4));
    }

    return deployResult;
}

module.exports = {
    updateAddress,
    myDeploy,
}
