use eyre::Result;
use neon::prelude::*;
use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{
        AccountInfo, Address, Bytes, CreateScheme, Env, ExecutionResult, Output, ResultAndState,
        TransactTo, U256,
    },
    InMemoryDB, EVM,
};
use std::cell::RefCell;

use types::{EvmConfig, TxOpts};
mod types;

mod utils;
use utils::{halt_reason_as_string, to_spec_id};

type DB = CacheDB<EmptyDB>;
type BoxedDB = JsBox<RefCell<Revm>>;

struct Revm {
    db: DB,
}
impl Finalize for Revm {}

impl Revm {
    pub fn new() -> Self {
        let db = InMemoryDB::default();
        Revm { db }
    }

    pub fn get_balance(&self, address: Address) -> U256 {
        let mut db = self.db.clone();
        let account_info = db.load_account(address).unwrap();
        account_info.info.balance
    }

    fn get_env(&self, tx_opts: TxOpts, evm_config: EvmConfig) -> Env {
        let mut env = Env::default();
        // CfgEnv
        // for now we only support evm version.
        env.cfg.spec_id = evm_config.spec_id;

        // BlockEnv
        // ..

        // TxEnv
        // missing gas_priority_fee, chain_id, nonce, access_list
        env.tx.caller = tx_opts.from;
        env.tx.gas_limit = tx_opts.gas_limit;
        env.tx.gas_price = tx_opts.gas_price;
        env.tx.transact_to = tx_opts.to;
        env.tx.value = tx_opts.value;
        env.tx.data = tx_opts.tx_data;

        env
    }

    // change this to set account info (so insert code is also valid).
    pub fn set_balance(&mut self, address: Address, balance: U256) -> Result<()> {
        let info = AccountInfo {
            balance,
            ..Default::default()
        };

        self.db.insert_account_info(address, info);

        Ok(())
    }

    /// Executes the transaction, commits it to the DB and returns the result.
    pub fn call_commit(
        &mut self,
        tx_opts: TxOpts,
        evm_config: EvmConfig,
    ) -> Result<ExecutionResult> {
        // Build the EVM.
        let mut evm = EVM::new();

        // Get the environment ready for the tx execution.
        let env = self.get_env(tx_opts, evm_config);

        // Insert the db.
        evm.database(&mut self.db);

        // Insert the env.
        evm.env = env;

        // tx execution.
        let result: ExecutionResult = evm.transact_commit().unwrap();

        Ok(result)
    }

    /// Executes the transaction without committing to the DB, returns result.
    pub fn call_no_commit(
        &mut self,
        tx_opts: TxOpts,
        evm_config: EvmConfig,
    ) -> Result<ResultAndState> {
        let mut evm = EVM::new();

        let env = self.get_env(tx_opts, evm_config);

        evm.database(self.db.clone());

        evm.env = env;

        let result: ResultAndState = evm.transact_ref().unwrap();

        Ok(result)
    }
}

fn new(mut cx: FunctionContext) -> JsResult<BoxedDB> {
    let revm = Revm::new();

    Ok(cx.boxed(RefCell::new(revm)))
}

fn get_balance(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let revm = cx.argument::<BoxedDB>(0)?;
    let revm = revm.borrow_mut();

    let address = cx.argument::<JsString>(1)?.value(&mut cx);
    let address: Address = address.parse().expect("Invalid address");

    let balance = revm.get_balance(address);

    Ok(cx.number(balance))
}

fn set_balance(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let revm = cx.argument::<BoxedDB>(0)?;
    let mut revm = revm.borrow_mut();

    let address = cx.argument::<JsString>(1)?.value(&mut cx);
    let address: Address = address.parse().expect("Invalid address");

    let balance = cx.argument::<JsNumber>(2)?.value(&mut cx);

    revm.set_balance(address, U256::from(balance)).unwrap();
    Ok(cx.boolean(true))
}

fn call_commit(mut cx: FunctionContext) -> JsResult<JsObject> {
    let revm = cx.argument::<BoxedDB>(0)?;
    let mut revm = revm.borrow_mut();

    let from = cx.argument::<JsString>(1)?.value(&mut cx);
    let from: Address = from.parse().expect("Invalid address");

    let to = cx.argument::<JsString>(2)?.value(&mut cx);
    let to = if to.is_empty() {
        // If it is empty, it is contract creation.
        TransactTo::Create(CreateScheme::Create)
    } else {
        let to_addr: Address = to.parse().unwrap();
        TransactTo::Call(to_addr)
    };
    let value = cx.argument::<JsNumber>(3)?.value(&mut cx);

    let tx_data = cx.argument::<JsString>(4)?.value(&mut cx);
    let tx_data: &str = &tx_data;
    let tx_data = if tx_data.starts_with("0x") {
        tx_data.trim_start_matches("0x")
    } else {
        tx_data
    };

    let gas_limit = cx.argument::<JsNumber>(5)?.value(&mut cx);

    let gas_price = cx.argument::<JsNumber>(6)?.value(&mut cx);

    let spec_id = cx.argument::<JsString>(7)?.value(&mut cx);
    let spec_id: &str = &spec_id;
    let spec_id = to_spec_id(spec_id);

    let tx_opts: TxOpts = TxOpts {
        from,
        to,
        value: U256::from(value),
        tx_data: Bytes::from(hex::decode(tx_data).unwrap()),
        gas_limit: gas_limit as u64,
        gas_price: U256::from(gas_price),
    };

    let evm_config: EvmConfig = EvmConfig { spec_id };

    let result = revm.call_commit(tx_opts, evm_config).unwrap();

    // returned object.
    let obj: Handle<JsObject> = cx.empty_object();

    match result {
        // missing: reason, gas_refuned and logs.
        ExecutionResult::Success {
            gas_used, output, ..
        } => {
            // First let's set the tx object as success.
            let cx_true = cx.boolean(true);
            obj.set(&mut cx, "success", cx_true)?;

            let cx_gas_used = cx.number(gas_used as f64);
            // gas used.
            obj.set(&mut cx, "gas_used", cx_gas_used)?;

            match output {
                Output::Create(_, Some(contract)) => {
                    // The tx was a contract creation.
                    let contract = format!("{:?}", contract);
                    let cx_contract = cx.string(contract);
                    obj.set(&mut cx, "contract_created", cx_contract)?;
                }
                Output::Call(output) => {
                    let cx_output = cx.string(hex::encode(output));
                    obj.set(&mut cx, "call_output", cx_output)?;
                }
                _ => {} // todo
            }
        }
        ExecutionResult::Revert { gas_used, output } => {
            let cx_false = cx.boolean(false);
            let cx_gas_used = cx.number(gas_used as f64);
            let cx_output = cx.string(hex::encode(output));
            obj.set(&mut cx, "success", cx_false)?;
            obj.set(&mut cx, "gas_used", cx_gas_used)?;
            obj.set(&mut cx, "revert_output", cx_output)?;
        }
        ExecutionResult::Halt { reason, gas_used } => {
            let cx_false = cx.boolean(false);
            let cx_gas_used = cx.number(gas_used as f64);
            obj.set(&mut cx, "success", cx_false)?;
            obj.set(&mut cx, "gas_used", cx_gas_used)?;
            let cx_reason = cx.string(halt_reason_as_string(reason));
            obj.set(&mut cx, "reason", cx_reason)?;
        }
    }

    Ok(obj)
}

fn call_no_commit(mut cx: FunctionContext) -> JsResult<JsObject> {
    let revm = cx.argument::<BoxedDB>(0)?;
    let mut revm = revm.borrow_mut();

    let from = cx.argument::<JsString>(1)?.value(&mut cx);
    let from: Address = from.parse().expect("Invalid address");

    let to = cx.argument::<JsString>(2)?.value(&mut cx);
    let to = if to.is_empty() {
        // If it is empty, it is contract creation.
        TransactTo::Create(CreateScheme::Create)
    } else {
        let to_addr: Address = to.parse().unwrap();
        TransactTo::Call(to_addr)
    };
    let value = cx.argument::<JsNumber>(3)?.value(&mut cx);

    let tx_data = cx.argument::<JsString>(4)?.value(&mut cx);
    let tx_data: &str = &tx_data;
    let tx_data = if tx_data.starts_with("0x") {
        tx_data.trim_start_matches("0x")
    } else {
        tx_data
    };

    let gas_limit = cx.argument::<JsNumber>(5)?.value(&mut cx);

    let gas_price = cx.argument::<JsNumber>(6)?.value(&mut cx);

    let spec_id = cx.argument::<JsString>(7)?.value(&mut cx);
    let spec_id: &str = &spec_id;
    let spec_id = to_spec_id(spec_id);

    let tx_opts: TxOpts = TxOpts {
        from,
        to,
        value: U256::from(value),
        tx_data: Bytes::from(hex::decode(tx_data).unwrap()),
        gas_limit: gas_limit as u64,
        gas_price: U256::from(gas_price),
    };

    let evm_config: EvmConfig = EvmConfig { spec_id };

    let result = revm.call_no_commit(tx_opts, evm_config).unwrap();

    let obj: Handle<JsObject> = cx.empty_object();

    match result.state {
        // todo
        _ => {}
    }

    match result.result {
        // missing: reason, gas_refuned and logs.
        ExecutionResult::Success {
            gas_used, output, ..
        } => {
            // First let's set the tx object as success.
            let cx_true = cx.boolean(true);
            obj.set(&mut cx, "success", cx_true)?;

            let cx_gas_used = cx.number(gas_used as f64);
            // gas used.
            obj.set(&mut cx, "gas_used", cx_gas_used)?;

            match output {
                Output::Create(_, Some(contract)) => {
                    // The tx was a contract creation.
                    let contract = format!("{:?}", contract);
                    let cx_contract = cx.string(contract);
                    obj.set(&mut cx, "contract_created", cx_contract)?;
                }
                Output::Call(output) => {
                    let cx_output = cx.string(hex::encode(output));
                    obj.set(&mut cx, "call_output", cx_output)?;
                }
                _ => {} // todo
            }
        }
        ExecutionResult::Revert { gas_used, output } => {
            let cx_false = cx.boolean(false);
            let cx_gas_used = cx.number(gas_used as f64);
            let cx_output = cx.string(hex::encode(output));
            obj.set(&mut cx, "success", cx_false)?;
            obj.set(&mut cx, "gas_used", cx_gas_used)?;
            obj.set(&mut cx, "revert_output", cx_output)?;
        }
        ExecutionResult::Halt { reason, gas_used } => {
            let cx_false = cx.boolean(false);
            let cx_gas_used = cx.number(gas_used as f64);
            obj.set(&mut cx, "success", cx_false)?;
            obj.set(&mut cx, "gas_used", cx_gas_used)?;
            let cx_reason = cx.string(halt_reason_as_string(reason));
            obj.set(&mut cx, "reason", cx_reason)?;
        }
    }

    Ok(obj)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("new", new)?;
    cx.export_function("get_balance", get_balance)?;
    cx.export_function("call_commit", call_commit)?;
    cx.export_function("call_no_commit", call_no_commit)?;
    cx.export_function("set_balance", set_balance)?;
    Ok(())
}
