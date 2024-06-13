use generator_error::GeneratorError;
use utils::{build_contract_with_scarb, modify_contract_enum, write_contract_to_file};

pub mod generator_error;
pub mod utils;

fn main() -> Result<(), GeneratorError> {
    let sample_contract = r#"#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> BalanceResult;
}

#[derive(Debug, PartialEq, Serde, Drop)]
pub enum BalanceResult {
    Positive,
    Zero,
    Negative
}

#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }
    use super::BalanceResult;
    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        fn get_balance(self: @ContractState) -> BalanceResult {
            let balance: u32 = self.balance.read().try_into().unwrap();

            if balance > 0 {
                BalanceResult::Positive
            } else if balance < 0 {
                BalanceResult::Negative
            } else {
                BalanceResult::Zero
            }
        }
    }
}
"#;

    let new_enum = r#"
    Positive,
    Zero,
    Negative,
    Overdrawn
"#;

    let modified_contract = modify_contract_enum(sample_contract, new_enum);
    let contract_path = "./example/src/lib.cairo";
    let package = "example";
    write_contract_to_file(&modified_contract, contract_path)?;
    build_contract_with_scarb(package)?;

    Ok(())
}
