mod sample_contract_1;
mod sample_contract_2;
mod sample_contract_3;
mod sample_contract_4;
mod sample_contract_5;

/// Paymaster implementation.
mod paymaster {
    /// Implementation of an account that an execute txns from outside the contract
    mod account_executable;

    /// OpenZapplein account implementation
    mod account_oz;

    /// ERC20 token implementation
    mod erc20;

    //// Common interfaces and message hashing utilities.
    mod interface;
}

