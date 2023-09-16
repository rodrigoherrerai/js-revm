use revm::primitives::{Halt, SpecId};

pub fn to_spec_id(spec_id: &str) -> SpecId {
    match spec_id {
        "FRONTIER" => SpecId::FRONTIER,
        "FRONTIER_THAWING" => SpecId::FRONTIER_THAWING,
        "HOMESTAD" => SpecId::HOMESTEAD,
        "DAO_FORK" => SpecId::DAO_FORK,
        "TANGERINE" => SpecId::TANGERINE,
        "SPURIOUS_DRAGON" => SpecId::SPURIOUS_DRAGON,
        "BYZANTIUM" => SpecId::BYZANTIUM,
        "CONSTANTINOPLE" => SpecId::CONSTANTINOPLE,
        "PETERSBURG" => SpecId::PETERSBURG,
        "ISTANBUL" => SpecId::ISTANBUL,
        "MUIR_GLACIER" => SpecId::MUIR_GLACIER,
        "BERLIN" => SpecId::BERLIN,
        "LONDON" => SpecId::LONDON,
        "ARROW_GLACIER" => SpecId::ARROW_GLACIER,
        "GRAY_GLACIER" => SpecId::GRAY_GLACIER,
        "MERGE" => SpecId::MERGE,
        "SHANGHAI" => SpecId::SHANGHAI,
        _ => SpecId::LATEST,
    }
}

pub fn halt_reason_as_string(reason: Halt) -> String {
    match reason {
        Halt::OutOfGas(_) => "Out of Gas".to_string(),
        Halt::OpcodeNotFound => "Opcode Not Found".to_string(),
        Halt::InvalidFEOpcode => "Invalid FE Opcode".to_string(),
        Halt::InvalidJump => "Invalid Jump".to_string(),
        Halt::NotActivated => "Not Activated".to_string(),
        Halt::StackUnderflow => "Stack Underflow".to_string(),
        Halt::StackOverflow => "Stack Overflow".to_string(),
        Halt::OutOfOffset => "Out of Offset".to_string(),
        Halt::CreateCollision => "Create Collision".to_string(),
        Halt::PrecompileError => "Precompile Error".to_string(),
        Halt::NonceOverflow => "Nonce Overflow".to_string(),
        Halt::CreateContractSizeLimit => "Create Contract Size Limit".to_string(),
        Halt::CreateContractStartingWithEF => "Create Contract Starting With EF".to_string(),
        Halt::CreateInitcodeSizeLimit => "Create Initcode Size Limit".to_string(),
        Halt::OverflowPayment => "Overflow Payment".to_string(),
        Halt::StateChangeDuringStaticCall => "State Change During Static Call".to_string(),
        Halt::CallNotAllowedInsideStatic => "Call Not Allowed Inside Static".to_string(),
        Halt::OutOfFund => "Out of Fund".to_string(),
        Halt::CallTooDeep => "Call Too Deep".to_string(),
    }
}
