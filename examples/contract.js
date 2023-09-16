const Revm = require("js-revm");

const { ethers } = require("ethers");

// This example deploys a new contract and interacts with it.
function main() {
  const evm = new Revm();

  const from = ethers.Wallet.createRandom().address;
  evm.setBalance(from, 1e18);

  // we get the contract bytecode.
  const bytecode = getBytecode();
  let txOpts = {
    from,
    to: "", // empty field to deploy contract.
    value: 0,
    txData: bytecode,
    gasLimit: 30000000,
    gasPrice: 0,
  };
  let txResult = evm.callCommit(txOpts);
  console.log("gas used ->", txResult.gas_used);
  console.log("success ->", txResult.success);

  // the address of the new contract created.
  const contractAddr = txResult.contract_created;

  const abiCoder = new ethers.AbiCoder();
  // modify the number.
  const newNum = 1111;
  // bytes4(keccak256("setNum(uint256)"))
  let data = "0xcd16ecbf";
  data += abiCoder.encode(["uint256"], [newNum]).slice(2);

  // exec the tx.
  txOpts.to = contractAddr;
  txOpts.txData = data;

  // all the other tx fields can remain the same.
  txResult = evm.callCommit(txOpts);
  console.log("tx result ->", txResult);

  // let's get the number, should be updated.
  // should be updated.
  txOpts.txData = "0x4e70b1dc";
  txResult = evm.callCommit(txOpts);
  console.log("tx result ->", txResult);
  const result = txResult.call_output;
  num = abiCoder.decode(["uint256"], "0x" + result);

  // should be the new number.
  console.log("updated num ->", num);
}
main();

function getBytecode() {
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
  return bytecode;
}
