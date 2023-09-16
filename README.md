# js-revm

**Super fast JS bindings to [revm](https://github.com/bluealloy/revm/)**

## What
**Use the EVM implemented in Rust in your JS projects by simply installing an npm module**

- Extremely simple to use
- Extremely fast

## Tooling
**This project uses [neon](https://github.com/neon-bindings/neon) as a core dependency**

## Usage
**Install the npm pacakge (version should be >= 0.1.2)** 

````
npm i js-revm
````
**Example Usage**

*You can find more examples under the examples folder.

```js
const Revm = require("js-revm");

function main() {
  const evm = new Revm(); // defaults to latest
  const addr = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

  // get new balance.
  const bal = evm.getBalance(addr);
  console.log("bal ->", bal);

  // set balance.
  evm.setBalance(addr, 1e18);
  console.log("new bal ->", evm.getBalance(addr));

  // tx.
  const from = addr;
  const to = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
  console.log("balance to before ->", evm.getBalance(to));
  const gasLimit = 21000;
  const gasPrice = 0;
  const value = 1111;
  const txData = "";

  const txOpts = {
    from,
    to,
    value,
    gasLimit,
    gasPrice,
    txData,
  };

  // execute the tx and commit it.
  const result = evm.callCommit(txOpts);
  console.log(result);

  console.log("balance post tx ->", evm.getBalance(to));
}

main();

``````

### Tx Simulation
TODO not supported yed

## API
**For now, JS-REVM comes with 4 public functions**

### 1. getBalance(addr)
Returns the balance of the address

### 2. setBalance(address, amount)
Sets a new balance to the address, amount in WEI 

### 3. callCommit(txOpts)
Executes a transaction and commits it to the memory db created by default. 

#### txOpts:

- from: The address executing the transaction (defaults to address(0))
- to: The destination address (empty for contract creation)
- value: Amount in WEI (defaults to 0)
- txData: Transaction data or bytecode for contract creation
- gasLimit: Gas limit for the executing transaction (defaults to 30_000_000)
- gasPrice: Gas price for the transaction (defaults to 0)

#### Result
The tx returns an object with some key properties depending on the transaction (contract_created for new contract creation, error reason if reverted, etc.)

### 4. callNoCommit(txOpts)
Same as callCommit but it executes the transaction without committing it to the db 

### Config
The EVM can be configured to any desired version, it defaults to the latest. 

**Example**
```js
function main() {
  const config = {
    specId: "London",
  };

  const evm = new Revm(config);
}
``````

## Benchmark 