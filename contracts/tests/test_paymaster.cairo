use starknet::{contract_address_const, ContractAddress, account::Call, contract_address_to_felt252};
use snforge_std::{
    declare, cheat_caller_address, start_cheat_caller_address_global,
    stop_cheat_caller_address_global, cheat_signature, spy_events, ContractClassTrait, CheatSpan,
    DeclareResultTrait, EventSpyAssertionsTrait, EventSpyTrait, Event,
};
use snforge_std::signature::{KeyPairTrait, KeyPair};
use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl};
use contracts::paymaster::{
    account_executable::{
        IExecuteFromOutsideCallbackDispatcher, IExecuteFromOutsideCallbackDispatcherTrait
    },
    erc20::{ITestTokenDispatcher, ITestTokenDispatcherTrait}
};
use contracts::paymaster::interface::{OutsideExecution, IOffChainMessageHashRev1};
use openzeppelin::account::interface::{AccountABIDispatcherTrait, AccountABIDispatcher};

pub fn DEPLOYER() -> ContractAddress {
    contract_address_const::<'DEPLOYER'>()
}

pub fn SPENDER() -> ContractAddress {
    let felt: felt252 = 0x1b175fe86400121641d32d47490f76cd1ff973a6f090631496c0a08a530ed18;
    felt.try_into().expect('Invalid address')
}

pub fn GENERATED_ACCOUNT() -> ContractAddress {
    contract_address_const::<'GENERATED'>()
}

pub fn deploy_contract(name: ByteArray, constructor_calldata: Array<felt252>) -> ContractAddress {
    let contract = declare(name).unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();
    contract_address
}

#[derive(Drop, Copy)]
pub struct SetupResult {
    pub token: ITestTokenDispatcher,
    pub erc_20_address: ContractAddress,
    pub paymaster_account: AccountABIDispatcher,
    pub paymaster_keys: KeyPair<felt252, felt252>,
    pub executable_account: IExecuteFromOutsideCallbackDispatcher,
    pub account_to_address: ContractAddress
}

pub fn setup() -> SetupResult {
    let paymaster_keys = KeyPairTrait::<felt252, felt252>::generate();
    println!("pub_key: {}", paymaster_keys.public_key);
    println!("priv_key: {}", paymaster_keys.secret_key);

    let external_keys = KeyPairTrait::<felt252, felt252>::generate();
    println!("pub_key: {}", external_keys.public_key);
    println!("priv_key: {}", external_keys.secret_key);

    let account_to_keys = KeyPairTrait::<felt252, felt252>::generate();
    println!("pub_key: {}", account_to_keys.public_key);
    println!("priv_key: {}", account_to_keys.secret_key);

    let account_paymaster = deploy_contract("OZAccount", array![paymaster_keys.public_key]);
    let account_to_address = deploy_contract("OZAccount", array![external_keys.public_key]);
    let executable_account_address = deploy_contract("MyAccount", array![external_keys.public_key]);
    let erc_20_address = deploy_contract("TestToken", array![]);

    let token = ITestTokenDispatcher { contract_address: erc_20_address };
    let paymaster_account = AccountABIDispatcher { contract_address: account_paymaster };
    let executable_account = IExecuteFromOutsideCallbackDispatcher {
        contract_address: executable_account_address
    };

    cheat_caller_address(token.contract_address, DEPLOYER(), CheatSpan::TargetCalls(1));
    token.mint(executable_account_address, 1_000_000);

    SetupResult {
        token,
        erc_20_address,
        paymaster_account,
        paymaster_keys,
        executable_account,
        account_to_address
    }
}


#[test]
fn test1() {
    let SetupResult { token,
    erc_20_address,
    paymaster_account,
    paymaster_keys,
    executable_account,
    account_to_address, } =
        setup();

    let call = Call {
        to: erc_20_address,
        selector: selector!("transfer"),
        calldata: array![account_to_address.into(), 1_000, 0].span(),
    };

    let outside_execution = OutsideExecution {
        caller: paymaster_account.contract_address, nonce: 1, calls: array![call].span(),
    };

    let hash = outside_execution.get_message_hash_rev_1();
    let (r, s): (felt252, felt252) = paymaster_keys.sign(hash).unwrap();

    let mut outside_execution_calldata: Array<felt252> = array![];

    outside_execution.serialize(ref outside_execution_calldata);
    let signature: Array<felt252> = array![r, s];
    signature.serialize(ref outside_execution_calldata);

    let call_to_executable_account = Call {
        to: executable_account.contract_address,
        selector: selector!("execute_from_outside"),
        calldata: outside_execution_calldata.span(),
    };

    cheat_caller_address(
        paymaster_account.contract_address, contract_address_const::<0>(), CheatSpan::TargetCalls(1)
    );

    paymaster_account.__execute__(array![call_to_executable_account]);

    let balance = token.balance_of(account_to_address);
    assert!(balance == 1_000, "ERC20: wrong balance");
}
