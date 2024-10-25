// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Contracts for Cairo ^0.18.0
use contracts::paymaster::interface::OutsideExecution;
#[starknet::interface]
pub trait IExecuteFromOutsideCallback<TContractState> {
    fn execute_from_outside_callback(
        self: @TContractState,
        outside_execution: OutsideExecution,
        outside_tx_hash: felt252,
        signature: Array<felt252>,
    ) -> Span<felt252>;
}

// #[starknet::interface]
// pub trait InternalTrait<TContractState> {
//     fn _is_valid_sign(
//         ref self: @ComponentState<TContractState>, hash: felt252, signature: Span<felt252>
//     ) -> bool;
//     fn _is_valid_stark_sign(
//         ref self: TContractState, msg_hash: felt252, public_key: felt252, signature:
//         Span<felt252>
//     ) -> bool;
// }

#[starknet::contract(account)]
mod MyAccount {
    use contracts::paymaster::interface::{OutsideExecution};
    use core::ecdsa::check_ecdsa_signature;
    use starknet::{get_tx_info, get_caller_address, get_contract_address};
    use super::{IExecuteFromOutsideCallback};
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    use openzeppelin::account::AccountComponent;
    use openzeppelin::introspection::src5::SRC5Component;
    use openzeppelin::upgrades::UpgradeableComponent;
    use openzeppelin::upgrades::interface::IUpgradeable;
    use starknet::{ClassHash, account::Call, ContractAddress};

    component!(path: AccountComponent, storage: account, event: AccountEvent);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);
    component!(path: UpgradeableComponent, storage: upgradeable, event: UpgradeableEvent);

    #[abi(embed_v0)]
    impl AccountMixinImpl = AccountComponent::AccountMixinImpl<ContractState>;

    impl AccountInternalImpl = AccountComponent::InternalImpl<ContractState>;
    impl UpgradeableInternalImpl = UpgradeableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        account: AccountComponent::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
        #[substorage(v0)]
        upgradeable: UpgradeableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        AccountEvent: AccountComponent::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
        #[flat]
        UpgradeableEvent: UpgradeableComponent::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, public_key: felt252) {
        self.account.initializer(public_key);
    }

    #[abi(embed_v0)]
    impl UpgradeableImpl of IUpgradeable<ContractState> {
        fn upgrade(ref self: ContractState, new_class_hash: ClassHash) {
            self.account.assert_only_self();
            self.upgradeable.upgrade(new_class_hash);
        }
    }

    #[abi(embed_v0)]
    impl ExecuteFromOutsideCallback of IExecuteFromOutsideCallback<ContractState> {
        fn execute_from_outside_callback(
            self: @ContractState,
            outside_execution: OutsideExecution,
            outside_tx_hash: felt252,
            signature: Array<felt252>,
        ) -> Span<felt252> {
            let x = self._is_valid_sign(outside_tx_hash, signature.span());

            let y: felt252 = 5;
            array![y].span()
        }
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        fn _is_valid_sign(self: @ContractState, hash: felt252, signature: Span<felt252>) -> bool {
            let public_key = self.account.Account_public_key.read();
            self._is_valid_stark_sign(hash, public_key, signature)
        }

        fn _is_valid_stark_sign(
            self: @ContractState, msg_hash: felt252, public_key: felt252, signature: Span<felt252>
        ) -> bool {
            let valid_length = signature.len() == 2;

            if valid_length {
                check_ecdsa_signature(
                    msg_hash, public_key, *signature.at(0_u32), *signature.at(1_u32)
                )
            } else {
                false
            }
        }

        fn _is_valid_calls(
            self: @ContractState,
            calls: Span<Call>,
            outside_execution_hash: felt252,
            signature: Span<felt252>,
            is_from_outside: bool,
            account_address: ContractAddress,
        ) -> Span<felt252> {}
    }
}
