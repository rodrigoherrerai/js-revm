const Revm = require("js-revm");

const { ethers } = require("ethers");

// Can select desired evm version.
function main() {
  const config = {
    specId: "London",
  };

  const evm = new Revm(config);
}

main();
