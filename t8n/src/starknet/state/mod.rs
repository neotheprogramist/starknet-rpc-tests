pub mod account;
pub mod add_declare_transaction;
pub mod add_deploy_account_transaction;
pub mod add_invoke_transaction;
pub mod add_l1_handler_transaction;
pub mod constants;
pub mod contract_class_choice;
pub mod defaulter;
pub mod dict_state;
pub mod dump;
pub mod errors;
pub mod estimations;
pub mod events;
pub mod get_class_impls;
pub mod predeployed;
pub mod predeployed_accounts;
pub mod raw_execution;
pub mod starknet_blocks;
pub mod starknet_config;
pub mod starknet_state;
pub mod starknet_transactions;
pub mod state_diff;
pub mod state_update;
pub mod system_contract;
pub mod traits;
pub mod transaction_trace;
pub mod types;
pub mod utils;
use std::{num::NonZeroU128, sync::Arc};

use super::messaging::MessagingBroker;
use account::{Account, UserAccount};
use blockifier::{
    block::BlockInfo,
    context::{BlockContext, ChainInfo, TransactionContext},
    execution::entry_point::CallEntryPoint,
    state::{
        cached_state::{CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST},
        state_api::StateReader,
    },
    transaction::{
        errors::TransactionPreValidationError, objects::TransactionExecutionInfo,
        transactions::ExecutableTransaction,
    },
};
use constants::{
    CHARGEABLE_ACCOUNT_ADDRESS, CHARGEABLE_ACCOUNT_PRIVATE_KEY, DEVNET_DEFAULT_CHAIN_ID,
    DEVNET_DEFAULT_DATA_GAS_PRICE, DEVNET_DEFAULT_GAS_PRICE, DEVNET_DEFAULT_STARTING_BLOCK_NUMBER,
    ETH_ERC20_CONTRACT_ADDRESS, ETH_ERC20_NAME, ETH_ERC20_SYMBOL, STRK_ERC20_CONTRACT_ADDRESS,
    STRK_ERC20_NAME, STRK_ERC20_SYMBOL,
};
use contract_class_choice::AccountContractClassChoice;
use defaulter::StarknetDefaulter;
use dump::DumpEvent;
use errors::{DevnetResult, Error, TransactionValidationError};
use predeployed::initialize_erc20_at_address;
use predeployed_accounts::UserDeployedAccounts;
use raw_execution::{Call, RawExecution};
use serde::Serialize;
use starknet_api::{
    block::{BlockNumber, BlockStatus, BlockTimestamp, GasPrice, GasPricePerToken},
    core::SequencerContractAddress,
    transaction::Fee,
};
use starknet_blocks::{StarknetBlock, StarknetBlocks};
use starknet_config::{StarknetConfig, StateArchiveCapacity};
use starknet_devnet_types::{
    chain_id::ChainId,
    contract_address::ContractAddress,
    contract_class::ContractClass,
    emitted_event::EmittedEvent,
    felt::{split_biguint, ClassHash, Felt, TransactionHash},
    num_bigint::BigUint,
    patricia_key::PatriciaKey,
    rpc::{
        block::{Block, BlockHeader},
        estimate_message_fee::FeeEstimateWrapper,
        state::ThinStateDiff,
        transaction_receipt::{
            DeployTransactionReceipt, L1HandlerTransactionReceipt, TransactionReceipt,
        },
        transactions::{
            broadcasted_invoke_transaction_v1::BroadcastedInvokeTransactionV1,
            l1_handler_transaction::L1HandlerTransaction, BlockTransactionTrace,
            BroadcastedDeclareTransaction, BroadcastedDeployAccountTransaction,
            BroadcastedInvokeTransaction, BroadcastedTransaction, BroadcastedTransactionCommon,
            DeclareTransaction, SimulatedTransaction, SimulationFlag, Transaction,
            TransactionTrace, TransactionWithHash, TransactionWithReceipt, Transactions,
        },
    },
    traits::HashProducer,
};
use starknet_rs_core::{
    types::{
        BlockId, ExecutionResult, FieldElement, MsgFromL1, TransactionExecutionStatus,
        TransactionFinalityStatus,
    },
    utils::get_selector_from_name,
};
use starknet_rs_signers::Signer;
use starknet_state::{CustomState, StarknetState, StateWithBlockNumber};
use starknet_transactions::{StarknetTransaction, StarknetTransactions};
use state_diff::StateDiff;
use state_update::StateUpdate;
use std::path::Path;

use tracing::{error, info};
use traits::{Deployed, HashIdentified, HashIdentifiedMut, UserAccountGenerator};
use transaction_trace::create_trace;
use utils::get_versioned_constants;

#[derive(Debug, Serialize)]
pub struct Starknet {
    pub state: StarknetState,
    #[serde(skip_serializing)]
    pub predeployed_accounts: UserDeployedAccounts,
    #[serde(skip_serializing)]
    pub block_context: BlockContext,
    // To avoid repeating some logic related to blocks,
    // having `blocks` public allows to re-use functions like `get_blocks()`.
    pub blocks: StarknetBlocks,
    // #[serde(skip_serializing)]
    pub transactions: StarknetTransactions,
    #[serde(skip_serializing)]
    pub config: StarknetConfig,
    #[serde(skip_serializing)]
    pub pending_block_timestamp_shift: i64,
    #[serde(skip_serializing)]
    pub next_block_timestamp: Option<u64>,
    #[serde(skip_serializing)]
    pub messaging: MessagingBroker,
    #[serde(skip_serializing)]
    pub dump_events: Vec<DumpEvent>,
    pub transaction_receipts: Vec<TransactionReceipt>,
}

impl Default for Starknet {
    fn default() -> Self {
        Self {
            block_context: Self::init_block_context(
                DEVNET_DEFAULT_GAS_PRICE,
                DEVNET_DEFAULT_DATA_GAS_PRICE,
                ETH_ERC20_CONTRACT_ADDRESS,
                STRK_ERC20_CONTRACT_ADDRESS,
                DEVNET_DEFAULT_CHAIN_ID,
                DEVNET_DEFAULT_STARTING_BLOCK_NUMBER,
            ),
            state: Default::default(),
            predeployed_accounts: Default::default(),
            blocks: Default::default(),
            transactions: Default::default(),
            config: Default::default(),
            pending_block_timestamp_shift: 0,
            next_block_timestamp: None,
            messaging: Default::default(),
            dump_events: Default::default(),
            transaction_receipts: Default::default(),
        }
    }
}

impl Starknet {
    pub fn new(config: &StarknetConfig, acc_path: &Path) -> DevnetResult<Self> {
        let defaulter = StarknetDefaulter::new(config.fork_config.clone());
        let mut state = StarknetState::new(defaulter);

        // predeclare account classes
        for account_class_choice in [
            AccountContractClassChoice::Cairo0,
            AccountContractClassChoice::Cairo1,
        ] {
            let class_wrapper = account_class_choice.get_class_wrapper()?;
            state.predeclare_contract_class(
                class_wrapper.class_hash,
                class_wrapper.contract_class,
            )?;
        }

        // deploy udc, eth erc20 and strk erc20 contracts
        let eth_erc20_fee_contract =
            predeployed::create_erc20_at_address(ETH_ERC20_CONTRACT_ADDRESS)?;
        let strk_erc20_fee_contract =
            predeployed::create_erc20_at_address(STRK_ERC20_CONTRACT_ADDRESS)?;

        let udc_contract = predeployed::create_udc()?;
        udc_contract.deploy(&mut state)?;

        eth_erc20_fee_contract.deploy(&mut state)?;
        initialize_erc20_at_address(
            &mut state,
            ETH_ERC20_CONTRACT_ADDRESS,
            ETH_ERC20_NAME,
            ETH_ERC20_SYMBOL,
        )?;

        strk_erc20_fee_contract.deploy(&mut state)?;
        initialize_erc20_at_address(
            &mut state,
            STRK_ERC20_CONTRACT_ADDRESS,
            STRK_ERC20_NAME,
            STRK_ERC20_SYMBOL,
        )?;

        let mut predeployed_accounts = UserDeployedAccounts::new(
            eth_erc20_fee_contract.get_address(),
            strk_erc20_fee_contract.get_address(),
        );

        let accounts = predeployed_accounts
            .generate_accounts(
                acc_path,
                config.account_contract_class_hash,
                &config.account_contract_class,
            )
            .unwrap();

        for account in accounts {
            account.deploy(&mut state)?;
        }

        let chargeable_account = Account::new_chargeable(
            eth_erc20_fee_contract.get_address(),
            strk_erc20_fee_contract.get_address(),
        )?;
        chargeable_account.deploy(&mut state)?;

        state.commit_with_diff()?;

        // when forking, the number of the first new block to be mined is equal to the last origin
        // block (the one specified by the user) plus one.
        let starting_block_number = config
            .fork_config
            .block_number
            .map_or(DEVNET_DEFAULT_STARTING_BLOCK_NUMBER, |n| n + 1);
        let mut this = Self {
            state,
            predeployed_accounts,
            block_context: Self::init_block_context(
                config.gas_price,
                config.data_gas_price,
                ETH_ERC20_CONTRACT_ADDRESS,
                STRK_ERC20_CONTRACT_ADDRESS,
                config.chain_id,
                starting_block_number,
            ),
            blocks: StarknetBlocks::new(starting_block_number),
            transactions: StarknetTransactions::default(),
            config: config.clone(),
            pending_block_timestamp_shift: 0,
            next_block_timestamp: None,
            messaging: Default::default(),
            dump_events: Default::default(),
            transaction_receipts: Default::default(),
        };

        this.restart_pending_block()?;

        // Create an empty genesis block, set start_time before if it's set
        if let Some(start_time) = config.start_time {
            this.set_next_block_timestamp(start_time);
        };
        this.create_block()?;

        // Load starknet transactions
        if this.config.dump_path.is_some() && this.config.re_execute_on_init {
            // Try to load transactions from dump_path, if there is no file skip this step
            match this.load_events() {
                Ok(events) => this.re_execute(events)?,
                Err(Error::FileNotFound) => {}
                Err(err) => return Err(err),
            };
        }

        Ok(this)
    }

    pub fn from_init_state(state: StateWithBlockNumber) -> DevnetResult<Self> {
        let config = StarknetConfig::default();

        let mut this = Self {
            state: state.state,
            predeployed_accounts: UserDeployedAccounts::default(),
            block_context: Self::init_block_context(
                config.gas_price,
                config.data_gas_price,
                ETH_ERC20_CONTRACT_ADDRESS,
                STRK_ERC20_CONTRACT_ADDRESS,
                config.chain_id,
                state.block_number.0 + 1,
            ),
            blocks: StarknetBlocks::new(state.block_number.0 + 1),
            transactions: StarknetTransactions::default(),
            config: config.clone(),
            pending_block_timestamp_shift: 0,
            next_block_timestamp: None,
            messaging: Default::default(),
            dump_events: Default::default(),
            transaction_receipts: Default::default(),
        };

        this.restart_pending_block()?;

        // Create an empty genesis block, set start_time before if it's set
        if let Some(start_time) = config.start_time {
            this.set_next_block_timestamp(start_time);
        };

        // Load starknet transactions
        if this.config.dump_path.is_some() && this.config.re_execute_on_init {
            // Try to load transactions from dump_path, if there is no file skip this step
            match this.load_events() {
                Ok(events) => this.re_execute(events)?,
                Err(Error::FileNotFound) => {}
                Err(err) => return Err(err),
            };
        }

        Ok(this)
    }

    pub fn restart(&mut self, acc_path: &Path) -> DevnetResult<()> {
        self.config.re_execute_on_init = false;
        *self = Starknet::new(&self.config, acc_path)?;
        info!("Starknet Devnet restarted");

        Ok(())
    }

    pub fn get_predeployed_accounts(&self) -> Vec<UserAccount> {
        self.predeployed_accounts.get_accounts().to_vec()
    }

    // Update block context
    // Initialize values for new pending block
    pub(crate) fn generate_pending_block(&mut self) -> DevnetResult<()> {
        Self::advance_block_context_block_number(&mut self.block_context);
        self.restart_pending_block()?;

        Ok(())
    }

    fn next_block_timestamp(&mut self) -> BlockTimestamp {
        match self.next_block_timestamp {
            Some(timestamp) => {
                self.next_block_timestamp = None;
                BlockTimestamp(timestamp)
            }
            None => BlockTimestamp(
                (Starknet::get_unix_timestamp_as_seconds() as i64
                    + self.pending_block_timestamp_shift) as u64,
            ),
        }
    }

    /// Transfer data from pending block into new block and save it to blocks collection
    /// Generates new pending block
    /// Returns the new block number
    pub(crate) fn generate_new_block(&mut self, state_diff: StateDiff) -> DevnetResult<Felt> {
        let mut new_block = self.pending_block().clone();

        // set new block header
        new_block.set_block_hash(new_block.generate_hash()?);
        new_block.status = BlockStatus::AcceptedOnL2;

        // set block timestamp and context block timestamp for contract execution
        let block_timestamp = self.next_block_timestamp();
        new_block.set_timestamp(block_timestamp);
        Self::update_block_context_block_timestamp(&mut self.block_context, block_timestamp);

        let new_block_number =
            BlockNumber(new_block.block_number().0 - self.blocks.aborted_blocks.len() as u64);
        new_block.header.block_number = new_block_number;
        let new_block_hash: Felt = new_block.header.block_hash.0.into();

        // update txs block hash block number for each transaction in the pending block
        new_block.get_transactions().iter().for_each(|tx_hash| {
            if let Some(tx) = self.transactions.get_by_hash_mut(tx_hash) {
                tx.block_hash = Some(new_block_hash);
                tx.block_number = Some(new_block_number);
                tx.finality_status = TransactionFinalityStatus::AcceptedOnL2;
            } else {
                error!("Transaction is not present in the transactions collection");
            }
        });

        // insert pending block in the blocks collection and connect it to the state diff
        self.blocks.insert(new_block, state_diff);

        // save into blocks state archive
        if self.config.state_archive == StateArchiveCapacity::Full {
            let clone = self.state.clone_historic();
            self.blocks.save_state_at(new_block_hash, clone);
        }

        self.generate_pending_block()?;

        Ok(new_block_hash)
    }

    /// Handles transaction result either Ok or Error and updates the state accordingly.
    ///
    /// # Arguments
    ///
    /// * `transaction` - Transaction to be added in the collection of transactions.
    /// * `contract_class` - Contract class to be added in the state cache. Only in declare
    ///   transactions.
    /// * `transaction_result` - Result with transaction_execution_info
    pub(crate) fn handle_transaction_result(
        &mut self,
        transaction: TransactionWithHash,
        contract_class: Option<ContractClass>,
        transaction_result: Result<
            TransactionExecutionInfo,
            blockifier::transaction::errors::TransactionExecutionError,
        >,
    ) -> DevnetResult<()> {
        let transaction_hash = *transaction.get_transaction_hash();

        fn declare_contract_class(
            class_hash: &ClassHash,
            contract_class: Option<ContractClass>,
            state: &mut StarknetState,
        ) -> DevnetResult<()> {
            state.declare_contract_class(
                *class_hash,
                contract_class.ok_or(Error::UnexpectedInternalError {
                    msg: "contract class not provided".to_string(),
                })?,
            )
        }

        match transaction_result {
            Ok(tx_info) => {
                // If transaction is not reverted
                // then save the contract class in the state cache for Declare transactions
                if !tx_info.is_reverted() {
                    match &transaction.transaction {
                        Transaction::Declare(DeclareTransaction::V1(declare_v1)) => {
                            declare_contract_class(
                                &declare_v1.class_hash,
                                contract_class,
                                &mut self.state,
                            )?
                        }
                        Transaction::Declare(DeclareTransaction::V2(declare_v2)) => {
                            declare_contract_class(
                                &declare_v2.class_hash,
                                contract_class,
                                &mut self.state,
                            )?
                        }
                        Transaction::Declare(DeclareTransaction::V3(declare_v3)) => {
                            declare_contract_class(
                                declare_v3.get_class_hash(),
                                contract_class,
                                &mut self.state,
                            )?
                        }
                        _ => {}
                    };
                }
                self.handle_accepted_transaction(&transaction_hash, &transaction, tx_info)
            }
            Err(tx_err) => {
                /// utility to avoid duplication
                fn match_tx_fee_error(
                    err: blockifier::transaction::errors::TransactionFeeError,
                ) -> DevnetResult<()> {
                    match err {
                        blockifier::transaction::errors::TransactionFeeError::FeeTransferError { .. }
                        | blockifier::transaction::errors::TransactionFeeError::MaxFeeTooLow { .. } => Err(
                            TransactionValidationError::InsufficientMaxFee.into()
                        ),
                        blockifier::transaction::errors::TransactionFeeError::MaxFeeExceedsBalance { .. } | blockifier::transaction::errors::TransactionFeeError::L1GasBoundsExceedBalance { .. } => Err(
                            TransactionValidationError::InsufficientAccountBalance.into()
                        ),
                        _ => Err(err.into())
                    }
                }

                // based on this https://community.starknet.io/t/efficient-utilization-of-sequencer-capacity-in-starknet-v0-12-1/95607#the-validation-phase-in-the-gateway-5
                // we should not save transactions that failed with one of the following errors
                match tx_err {
                    blockifier::transaction::errors::TransactionExecutionError::TransactionPreValidationError(
                        TransactionPreValidationError::InvalidNonce { .. }
                    ) => Err(TransactionValidationError::InvalidTransactionNonce.into()),
                    blockifier::transaction::errors::TransactionExecutionError::FeeCheckError { .. } =>
                        Err(TransactionValidationError::InsufficientMaxFee.into()),
                    blockifier::transaction::errors::TransactionExecutionError::TransactionPreValidationError(
                        TransactionPreValidationError::TransactionFeeError(err)
                    ) => match_tx_fee_error(err),
                    blockifier::transaction::errors::TransactionExecutionError::TransactionFeeError(err)
                      => match_tx_fee_error(err),
                    blockifier::transaction::errors::TransactionExecutionError::ValidateTransactionError(err) => {
                        Err(TransactionValidationError::ValidationFailure { reason: err.to_string() }.into())
                    }
                    _ => Err(tx_err.into())
                }
            }
        }
    }

    /// Handles suceeded and reverted transactions.
    /// The tx is stored and potentially dumped.
    /// A new block is generated.
    pub(crate) fn handle_accepted_transaction(
        &mut self,
        transaction_hash: &TransactionHash,
        transaction: &TransactionWithHash,
        tx_info: TransactionExecutionInfo,
    ) -> DevnetResult<()> {
        let state_diff = self.state.diff_trace()?;

        let trace = create_trace(
            &mut self.state.state,
            transaction.get_type(),
            &tx_info,
            state_diff.clone().into(),
        )?;
        let transaction_to_add = StarknetTransaction::create_accepted(transaction, tx_info, trace);

        // add accepted transaction to pending block
        self.blocks.pending_block.add_transaction(*transaction_hash);

        self.transactions
            .insert(transaction_hash, transaction_to_add);

        Ok(())
    }

    fn init_block_context(
        gas_price: NonZeroU128,
        data_gas_price: NonZeroU128,
        eth_fee_token_address: &str,
        strk_fee_token_address: &str,
        chain_id: ChainId,
        block_number: u64,
    ) -> BlockContext {
        use starknet_api::core::{ContractAddress, PatriciaKey};
        use starknet_api::hash::StarkHash;
        use starknet_api::{contract_address, patricia_key};

        // Create a BlockContext based on BlockContext::create_for_testing()

        let block_info = BlockInfo {
            block_number: BlockNumber(block_number),
            block_timestamp: BlockTimestamp(0),
            sequencer_address: contract_address!("0x1000"),
            gas_prices: blockifier::block::GasPrices {
                eth_l1_gas_price: gas_price,
                strk_l1_gas_price: gas_price,
                eth_l1_data_gas_price: data_gas_price,
                strk_l1_data_gas_price: data_gas_price,
            },
            use_kzg_da: true,
        };

        let chain_info = ChainInfo {
            chain_id: chain_id.into(),
            fee_token_addresses: blockifier::context::FeeTokenAddresses {
                eth_fee_token_address: contract_address!(eth_fee_token_address),
                strk_fee_token_address: contract_address!(strk_fee_token_address),
            },
        };

        BlockContext::new_unchecked(&block_info, &chain_info, &get_versioned_constants())
    }

    /// Update block context block_number with the next one
    /// # Arguments
    /// * `block_context` - BlockContext to be updated
    fn advance_block_context_block_number(block_context: &mut BlockContext) {
        let mut block_info = block_context.block_info().clone();
        block_info.block_number = block_info.block_number.next();
        // TODO: update block_context via preferred method in the documentation
        *block_context = BlockContext::new_unchecked(
            &block_info,
            block_context.chain_info(),
            &get_versioned_constants(),
        );
    }

    fn update_block_context_block_timestamp(
        block_context: &mut BlockContext,
        block_timestamp: BlockTimestamp,
    ) {
        let mut block_info = block_context.block_info().clone();
        block_info.block_timestamp = block_timestamp;

        // TODO: update block_context via preferred method in the documentation
        *block_context = BlockContext::new_unchecked(
            &block_info,
            block_context.chain_info(),
            &get_versioned_constants(),
        );
    }

    pub fn pending_block(&self) -> &StarknetBlock {
        &self.blocks.pending_block
    }

    /// Restarts pending block with information from block_context
    fn restart_pending_block(&mut self) -> DevnetResult<()> {
        let mut block = StarknetBlock::create_pending_block();

        block.header.block_number = self.block_context.block_info().block_number;
        block.header.l1_gas_price = GasPricePerToken {
            price_in_fri: GasPrice(
                self.block_context
                    .block_info()
                    .gas_prices
                    .strk_l1_gas_price
                    .get(),
            ),
            price_in_wei: GasPrice(
                self.block_context
                    .block_info()
                    .gas_prices
                    .eth_l1_gas_price
                    .get(),
            ),
        };
        block.header.l1_data_gas_price = GasPricePerToken {
            price_in_fri: GasPrice(
                self.block_context
                    .block_info()
                    .gas_prices
                    .strk_l1_data_gas_price
                    .get(),
            ),
            price_in_wei: GasPrice(
                self.block_context
                    .block_info()
                    .gas_prices
                    .eth_l1_data_gas_price
                    .get(),
            ),
        };
        block.header.sequencer =
            SequencerContractAddress(self.block_context.block_info().sequencer_address);

        self.blocks.pending_block = block;

        Ok(())
    }

    fn get_mut_state_at(&mut self, block_id: &BlockId) -> DevnetResult<&mut StarknetState> {
        match block_id {
            BlockId::Tag(_) => Ok(&mut self.state),
            _ => {
                if self.config.state_archive == StateArchiveCapacity::None {
                    return Err(Error::NoStateAtBlock {
                        block_id: *block_id,
                    });
                }

                let block = self
                    .blocks
                    .get_by_block_id(block_id)
                    .ok_or(Error::NoBlock)?;
                let block_hash = block.block_hash();
                let state = self.blocks.hash_to_state.get_mut(&block_hash).ok_or(
                    Error::NoStateAtBlock {
                        block_id: *block_id,
                    },
                )?;
                Ok(state)
            }
        }
    }

    pub fn get_class_hash_at(
        &mut self,
        block_id: &BlockId,
        contract_address: ContractAddress,
    ) -> DevnetResult<ClassHash> {
        get_class_impls::get_class_hash_at_impl(self, block_id, contract_address)
    }

    pub fn get_class(
        &mut self,
        block_id: &BlockId,
        class_hash: ClassHash,
    ) -> DevnetResult<ContractClass> {
        get_class_impls::get_class_impl(self, block_id, class_hash)
    }

    pub fn get_class_at(
        &mut self,
        block_id: &BlockId,
        contract_address: ContractAddress,
    ) -> DevnetResult<ContractClass> {
        get_class_impls::get_class_at_impl(self, block_id, contract_address)
    }

    pub fn call(
        &mut self,
        block_id: &BlockId,
        contract_address: Felt,
        entrypoint_selector: Felt,
        calldata: Vec<Felt>,
    ) -> DevnetResult<Vec<Felt>> {
        let block_context = self.block_context.clone();
        let state = self.get_mut_state_at(block_id)?;

        state.assert_contract_deployed(ContractAddress::new(contract_address)?)?;

        let call = CallEntryPoint {
            calldata: starknet_api::transaction::Calldata(std::sync::Arc::new(
                calldata.iter().map(|f| f.into()).collect(),
            )),
            storage_address: starknet_api::hash::StarkFelt::from(contract_address).try_into()?,
            entry_point_selector: starknet_api::core::EntryPointSelector(
                entrypoint_selector.into(),
            ),
            initial_gas: block_context.versioned_constants().tx_initial_gas(),
            ..Default::default()
        };

        let mut execution_context =
            blockifier::execution::entry_point::EntryPointExecutionContext::new(
                Arc::new(TransactionContext {
                    block_context: block_context.clone(),
                    tx_info: blockifier::transaction::objects::TransactionInfo::Deprecated(
                        blockifier::transaction::objects::DeprecatedTransactionInfo::default(),
                    ),
                }),
                blockifier::execution::common_hints::ExecutionMode::Execute,
                true,
            )?;

        let mut transactional_state = CachedState::create_transactional(&mut state.state);
        let res = call
            .execute(
                &mut transactional_state,
                &mut Default::default(),
                &mut execution_context,
            )
            .map_err(|err| {
                Error::BlockifierTransactionError(
                    blockifier::transaction::errors::TransactionExecutionError::ExecutionError(err),
                )
            })?;

        Ok(res
            .execution
            .retdata
            .0
            .into_iter()
            .map(Felt::from)
            .collect())
    }

    pub fn estimate_fee(
        &mut self,
        block_id: &BlockId,
        transactions: &[BroadcastedTransaction],
        simulation_flags: &[SimulationFlag],
    ) -> DevnetResult<Vec<FeeEstimateWrapper>> {
        let mut skip_validate = false;
        for flag in simulation_flags.iter() {
            if *flag == SimulationFlag::SkipValidate {
                skip_validate = true;
            }
        }
        estimations::estimate_fee(self, block_id, transactions, None, Some(!skip_validate))
    }

    pub fn estimate_message_fee(
        &mut self,
        block_id: &BlockId,
        message: MsgFromL1,
    ) -> DevnetResult<FeeEstimateWrapper> {
        estimations::estimate_message_fee(self, block_id, message)
    }

    pub fn add_declare_transaction(
        &mut self,
        declare_transaction: BroadcastedDeclareTransaction,
    ) -> DevnetResult<(TransactionHash, ClassHash)> {
        add_declare_transaction::add_declare_transaction(self, declare_transaction)
    }

    /// returning the chain id as object
    pub fn chain_id(&self) -> ChainId {
        self.config.chain_id
    }

    pub fn add_deploy_account_transaction(
        &mut self,
        deploy_account_transaction: BroadcastedDeployAccountTransaction,
    ) -> DevnetResult<(TransactionHash, ContractAddress)> {
        add_deploy_account_transaction::add_deploy_account_transaction(
            self,
            deploy_account_transaction,
        )
    }

    pub fn add_invoke_transaction(
        &mut self,
        invoke_transaction: BroadcastedInvokeTransaction,
    ) -> DevnetResult<TransactionHash> {
        add_invoke_transaction::add_invoke_transaction(self, invoke_transaction)
    }

    pub fn add_l1_handler_transaction(
        &mut self,
        l1_handler_transaction: L1HandlerTransaction,
    ) -> DevnetResult<TransactionHash> {
        add_l1_handler_transaction::add_l1_handler_transaction(self, l1_handler_transaction)
    }

    /// Creates an invoke tx for minting, using the chargeable account.
    pub async fn mint(
        &mut self,
        address: ContractAddress,
        amount: BigUint,
        erc20_address: ContractAddress,
    ) -> DevnetResult<Felt> {
        let sufficiently_big_max_fee = self.config.gas_price.get() * 1_000_000;
        let chargeable_address_felt = Felt::from_prefixed_hex_str(CHARGEABLE_ACCOUNT_ADDRESS)?;
        let nonce = self
            .state
            .get_nonce_at(starknet_api::core::ContractAddress::try_from(
                starknet_api::hash::StarkFelt::from(chargeable_address_felt),
            )?)?;

        let (high, low) = split_biguint(amount)?;

        let calldata = vec![Felt::from(address).into(), low.into(), high.into()];

        let raw_execution = RawExecution {
            calls: vec![Call {
                to: erc20_address.into(),
                selector: get_selector_from_name("mint").unwrap(),
                calldata: calldata.clone(),
            }],
            nonce: Felt::from(nonce.0).into(),
            max_fee: FieldElement::from(sufficiently_big_max_fee),
        };

        // generate msg hash (not the same as tx hash)
        let chain_id_felt: Felt = self.config.chain_id.to_felt();
        let msg_hash_felt =
            raw_execution.transaction_hash(chain_id_felt.into(), chargeable_address_felt.into());

        // generate signature by signing the msg hash
        let signer = starknet_rs_signers::LocalWallet::from(
            starknet_rs_signers::SigningKey::from_secret_scalar(
                FieldElement::from_hex_be(CHARGEABLE_ACCOUNT_PRIVATE_KEY).unwrap(),
            ),
        );
        let signature = signer.sign_hash(&msg_hash_felt).await?;

        let invoke_tx = BroadcastedInvokeTransactionV1 {
            sender_address: ContractAddress::new(chargeable_address_felt)?,
            calldata: raw_execution
                .raw_calldata()
                .into_iter()
                .map(|c| c.into())
                .collect(),
            common: BroadcastedTransactionCommon {
                max_fee: Fee(sufficiently_big_max_fee),
                version: Felt::from(1),
                signature: vec![signature.r.into(), signature.s.into()],
                nonce: nonce.0.into(),
            },
        };

        // apply the invoke tx
        add_invoke_transaction::add_invoke_transaction(
            self,
            BroadcastedInvokeTransaction::V1(invoke_tx),
        )
    }

    pub fn block_state_update(&self, block_id: &BlockId) -> DevnetResult<StateUpdate> {
        state_update::state_update_by_block_id(self, block_id)
    }

    pub fn abort_blocks(&mut self, starting_block_hash: Felt) -> DevnetResult<Vec<Felt>> {
        if self.config.state_archive != StateArchiveCapacity::Full {
            return Err(Error::UnsupportedAction {
                msg: ("The abort blocks feature requires state-archive-capacity set to full."
                    .into()),
            });
        }

        if self.blocks.aborted_blocks.contains(&starting_block_hash) {
            return Err(Error::UnsupportedAction {
                msg: "Block is already aborted".into(),
            });
        }

        let genesis_block_number = if let Some(block_number) = self.config.fork_config.block_number
        {
            block_number + 1
        } else {
            DEVNET_DEFAULT_STARTING_BLOCK_NUMBER
        };
        let genesis_block = self
            .blocks
            .get_by_block_id(&BlockId::Number(genesis_block_number))
            .unwrap();

        if starting_block_hash == genesis_block.block_hash() {
            return Err(Error::UnsupportedAction {
                msg: "Genesis block can't be aborted".into(),
            });
        }

        let mut next_block_to_abort_hash =
            self.blocks
                .last_block_hash
                .ok_or(Error::UnsupportedAction {
                    msg: "No blocks to abort".into(),
                })?;
        let mut reached_starting_block = false;
        let mut aborted: Vec<Felt> = Vec::new();

        // Abort blocks from latest to starting (iterating backwards) and revert transactions.
        while !reached_starting_block {
            reached_starting_block = next_block_to_abort_hash == starting_block_hash;
            let block_to_abort = self.blocks.hash_to_block.get_mut(&next_block_to_abort_hash);

            if let Some(block) = block_to_abort {
                block.status = BlockStatus::Rejected;
                self.blocks.num_to_hash.shift_remove(&block.block_number());

                // Revert transactions
                for tx_hash in block.get_transactions() {
                    let tx = self
                        .transactions
                        .get_by_hash_mut(tx_hash)
                        .ok_or(Error::NoTransaction)?;
                    tx.execution_result = ExecutionResult::Reverted {
                        reason: "Block aborted manually".to_string(),
                    };
                }

                aborted.push(block.block_hash());

                // Update next block hash to abort
                next_block_to_abort_hash = block.parent_hash();
            }
        }
        let last_reached_block_hash = next_block_to_abort_hash;

        // Update last_block_hash based on last reached block and revert state only if
        // starting block is reached in while loop.
        if reached_starting_block {
            let current_block = self
                .blocks
                .hash_to_block
                .get(&last_reached_block_hash)
                .ok_or(Error::NoBlock)?;
            self.blocks.last_block_hash = Some(current_block.block_hash());

            let reverted_state = self
                .blocks
                .hash_to_state
                .get(&current_block.block_hash())
                .ok_or(Error::NoStateAtBlock {
                    block_id: BlockId::Number(current_block.block_number().0),
                })?;
            self.state = reverted_state.clone_historic();
        }

        self.blocks.aborted_blocks = aborted.clone();

        Ok(aborted)
    }

    pub fn get_block_txs_count(&self, block_id: &BlockId) -> DevnetResult<u64> {
        let block = self
            .blocks
            .get_by_block_id(block_id)
            .ok_or(Error::NoBlock)?;

        Ok(block.get_transactions().len() as u64)
    }

    pub fn contract_nonce_at_block(
        &mut self,
        block_id: &BlockId,
        contract_address: ContractAddress,
    ) -> DevnetResult<Felt> {
        let state = self.get_mut_state_at(block_id)?;
        state.assert_contract_deployed(contract_address)?;
        Ok(state.get_nonce_at(contract_address.try_into()?)?.into())
    }

    pub fn contract_storage_at_block(
        &mut self,
        block_id: &BlockId,
        contract_address: ContractAddress,
        storage_key: PatriciaKey,
    ) -> DevnetResult<Felt> {
        let state = self.get_mut_state_at(block_id)?;
        state.assert_contract_deployed(contract_address)?;
        Ok(state
            .get_storage_at(contract_address.try_into()?, storage_key.try_into()?)?
            .into())
    }

    pub fn get_block(&self, block_id: &BlockId) -> DevnetResult<StarknetBlock> {
        let block = self
            .blocks
            .get_by_block_id(block_id)
            .ok_or(Error::NoBlock)?;
        Ok(block.clone())
    }

    pub fn get_block_with_transactions(&self, block_id: &BlockId) -> DevnetResult<Block> {
        let block = self
            .blocks
            .get_by_block_id(block_id)
            .ok_or(Error::NoBlock)?;
        let transactions = block
            .get_transactions()
            .iter()
            .map(|transaction_hash| {
                self.transactions
                    .get_by_hash(*transaction_hash)
                    .ok_or(Error::NoTransaction)
                    .map(|transaction| transaction.inner.clone())
            })
            .collect::<DevnetResult<Vec<TransactionWithHash>>>()?;

        Ok(Block {
            status: *block.status(),
            header: BlockHeader::from(block),
            transactions: Transactions::Full(transactions),
        })
    }

    pub fn get_block_with_receipts(&self, block_id: BlockId) -> DevnetResult<Block> {
        let block = self
            .blocks
            .get_by_block_id(&block_id)
            .ok_or(Error::NoBlock)?;
        let mut transaction_receipts: Vec<TransactionWithReceipt> = vec![];

        for transaction_hash in block.get_transactions() {
            let sn_transaction = self
                .transactions
                .get_by_hash(*transaction_hash)
                .ok_or(Error::NoTransaction)?;

            let transaction = sn_transaction.inner.clone();
            let mut receipt = sn_transaction.get_receipt()?;

            // remove the fields block_hash and block_number, because they are not needed as per the
            // spec
            // @Mario: waiting for the final decision on this fields, so we can refactor this.
            // Currently the spec is at 0.7.0-rc.1
            let common_field = match receipt {
                TransactionReceipt::Deploy(DeployTransactionReceipt { ref mut common, .. })
                | TransactionReceipt::L1Handler(L1HandlerTransactionReceipt {
                    ref mut common,
                    ..
                })
                | TransactionReceipt::Common(ref mut common) => common,
            };
            common_field.maybe_pending_properties.block_hash = None;
            common_field.maybe_pending_properties.block_number = None;

            transaction_receipts.push(TransactionWithReceipt {
                receipt,
                transaction: transaction.transaction,
            });
        }

        Ok(Block {
            status: *block.status(),
            header: BlockHeader::from(block),
            transactions: Transactions::FullWithReceipts(transaction_receipts),
        })
    }

    pub fn get_transaction_by_block_id_and_index(
        &self,
        block_id: &BlockId,
        index: u64,
    ) -> DevnetResult<&TransactionWithHash> {
        let block = self.get_block(block_id)?;
        let transaction_hash = block
            .get_transactions()
            .get(index as usize)
            .ok_or(Error::InvalidTransactionIndexInBlock)?;

        self.get_transaction_by_hash(*transaction_hash)
    }

    pub fn get_latest_block(&self) -> DevnetResult<StarknetBlock> {
        let block = self
            .blocks
            .get_by_block_id(&BlockId::Tag(starknet_rs_core::types::BlockTag::Latest))
            .ok_or(Error::NoBlock)?;

        Ok(block.clone())
    }

    pub fn get_transaction_by_hash(
        &self,
        transaction_hash: Felt,
    ) -> DevnetResult<&TransactionWithHash> {
        self.transactions
            .get_by_hash(transaction_hash)
            .map(|starknet_transaction| &starknet_transaction.inner)
            .ok_or(Error::NoTransaction)
    }

    pub fn get_events(
        &self,
        from_block: Option<BlockId>,
        to_block: Option<BlockId>,
        address: Option<ContractAddress>,
        keys: Option<Vec<Vec<Felt>>>,
        skip: usize,
        limit: Option<usize>,
    ) -> DevnetResult<(Vec<EmittedEvent>, bool)> {
        events::get_events(self, from_block, to_block, address, keys, skip, limit)
    }

    pub fn get_transaction_receipt_by_hash(
        &self,
        transaction_hash: &TransactionHash,
    ) -> DevnetResult<TransactionReceipt> {
        let transaction_to_map = self
            .transactions
            .get(transaction_hash)
            .ok_or(Error::NoTransaction)?;

        transaction_to_map.get_receipt()
    }

    pub fn get_transaction_trace_by_hash(
        &self,
        transaction_hash: TransactionHash,
    ) -> DevnetResult<TransactionTrace> {
        let tx = self
            .transactions
            .get(&transaction_hash)
            .ok_or(Error::NoTransaction)?;
        tx.get_trace().ok_or(Error::NoTransactionTrace)
    }

    pub fn get_transaction_traces_from_block(
        &self,
        block_id: &BlockId,
    ) -> DevnetResult<Vec<BlockTransactionTrace>> {
        let transactions = self.get_block_with_transactions(block_id)?.transactions;

        let mut traces = Vec::new();
        if let Transactions::Full(txs) = transactions {
            for tx in txs {
                let tx_hash = *tx.get_transaction_hash();
                let trace = self.get_transaction_trace_by_hash(tx_hash)?;
                let block_trace = BlockTransactionTrace {
                    transaction_hash: tx_hash,
                    trace_root: trace,
                };

                traces.push(block_trace);
            }
        }

        Ok(traces)
    }

    pub fn get_transaction_execution_and_finality_status(
        &self,
        transaction_hash: TransactionHash,
    ) -> DevnetResult<(TransactionExecutionStatus, TransactionFinalityStatus)> {
        let transaction = self
            .transactions
            .get(&transaction_hash)
            .ok_or(Error::NoTransaction)?;

        Ok((
            transaction.execution_result.status(),
            transaction.finality_status,
        ))
    }

    pub fn simulate_transactions(
        &mut self,
        block_id: &BlockId,
        transactions: &[BroadcastedTransaction],
        simulation_flags: Vec<SimulationFlag>,
    ) -> DevnetResult<Vec<SimulatedTransaction>> {
        let chain_id = self.chain_id().to_felt();
        let block_context = self.block_context.clone();

        let mut skip_validate = false;
        let mut skip_fee_charge = false;
        for flag in simulation_flags.iter() {
            match flag {
                SimulationFlag::SkipValidate => {
                    skip_validate = true;
                }
                SimulationFlag::SkipFeeCharge => skip_fee_charge = true,
            }
        }

        let mut transactions_traces: Vec<TransactionTrace> = vec![];
        let state = self.get_mut_state_at(block_id)?;
        let mut transactional_rpc_contract_classes = state.clone_rpc_contract_classes();
        let mut transactional_state = CachedState::new(
            CachedState::create_transactional(&mut state.state),
            GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
        );
        for broadcasted_transaction in transactions.iter() {
            let blockifier_transaction =
                broadcasted_transaction.to_blockifier_account_transaction(&chain_id)?;
            let tx_execution_info = blockifier_transaction.execute(
                &mut transactional_state,
                &block_context,
                !skip_fee_charge,
                !skip_validate,
            )?;

            let state_diff: ThinStateDiff = StateDiff::generate(
                &mut transactional_state,
                &mut transactional_rpc_contract_classes,
            )?
            .into();
            let trace = create_trace(
                &mut transactional_state,
                broadcasted_transaction.get_type(),
                &tx_execution_info,
                state_diff,
            )?;
            transactions_traces.push(trace);
        }

        let estimated = estimations::estimate_fee(
            self,
            block_id,
            transactions,
            Some(!skip_fee_charge),
            Some(!skip_validate),
        )?;

        // if the underlying simulation is correct, this should never be the case
        // in alignment with always avoiding assertions in production code, this has to be done
        if transactions_traces.len() != estimated.len() {
            return Err(Error::UnexpectedInternalError {
                msg: format!(
                    "Non-matching number of simulations ({}) and estimations ({})",
                    transactions_traces.len(),
                    estimated.len()
                ),
            });
        }

        let simulation_results = transactions_traces
            .into_iter()
            .zip(estimated)
            .map(|(trace, fee_estimation)| SimulatedTransaction {
                transaction_trace: trace,
                fee_estimation,
            })
            .collect();

        Ok(simulation_results)
    }

    /// create new block from pending one
    pub fn create_block(&mut self) -> DevnetResult<(), Error> {
        self.generate_new_block(StateDiff::default())?;
        Ok(())
    }

    // Create block and add DumpEvent
    pub fn create_block_dump_event(
        &mut self,
        dump_event: Option<DumpEvent>,
    ) -> DevnetResult<(), Error> {
        self.create_block()?;

        // handle custom event if provided e.g. SetTime, IncreaseTime, otherwise log create block
        // events
        match dump_event {
            Some(event) => self.handle_dump_event(event)?,
            None => self.handle_dump_event(DumpEvent::CreateBlock)?,
        }

        Ok(())
    }

    // Set time and optionally create a new block
    pub fn set_time(&mut self, timestamp: u64, create_block: bool) -> DevnetResult<(), Error> {
        self.set_block_timestamp_shift(
            timestamp as i64 - Starknet::get_unix_timestamp_as_seconds() as i64,
        );

        if create_block {
            self.set_next_block_timestamp(timestamp);
            self.create_block()?;
            self.handle_dump_event(DumpEvent::SetTime(timestamp))?;
            self.handle_dump_event(DumpEvent::CreateBlock)?;
        } else {
            self.set_next_block_timestamp(timestamp);
            self.handle_dump_event(DumpEvent::SetTime(timestamp))?;
        }

        Ok(())
    }

    // Set timestamp shift and create empty block
    pub fn increase_time(&mut self, time_shift: u64) -> DevnetResult<(), Error> {
        self.set_block_timestamp_shift(self.pending_block_timestamp_shift + time_shift as i64);
        self.create_block_dump_event(Some(DumpEvent::IncreaseTime(time_shift)))
    }

    // Set timestamp shift for next blocks
    pub fn set_block_timestamp_shift(&mut self, timestamp: i64) {
        self.pending_block_timestamp_shift = timestamp;
    }

    // Set next block timestamp
    pub fn set_next_block_timestamp(&mut self, timestamp: u64) {
        self.next_block_timestamp = Some(timestamp);
    }

    pub fn get_unix_timestamp_as_seconds() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("should get current UNIX timestamp")
            .as_secs()
    }
}
