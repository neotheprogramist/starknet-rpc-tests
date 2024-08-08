use starknet_devnet_types::contract_address::ContractAddress;
use starknet_devnet_types::emitted_event::{EmittedEvent, Event};
use starknet_devnet_types::felt::Felt;
use starknet_rs_core::types::BlockId;

use super::traits::HashIdentified;
use super::Starknet;
use super::{DevnetResult, Error};

/// The method returns transaction events, based on query and if there are more results to be
/// fetched in the form of a tuple (events, has_more).
///
/// # Arguments
///
/// * `from_block` - Optional. The block id to start the query from.
/// * `to_block` - Optional. The block id to end the query at.
/// * `contract_address` - Optional. The contract address to filter the events by.
/// * `keys_filter` - Optional. The keys to filter the events by.
/// * `skip` - The number of elements to skip.
/// * `limit` - Optional. The maximum number of elements to return.
pub(crate) fn get_events(
    starknet: &Starknet,
    from_block: Option<BlockId>,
    to_block: Option<BlockId>,
    contract_address: Option<ContractAddress>,
    keys_filter: Option<Vec<Vec<Felt>>>,
    mut skip: usize,
    limit: Option<usize>,
) -> DevnetResult<(Vec<EmittedEvent>, bool)> {
    let blocks = starknet.blocks.get_blocks(from_block, to_block)?;
    let mut events: Vec<EmittedEvent> = Vec::new();
    let mut elements_added = 0;

    // iterate over each block and get the transactions for each one
    // then iterate over each transaction events and filter them
    for block in blocks {
        for transaction_hash in block.get_transactions() {
            let transaction = starknet
                .transactions
                .get_by_hash(*transaction_hash)
                .ok_or(Error::NoTransaction)?;

            // filter the events from the transaction
            let filtered_transaction_events = transaction
                .get_events()
                .into_iter()
                .filter(|event| {
                    check_if_filter_applies_for_event(&contract_address, &keys_filter, event)
                })
                .skip_while(|_| {
                    if skip > 0 {
                        skip -= 1;
                        true
                    } else {
                        false
                    }
                });

            // produce an emitted event for each filtered transaction event
            for transaction_event in filtered_transaction_events {
                // check if there are more elements to fetch
                if let Some(limit) = limit {
                    if elements_added == limit {
                        return Ok((events, true));
                    }
                }

                let emitted_event = EmittedEvent {
                    transaction_hash: *transaction_hash,
                    block_hash: block.block_hash(),
                    block_number: block.block_number(),
                    keys: transaction_event.keys,
                    from_address: transaction_event.from_address,
                    data: transaction_event.data,
                };

                events.push(emitted_event);
                elements_added += 1;
            }
        }
    }

    Ok((events, false))
}

/// This method checks if the event applies to the provided filters and returns true or false
///
/// # Arguments
/// * `address` - Optional. The address to filter the event by.
/// * `keys_filter` - Optional. The keys to filter the event by.
/// * `event` - The event to check if it applies to the filters.
fn check_if_filter_applies_for_event(
    address: &Option<ContractAddress>,
    keys_filter: &Option<Vec<Vec<Felt>>>,
    event: &Event,
) -> bool {
    let address_condition = match &address {
        Some(from_contract_address) => event.from_address == *from_contract_address,
        None => true,
    };

    address_condition && check_if_filter_applies_for_event_keys(keys_filter, &event.keys)
}

/// This method checks if the keys apply to the keys_filter and returns true or false
///
/// # Arguments
/// * `keys_filter` - Optional. The values to filter the keys by.
/// * `keys` - The keys to check if they apply to the filter.
fn check_if_filter_applies_for_event_keys<T>(keys_filter: &Option<Vec<Vec<T>>>, keys: &[T]) -> bool
where
    T: PartialEq + Eq,
{
    match &keys_filter {
        Some(keys_filter) => {
            for (event_key, accepted_keys) in keys.iter().zip(keys_filter) {
                if !accepted_keys.is_empty() && !accepted_keys.contains(event_key) {
                    return false;
                }
            }

            true
        }
        None => true,
    }
}
