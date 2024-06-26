use std::fmt::{self, Formatter};

use serde::{
    de::{Error as DeError, Visitor},
    Deserializer, Serializer,
};
use serde_with::{DeserializeAs, SerializeAs};
use starknet_crypto::FieldElement;

pub struct UfeHex;

#[allow(dead_code)]
pub struct UfeHexOption;

#[allow(dead_code)]
pub struct UfePendingBlockHash;

struct UfeHexVisitor;
struct UfeHexOptionVisitor;
struct UfePendingBlockHashVisitor;

impl SerializeAs<FieldElement> for UfeHex {
    fn serialize_as<S>(value: &FieldElement, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{value:#x}"))
    }
}

impl<'de> DeserializeAs<'de, FieldElement> for UfeHex {
    fn deserialize_as<D>(deserializer: D) -> Result<FieldElement, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UfeHexVisitor)
    }
}

impl<'de> Visitor<'de> for UfeHexVisitor {
    type Value = FieldElement;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        FieldElement::from_hex_be(v)
            .map_err(|err| DeError::custom(format!("invalid hex string: {err}")))
    }
}

impl SerializeAs<Option<FieldElement>> for UfeHexOption {
    fn serialize_as<S>(value: &Option<FieldElement>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_str(&format!("{value:#064x}")),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> DeserializeAs<'de, Option<FieldElement>> for UfeHexOption {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<FieldElement>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UfeHexOptionVisitor)
    }
}

impl<'de> Visitor<'de> for UfeHexOptionVisitor {
    type Value = Option<FieldElement>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        match v {
            "" => Ok(None),
            _ => match FieldElement::from_hex_be(v) {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(DeError::custom(format!("invalid hex string: {err}"))),
            },
        }
    }
}

impl SerializeAs<Option<FieldElement>> for UfePendingBlockHash {
    fn serialize_as<S>(value: &Option<FieldElement>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.serialize_str(&format!("{value:#064x}")),
            // We don't know if it's `null` or `"pending"`
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> DeserializeAs<'de, Option<FieldElement>> for UfePendingBlockHash {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<FieldElement>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UfePendingBlockHashVisitor)
    }
}

impl<'de> Visitor<'de> for UfePendingBlockHashVisitor {
    type Value = Option<FieldElement>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if v.is_empty() || v == "pending" || v == "None" {
            Ok(None)
        } else {
            match FieldElement::from_hex_be(v) {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(DeError::custom(format!("invalid hex string: {err}"))),
            }
        }
    }
}
