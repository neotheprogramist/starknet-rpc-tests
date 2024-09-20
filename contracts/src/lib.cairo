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
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
        balances: LegacyMap<felt252, felt252>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        DepositFromL1: DepositFromL1,
    }

    #[derive(Drop, starknet::Event)]
    struct DepositFromL1 {
        #[key]
        user: felt252,
        #[key]
        amount: felt252,
    }

    #[l1_handler]
    fn deposit(ref self: ContractState, from_address: felt252, user: felt252, amount: felt252) {
        let balance = self.balances.read(user);
        self.balances.write(user, balance + amount);
        self.emit(DepositFromL1 { user, amount });
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
