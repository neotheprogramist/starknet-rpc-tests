use starknet_devnet_types::emitted_event::Event as DevnetEvent;
use starknet_devnet_types::{rpc::transaction_receipt::TransactionReceipt, traits::ToHexString};
use std::str::FromStr;

use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_with::serde_conv;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct EmittedEvent {
    pub transaction_hash: Felt,
    pub events: Vec<Event>,
}

// #[serde_with::serde_as]
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Event {
    // #[serde_as(as = "Vec<EventDataAsDecimalStr>")]
    pub data: Vec<Felt>,
    pub from_address: Felt,
    // #[serde_as(as = "Vec<EventKeyAsDecimalStr>")]
    pub keys: Vec<Felt>,
}

serde_conv!(
    EventDataAsDecimalStr,
    Felt,
    |serialize_me: &Felt| starkhash_to_dec_str(&serialize_me),
    |s: &str| starkhash_from_dec_str(s)
);
serde_conv!(
    EventKeyAsDecimalStr,
    Felt,
    |serialize_me: &Felt| starkhash_to_dec_str(&serialize_me),
    |s: &str| starkhash_from_dec_str(s)
);

/// A helper conversion function. Only use with __sequencer API related types__.
fn starkhash_to_dec_str(h: &Felt) -> String {
    let b = h.to_bytes_be();
    let b = BigUint::from_bytes_be(&b);
    b.to_str_radix(10)
}

/// A helper conversion function. Only use with __sequencer API related types__.
fn starkhash_from_dec_str(s: &str) -> Result<Felt, anyhow::Error> {
    match BigUint::from_str(s) {
        Ok(b) => {
            let h = Felt::from_bytes_be_slice(&b.to_bytes_be());
            Ok(h)
        }
        Err(_) => {
            let h = Felt::from_dec_str(s).unwrap();
            Ok(h)
        }
    }
}

pub fn extract_emmited_events(transaction_receipts: Vec<TransactionReceipt>) -> Vec<EmittedEvent> {
    let mut events: Vec<EmittedEvent> = vec![];
    for receipt in transaction_receipts.iter() {
        match receipt {
            TransactionReceipt::Common(tx_receipt) => {
                events.push(EmittedEvent {
                    transaction_hash: Felt::from_hex_unchecked(
                        tx_receipt.transaction_hash.to_prefixed_hex_str().as_str(),
                    ),
                    events: convert_events(tx_receipt.events.clone()),
                });
            }
            TransactionReceipt::Deploy(tx_receipt) => {
                events.push(EmittedEvent {
                    transaction_hash: Felt::from_hex_unchecked(
                        tx_receipt
                            .common
                            .transaction_hash
                            .to_prefixed_hex_str()
                            .as_str(),
                    ),
                    events: convert_events(tx_receipt.common.events.clone()),
                });
            }
            TransactionReceipt::L1Handler(tx_receipt) => {
                events.push(EmittedEvent {
                    transaction_hash: Felt::from_hex_unchecked(
                        tx_receipt
                            .common
                            .transaction_hash
                            .to_prefixed_hex_str()
                            .as_str(),
                    ),
                    events: convert_events(tx_receipt.common.events.clone()),
                });
            }
        }
    }
    events
}

fn convert_events(old_events: Vec<DevnetEvent>) -> Vec<Event> {
    old_events
        .into_iter()
        .map(|event| {
            let new_data: Vec<Felt> = event
                .data
                .into_iter()
                .map(|felt| Felt::from_hex_unchecked(felt.to_prefixed_hex_str().as_str()))
                .collect();

            let new_from_address =
                Felt::from_hex_unchecked(event.from_address.to_prefixed_hex_str().as_str());

            let new_keys: Vec<Felt> = event
                .keys
                .into_iter()
                .map(|felt| Felt::from_hex_unchecked(felt.to_prefixed_hex_str().as_str()))
                .collect();

            Event {
                data: new_data,
                from_address: new_from_address,
                keys: new_keys,
            }
        })
        .collect()
}

pub fn get_events_count(events: Vec<EmittedEvent>) -> u32 {
    let mut count = 0;
    for event in events.iter() {
        count += event.events.len() as u32;
    }
    count
}
