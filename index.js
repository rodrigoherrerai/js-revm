"use-strict";

const revm = require("./index.node");

const chalk = require("chalk");

const ZERO_ADDR = "0x0000000000000000000000000000000000000000";
const SPEC_ID = [
  "FRONTIER",
  "FRONTIER_THAWING",
  "HOMESTAD",
  "DAO_FORK",
  "TANGERINE",
  "SPURIOUS_DRAGON",
  "BYZANTIUM",
  "CONSTANTINOPLE",
  "PETERSBURG",
  "INSTANBUL",
  "MUIR_GLACIER",
  "BERLIN",
  "LONDON",
  "ARROW_GLACIER",
  "GRAY_GLACIER",
  "MERGE",
  "SHANGHAI",
];

function sanitizeTxOpts(txOpts) {
  if (txOpts.from == undefined) {
    console.warn(
      chalk.yellow("WARNING !!!: ") +
        chalk.magenta("The from field was not set. Default will be address(0)")
    );
    txOpts.from = ZERO_ADDR;
  }

  if (txOpts.to == undefined || txOpts.to == "0x") {
    // contract creation.
    txOpts.to = "";
  }

  if (txOpts.txData == undefined) {
    console.warn(
      chalk.yellow("WARNING !!!: ") +
        chalk.magenta(
          "The data field was not set. Default will be empty string"
        )
    );
    txOpts.txData = "";
  }

  if (txOpts.gasLimit == undefined) {
    console.warn(
      chalk.yellow("WARNING !!!: ") +
        chalk.magenta(
          "The gasLimit field was not set. Default will be 30_000_000"
        )
    );

    txOpts.gasLimit = 30_000_000;
  }
  if (txOpts.gasPrice == undefined) {
    console.warn(
      chalk.yellow("WARNING !!!: ") +
        chalk.magenta("The gasPrice field was not set. Default will be 0")
    );
    txOpts.gasPrice = 0;
  }
}

/**
 * REVM.
 *
 * JS Wrapper around Rust EVM implementation (REVM).
 */
class Revm {
  constructor(config = {}) {
    if (config.specId == undefined) {
      // default config.
      config.specId = "SHANGHAI";
    }

    config.specId = config.specId.toUpperCase();
    if (!SPEC_ID.includes(config.specId)) {
      throw new Error(
        `Invalid EVM version. Valid versions are: ${SPEC_ID.join(", ")}`
      );
    }
    this.config = config;
    this.db = revm.new();
    this.revm = revm;
  }

  // Updates the state. Sets 'amount = WEI'to 'address'.
  setBalance(address, amount) {
    this.revm.set_balance(this.db, address, amount);
  }

  // Returns the balance of 'address'.
  getBalance(address) {
    const balance = this.revm.get_balance(this.db, address);
    return balance;
  }

  // Executes the transaction, commits it to the DB and returns the result.
  callCommit(txOpts) {
    sanitizeTxOpts(txOpts);
    return this.revm.call_commit(
      this.db,
      txOpts.from,
      txOpts.to,
      txOpts.value,
      txOpts.txData,
      txOpts.gasLimit,
      txOpts.gasPrice,
      this.config.specId
    );
  }

  // Executes the transaction without committing to the DB, returns result.
  callNoCommit(txOpts) {
    sanitizeTxOpts(txOpts);
    return this.revm.call_no_commit(
      this.db,
      txOpts.from,
      txOpts.to,
      txOpts.value,
      txOpts.txData,
      txOpts.gasLimit,
      txOpts.gasPrice,
      this.config.specId
    );
  }
}

module.exports = Revm;
