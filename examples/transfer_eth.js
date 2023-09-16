const Revm = require("js-revm");

const { ethers } = require("ethers");

function main() {
  const evm = new Revm();

  const from = ethers.Wallet.createRandom().address;
  console.assert(evm.getBalance(from) === 0, "balance from should be 0");

  // set balance.
  evm.setBalance(from, 1e18);
  console.assert(evm.getBalance(from) === 1e18, "balance wasn't set");

  const to = ethers.Wallet.createRandom().address;
  console.assert(evm.getBalance(to) === 0, "balance to should be 0");

  const amount = 1111;

  // build the tx.
  const txOpts = {
    from,
    to,
    value: amount,
    txData: "0x",
    gasLimit: 21000,
    gasPrice: 1,
  };
  const txResult = evm.callCommit(txOpts);
  console.log(txResult);
  if (!txResult.success) {
    throw new Error("tx failed");
  }

  console.assert(evm.getBalance(to) === amount, "balance to should be updated");
}

main();
