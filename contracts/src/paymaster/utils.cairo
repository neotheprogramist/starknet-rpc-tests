use starknet::{get_contract_address, get_caller_address, ContractAddress, account::Call};

fn assert_no_self_call(mut calls: Span<Call>, self: ContractAddress) {
    while let Option::Some(call) = calls.pop_front() {
        assert(*call.to != self, 'oz/no-multicall-to-self')
    }
}
