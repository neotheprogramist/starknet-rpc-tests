use std::fmt::{self, Formatter};

use serde::{de::Visitor, Deserializer};
use serde_with::{DeserializeAs, SerializeAs};
pub(crate) struct NumAsHex;

struct NumAsHexVisitorU64;
struct NumAsHexVisitorU128;

impl SerializeAs<u64> for NumAsHex {
    fn serialize_as<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{value:#x}"))
    }
}

impl SerializeAs<&u64> for NumAsHex {
    fn serialize_as<S>(value: &&u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{value:#x}"))
    }
}

impl<'de> DeserializeAs<'de, u64> for NumAsHex {
    fn deserialize_as<D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumAsHexVisitorU64)
    }
}

impl SerializeAs<u128> for NumAsHex {
    fn serialize_as<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{value:#x}"))
    }
}

impl<'de> DeserializeAs<'de, u128> for NumAsHex {
    fn deserialize_as<D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumAsHexVisitorU128)
    }
}

impl<'de> Visitor<'de> for NumAsHexVisitorU64 {
    type Value = u64;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string or number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match u64::from_str_radix(v.trim_start_matches("0x"), 16) {
            Ok(value) => Ok(value),
            Err(err) => Err(serde::de::Error::custom(format!(
                "invalid hex string: {err}"
            ))),
        }
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.try_into() {
            Ok(value) => self.visit_u64(value),
            Err(_) => Err(serde::de::Error::custom(format!(
                "value cannot be negative: {}",
                v
            ))),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

impl<'de> Visitor<'de> for NumAsHexVisitorU128 {
    type Value = u128;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string or number")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match u128::from_str_radix(v.trim_start_matches("0x"), 16) {
            Ok(value) => Ok(value),
            Err(err) => Err(serde::de::Error::custom(format!(
                "invalid hex string: {err}"
            ))),
        }
    }
}
pub(crate) mod u64_hex {
    use serde::de::Visitor;

    struct U64HexVisitor;

    pub fn serialize<S>(v: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#x}", v))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(U64HexVisitor)
    }

    impl<'de> Visitor<'de> for U64HexVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u64::from_str_radix(v.trim_start_matches("0x"), 16)
                .map_err(|err| serde::de::Error::custom(format!("invalid u64 hex string: {err}")))
        }
    }
}

#[allow(dead_code)]
pub(crate) mod u128_hex {
    use serde::de::Visitor;

    struct U128HexVisitor;

    pub fn serialize<S>(v: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#x}", v))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(U128HexVisitor)
    }

    impl<'de> Visitor<'de> for U128HexVisitor {
        type Value = u128;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            u128::from_str_radix(v.trim_start_matches("0x"), 16)
                .map_err(|err| serde::de::Error::custom(format!("invalid u128 hex string: {err}")))
        }
    }
}

#[allow(dead_code)]
pub(crate) mod u64_hex_opt {
    use serde::de::Visitor;

    struct U64HexOptVisitor;

    pub fn serialize<S>(v: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match v {
            Some(v) => serializer.serialize_str(&format!("{:#x}", v)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(U64HexOptVisitor)
    }

    impl<'de> Visitor<'de> for U64HexOptVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "null or string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(
                u64::from_str_radix(v.trim_start_matches("0x"), 16).map_err(|err| {
                    serde::de::Error::custom(format!("invalid u64 hex string: {err}"))
                })?,
            ))
        }
    }
}

mod block_id {
    use crate::unsigned_field_element::UfeHex;
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_with::serde_as;
    use starknet_crypto::FieldElement;

    use crate::{codegen::BlockTag, models::BlockId};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BlockIdDe {
        Hash(BlockHash),
        Number(BlockNumber),
        Tag(BlockTag),
    }

    #[serde_as]
    #[derive(Serialize, Deserialize)]
    #[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
    struct BlockHash {
        #[serde_as(as = "UfeHex")]
        block_hash: FieldElement,
    }

    #[derive(Serialize, Deserialize)]
    #[cfg_attr(feature = "no_unknown_fields", serde(deny_unknown_fields))]
    struct BlockNumber {
        block_number: u64,
    }

    impl Serialize for BlockId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                Self::Hash(hash) => {
                    BlockHash::serialize(&BlockHash { block_hash: *hash }, serializer)
                }
                Self::Number(number) => BlockNumber::serialize(
                    &BlockNumber {
                        block_number: *number,
                    },
                    serializer,
                ),
                Self::Tag(tag) => BlockTag::serialize(tag, serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for BlockId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(match BlockIdDe::deserialize(deserializer)? {
                BlockIdDe::Hash(hash) => Self::Hash(hash.block_hash),
                BlockIdDe::Number(number) => Self::Number(number.block_number),
                BlockIdDe::Tag(tag) => Self::Tag(tag),
            })
        }
    }
}

// Deriving the Serialize trait directly results in duplicate fields since the variants also write
// the tag fields when individually serialized.
mod enum_ser_impls {

    use serde::Serialize;

    use crate::models::BroadcastedDeclareTransaction;
    use crate::models::BroadcastedDeployAccountTransaction;
    use crate::models::BroadcastedInvokeTransaction;
    use crate::models::BroadcastedTransaction;
    use crate::models::DeclareTransaction;
    use crate::models::InvokeTransaction;
    use crate::models::Transaction;
    use crate::models::TransactionReceipt;
    use crate::models::TransactionTrace;
    use crate::transports::ExecuteInvocation;

    impl Serialize for Transaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Invoke(variant) => variant.serialize(serializer),
                // Self::L1Handler(variant) => variant.serialize(serializer),
                Self::Declare(variant) => variant.serialize(serializer),
                Self::Deploy(variant) => variant.serialize(serializer),
                // Self::DeployAccount(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for BroadcastedTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Invoke(variant) => variant.serialize(serializer),
                Self::Declare(variant) => variant.serialize(serializer),
                // Self::DeployAccount(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for InvokeTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::V0(variant) => variant.serialize(serializer),
                Self::V1(variant) => variant.serialize(serializer),
                Self::V3(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for DeclareTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::V0(variant) => variant.serialize(serializer),
                Self::V1(variant) => variant.serialize(serializer),
                Self::V2(variant) => variant.serialize(serializer),
                Self::V3(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for BroadcastedInvokeTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::V1(variant) => variant.serialize(serializer),
                Self::V3(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for BroadcastedDeclareTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::V1(variant) => variant.serialize(serializer),
                Self::V2(variant) => variant.serialize(serializer),
                Self::V3(variant) => variant.serialize(serializer),
            }
        }
    }
    impl Serialize for TransactionTrace {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Invoke(variant) => variant.serialize(serializer),
                Self::DeployAccount(variant) => variant.serialize(serializer),
                Self::L1Handler(variant) => variant.serialize(serializer),
                Self::Declare(variant) => variant.serialize(serializer),
            }
        }
    }
    impl Serialize for BroadcastedDeployAccountTransaction {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::V1(variant) => variant.serialize(serializer),
                Self::V3(variant) => variant.serialize(serializer),
            }
        }
    }

    impl Serialize for TransactionReceipt {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Invoke(variant) => variant.serialize(serializer),
                Self::L1Handler(variant) => variant.serialize(serializer),
                Self::Declare(variant) => variant.serialize(serializer),
                Self::Deploy(variant) => variant.serialize(serializer),
                Self::DeployAccount(variant) => variant.serialize(serializer),
            }
        }
    }
    impl Serialize for ExecuteInvocation {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match self {
                Self::Success(variant) => variant.serialize(serializer),
                Self::Reverted(variant) => variant.serialize(serializer),
            }
        }
    }
}
