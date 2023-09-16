"use-strict";

const assert = require("assert");

const { ethers } = require("ethers");

const Revm = require("..");

function getRandom() {
  return ethers.Wallet.createRandom().address;
}

describe("REVM", () => {
  it("get_balance, set_balance, should get and set new balance", () => {
    const revm = new Revm();
    const addr = getRandom();

    let bal = revm.getBalance(addr);
    assert.equal(bal, 0);

    const amount = 1000e18;

    // set bal.
    revm.setBalance(addr, amount);

    bal = revm.getBalance(addr);
    assert.equal(bal, amount);
  });

  it("call_commit: should transfer eth", () => {
    const revm = new Revm();
    const from = getRandom();
    const to = getRandom();

    // both should be 0.
    assert.equal(revm.getBalance(from), 0);
    assert.equal(revm.getBalance(to), 0);

    // set balance from.
    const amount = 1e18;
    revm.setBalance(from, amount);
    assert.equal(revm.getBalance(from), amount);

    const txOpts = {
      from,
      to,
      value: amount,
      txData: "",
      gasLimit: 21000,
      gasPrice: 0,
    };
    const txResult = revm.callCommit(txOpts);
    assert.equal(txResult.success, true);

    // balance should be updated.
    assert.equal(revm.getBalance(to), amount);
    assert.equal(revm.getBalance(from), 0);
  });

  it("call_commit: should deploy a contract, update the state, and fetch state", () => {
    const solc = require("solc");

    const input = {
      language: "Solidity",
      sources: {
        "test.sol": {
          content: `contract T {
            uint public num;
            string public str;

            function setNum(uint n) external {num = n;}
            function setStr(string memory s) external {str = s;}
          }
          `,
        },
      },
      settings: {
        outputSelection: {
          "*": {
            "*": ["*"],
          },
        },
      },
    };

    const output = JSON.parse(solc.compile(JSON.stringify(input)));
    const bytecode = output.contracts["test.sol"]["T"].evm.bytecode.object;

    const revm = new Revm();
    const from = getRandom();
    let txOpts = {
      from,
      to: "", // empty field to deploy contract.
      value: 0,
      txData: bytecode,
      gasLimit: 1000000,
      gasPrice: 0,
    };
    let txResult = revm.callCommit(txOpts);
    assert.equal(txResult.success, true);
    const contractAddress = txResult.contract_created;
    const abiCoder = new ethers.AbiCoder();

    // bytes4(keccak256("num()"))
    let data = "0x4e70b1dc";

    // modify ops.
    txOpts.to = contractAddress;
    txOpts.txData = data;
    txResult = revm.callCommit(txOpts);
    let result = txResult.call_output;
    num = abiCoder.decode(["uint256"], "0x" + result);
    assert.equal(num, 0);

    // let's update num.
    const newNum = 1111;
    // bytes4(keccak256("setNum(uint256)"))
    data = "0xcd16ecbf";
    data += abiCoder.encode(["uint256"], [newNum]).slice(2);

    txOpts.txData = data;

    txResult = revm.callCommit(txOpts);
    assert.equal(txResult.success, true);

    // should be updated.
    txOpts.txData = "0x4e70b1dc";
    txResult = revm.callCommit(txOpts);
    result = txResult.call_output;
    num = abiCoder.decode(["uint256"], "0x" + result);
    assert.equal(num, newNum);

    // update the str.
    const newStr = "Rust is fast!";
    // bytes4(keccak256("setStr(string)"))
    data = "0x191347df";
    data += abiCoder.encode(["string"], [newStr]).slice(2);
    txOpts.txData = data;
    txResult = revm.callCommit(txOpts);
    assert.equal(txResult.success, true);

    // string should be updated.
    // bytes4(keccak256("str()"))
    txOpts.txData = "0xc15bae84";
    revm.callCommit(txOpts);
    txResult = revm.callCommit(txOpts);
    result = txResult.call_output;
    str = abiCoder.decode(["string"], "0x" + result);
    assert.equal(str, "Rust is fast!");
  });

  it("transact_ref: state shouldn't persist", () => {
    const revm = new Revm();
    const from = getRandom();
    assert.equal(revm.getBalance(from), 0);

    // set balance.
    revm.setBalance(from, 100e18);
    assert.equal(revm.getBalance(from), 100e18);

    const to = getRandom();
    assert.equal(revm.getBalance(to), 0);

    // tx.
    const txOpts = {
      from,
      to,
      gasPrice: 0,
      value: 5,
      gasLimit: 21000,
      txData: "",
    };
    const txResult = revm.callNoCommit(txOpts);
    assert.equal(txResult.success, true);

    // state shouldn't change
    assert.equal(revm.getBalance(to), 0);
  });
});
