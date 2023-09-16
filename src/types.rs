use revm::primitives::{Address, Bytes, SpecId, TransactTo, U256};

pub struct EvmConfig {
    pub spec_id: SpecId,
}

#[derive(Debug)]
pub struct TxOpts {
    pub from: Address,
    pub to: TransactTo,
    pub value: U256,
    pub tx_data: Bytes,
    pub gas_limit: u64,
    pub gas_price: U256,
}
