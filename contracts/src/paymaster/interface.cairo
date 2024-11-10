use starknet::{ContractAddress, account::Call, get_tx_info, get_contract_address};
use core::hash::{HashStateTrait, HashStateExTrait};
use core::poseidon::{hades_permutation, HashState, poseidon_hash_span};

const MAINNET_FIRST_HADES_PERMUTATION: (felt252, felt252, felt252) =
    (
        2727651893633223888261849279042022325174182119102281398572272198960815727249,
        729016093840936084580216898033636860729342953928695140840860652272753125883,
        2792630223211151632174198306610141883878913626231408099903852589995722964080
    );

const SEPOLIA_FIRST_HADES_PERMUTATION: (felt252, felt252, felt252) =
    (
        3580606761507954093996364807837346681513890124685758374532511352257317798951,
        3431227198346789440159663709265467470274870120429209591243179659934705045436,
        974062396530052497724701732977002885691473732259823426261944148730229556466
    );

const OUTSIDE_CALL_TYPE_HASH_REV_0: felt252 =
    selector!("OutsideCall(to:felt,selector:felt,calldata_len:felt,calldata:felt*)");

const OUTSIDE_EXECUTION_TYPE_HASH_REV_0: felt252 =
    selector!(
        "OutsideExecution(caller:felt,nonce:felt,execute_after:felt,execute_before:felt,calls_len:felt,calls:OutsideCall*)OutsideCall(to:felt,selector:felt,calldata_len:felt,calldata:felt*)"
    );

const OUTSIDE_EXECUTION_TYPE_HASH_REV_1: felt252 =
    selector!(
        "\"OutsideExecution\"(\"Caller\":\"ContractAddress\",\"Nonce\":\"felt\",\"Execute After\":\"u128\",\"Execute Before\":\"u128\",\"Calls\":\"Call*\")\"Call\"(\"To\":\"ContractAddress\",\"Selector\":\"selector\",\"Calldata\":\"felt*\")"
    );

const CALL_TYPE_HASH_REV_1: felt252 =
    selector!(
        "\"Call\"(\"To\":\"ContractAddress\",\"Selector\":\"selector\",\"Calldata\":\"felt*\")"
    );

#[derive(Copy, Drop, Serde, Debug)]
struct OutsideExecution {
    caller: ContractAddress,
    nonce: felt252,
    calls: Span<Call>
}


/// @notice StarkNetDomain using SNIP 12 Revision 1
#[derive(Hash, Drop, Copy)]
struct StarknetDomain {
    name: felt252,
    version: felt252,
    chain_id: felt252,
    revision: felt252,
}

const STARKNET_DOMAIN_TYPE_HASH_REV_1: felt252 =
    selector!(
        "\"StarknetDomain\"(\"name\":\"shortstring\",\"version\":\"shortstring\",\"chainId\":\"shortstring\",\"revision\":\"shortstring\")"
    );

impl StructHashStarknetDomain of IStructHashRev1<StarknetDomain> {
    fn get_struct_hash_rev_1(self: @StarknetDomain) -> felt252 {
        poseidon_hash_span(
            array![
                STARKNET_DOMAIN_TYPE_HASH_REV_1,
                *self.name,
                *self.version,
                *self.chain_id,
                *self.revision
            ]
                .span()
        )
    }
}

/// @notice Defines the function to generate the SNIP-12 revision 1 compliant message hash
trait IOffChainMessageHashRev1<T> {
    fn get_message_hash_rev_1(self: @T) -> felt252;
}

impl StructHashCallRev1 of IStructHashRev1<Call> {
    fn get_struct_hash_rev_1(self: @Call) -> felt252 {
        poseidon_hash_span(
            array![
                CALL_TYPE_HASH_REV_1,
                (*self.to).into(),
                *self.selector,
                poseidon_hash_span(*self.calldata)
            ]
                .span()
        )
    }
}

impl StructHashOutsideExecutionRev1 of IStructHashRev1<OutsideExecution> {
    fn get_struct_hash_rev_1(self: @OutsideExecution) -> felt252 {
        let self = *self;
        let mut calls_span = self.calls;
        let mut hashed_calls = array![];

        while let Option::Some(call) = calls_span.pop_front() {
            hashed_calls.append(call.get_struct_hash_rev_1());
        };
        poseidon_hash_span(
            array![
                OUTSIDE_EXECUTION_TYPE_HASH_REV_1,
                self.caller.into(),
                self.nonce,
                poseidon_hash_span(hashed_calls.span()),
            ]
                .span()
        )
    }
}


impl OffChainMessageOutsideExecutionRev1 of IOffChainMessageHashRev1<OutsideExecution> {
    fn get_message_hash_rev_1(self: @OutsideExecution) -> felt252 {
        // Version is a felt instead of a shortstring in SNIP-9 due to a mistake in the Braavos
        // contracts and has been copied for compatibility.
        // Revision will also be a felt instead of a shortstring for all SNIP12-rev1 signatures
        // because of the same issue

        let chain_id = get_tx_info().unbox().chain_id;
        if chain_id == 'SN_MAIN' {
            return get_message_hash_rev_1_with_precalc(MAINNET_FIRST_HADES_PERMUTATION, *self);
        }
        if chain_id == 'SN_SEPOLIA' {
            return get_message_hash_rev_1_with_precalc(SEPOLIA_FIRST_HADES_PERMUTATION, *self);
        }
        let domain = StarknetDomain {
            name: 'Account.execute_from_outside', version: 2, chain_id, revision: 1
        };
        poseidon_hash_span(
            array![
                'StarkNet Message',
                domain.get_struct_hash_rev_1(),
                get_contract_address().into(),
                (*self).get_struct_hash_rev_1(),
            ]
                .span()
        )
    }
}


fn get_message_hash_rev_1_with_precalc<T, +Drop<T>, +IStructHashRev1<T>>(
    hades_permutation_state: (felt252, felt252, felt252), rev1_struct: T
) -> felt252 {
    // mainnet_domain_hash = domain.get_struct_hash_rev_1()
    // hades_permutation_state == hades_permutation('StarkNet Message', mainnet_domain_hash, 0);
    let (s0, s1, s2) = hades_permutation_state;

    let (fs0, fs1, fs2) = core::poseidon::hades_permutation(
        s0 + get_contract_address().into(), s1 + rev1_struct.get_struct_hash_rev_1(), s2
    );
    core::poseidon::HashState { s0: fs0, s1: fs1, s2: fs2, odd: false }.finalize()
}

/// @notice Defines the function to generates the SNIP-12 revision 1 compliant hash on an object
trait IStructHashRev1<T> {
    fn get_struct_hash_rev_1(self: @T) -> felt252;
}
