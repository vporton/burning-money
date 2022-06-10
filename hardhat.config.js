require("@nomiclabs/hardhat-waffle");
require('hardhat-deploy');

// This is a sample Hardhat task. To learn how to create your own go to
// https://hardhat.org/guides/create-task.html
task("accounts", "Prints the list of accounts", async (taskArgs, hre) => {
  const accounts = await hre.ethers.getSigners();

  for (const account of accounts) {
    console.log(account.address);
  }
});

// You need to export an object to set up your config
// Go to https://hardhat.org/config/ to learn more

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
module.exports = {
  namedAccounts: {
    deployer: {
      default: 0, // here this will by default take the first account as deployer
    },
  },
  networks: {
    hardhat: {
      accounts: [
        {
          privateKey: "2366a48160bcc5f0cef8bbace95928130d3aabe972475cea2c1b978ebcad4212", // 0xe9243658aFAD5CEAd2e6ca3C0E44087EcA1D11A3
          balance: String(10**18),
        },
        {
          privateKey: "34bc09a2210e5a1e2cf34cee3e1e7cc73cbe6ae3cdf2dec15b15dd1a814c9540", // 0xFbe0204Ffa36E3C621331d36FB566352e1EB1F7e
          balance: String(10**18),
        },
      ]
    },
  },
  solidity: {
    compilers: [
      { version: "0.8.14" },
      { version: "0.7.6" },
    ],
  },
};
