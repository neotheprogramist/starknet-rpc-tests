use core::str::FromStr;
use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

// 0xFFfFfFffFFfffFFfFFfFFFFFffFFFffffFfFFFfF
const MAX_L1_ADDRESS: FieldElement = FieldElement::from_mont([
    18406070939574861858,
    74766790688767,
    18446743936270598144,
    461478224317121089,
]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthAddress {
    inner: [u8; 20],
}

struct EthAddressVisitor;

mod errors {
    use core::fmt::{Display, Formatter, Result};

    #[derive(Debug)]
    pub enum FromHexError {
        UnexpectedLength,
        InvalidHexString,
    }

    #[derive(Debug)]
    pub struct FromFieldElementError;

    #[derive(Debug)]
    pub struct FromBytesSliceError;

    impl std::error::Error for FromHexError {}

    impl std::error::Error for FromFieldElementError {}

    impl std::error::Error for FromBytesSliceError {}

    impl Display for FromHexError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::UnexpectedLength => {
                    write!(f, "unexpected length for ETH address")
                }
                Self::InvalidHexString => {
                    write!(f, "invalid hex string")
                }
            }
        }
    }

    impl Display for FromFieldElementError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "FieldElement value out of range")
        }
    }

    impl Display for FromBytesSliceError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "invalid slice for ETH address")
        }
    }
}
pub use errors::{FromBytesSliceError, FromFieldElementError, FromHexError};
use starknet_crypto::FieldElement;

impl EthAddress {
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self { inner: bytes }
    }

    pub fn from_hex(hex: &str) -> Result<Self, FromHexError> {
        hex.parse()
    }

    pub fn from_felt(felt: &FieldElement) -> Result<Self, FromFieldElementError> {
        felt.try_into()
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.inner
    }
}

impl Serialize for EthAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(self.inner)))
    }
}

impl<'de> Deserialize<'de> for EthAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(EthAddressVisitor)
    }
}

impl<'de> Visitor<'de> for EthAddressVisitor {
    type Value = EthAddress;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse()
            .map_err(|err| serde::de::Error::custom(format!("{}", err)))
    }
}

impl FromStr for EthAddress {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim_start_matches("0x");

        if value.len() == 40 {
            match hex::decode(value) {
                Ok(bytes) => {
                    Ok(Self {
                        // It's safe to unwrap here as the length must be 20
                        inner: bytes.try_into().unwrap(),
                    })
                }
                Err(_) => Err(FromHexError::InvalidHexString),
            }
        } else {
            Err(FromHexError::UnexpectedLength)
        }
    }
}

impl TryFrom<FieldElement> for EthAddress {
    type Error = FromFieldElementError;

    fn try_from(value: FieldElement) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&FieldElement> for EthAddress {
    type Error = FromFieldElementError;

    fn try_from(value: &FieldElement) -> Result<Self, Self::Error> {
        if value <= &MAX_L1_ADDRESS {
            let mut buffer = [0u8; 20];
            buffer.copy_from_slice(&value.to_bytes_be()[12..]);
            Ok(Self { inner: buffer })
        } else {
            Err(FromFieldElementError)
        }
    }
}

impl From<EthAddress> for FieldElement {
    fn from(value: EthAddress) -> Self {
        // Safe to unwrap here as the value is never out of range
        FieldElement::from_byte_slice_be(&value.inner).unwrap()
    }
}

impl TryFrom<&[u8]> for EthAddress {
    type Error = FromBytesSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 20 {
            Err(FromBytesSliceError)
        } else {
            // Safe to unwrap as we know length is 20.
            Ok(Self {
                inner: value.try_into().unwrap(),
            })
        }
    }
}

impl From<[u8; 20]> for EthAddress {
    fn from(value: [u8; 20]) -> Self {
        Self { inner: value }
    }
}
