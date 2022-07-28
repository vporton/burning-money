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
    admin: {
      default: 1,
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
      ],
    },
    local: {
      url: "http://localhost:8545",
      accounts: [
        "0x74b9eca92a6d3e5274c3cec8e5d869d2df8f5a75caab0df949a1a60c064bdd41", // 0x72F7Fe922C0A829c92E90794abA85940319D453E
        "0x3569575d34e46721b7932ced96053a26ae2e29ef6de61a6131a9bb7038761d54", // 0xbBF7056D91F05858c4534B325af042750BE70B84
      ],
    },
  },
  solidity: {
    compilers: [
      { version: "0.8.14" },
      { version: "0.7.6" },
    ],
  },
};
