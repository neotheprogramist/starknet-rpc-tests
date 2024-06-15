#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> BalanceResult;
}

#[derive(Debug, PartialEq, Serde, Drop)]
pub enum BalanceResult {
    Zero,
    Positive,
    Negative,
    Overdrawn
}

#[starknet::contract]
mod SampleStarknet {
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
