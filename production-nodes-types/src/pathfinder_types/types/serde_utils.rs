use std::error::Error;

use primitive_types::H160;
use serde::de::Visitor;
use serde_with::DeserializeAs;

use super::gateway_state_update::{EthereumAddress, GasPrice};

pub struct GasPriceAsHexStr;

impl<'de> DeserializeAs<'de, GasPrice> for GasPriceAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<GasPrice, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GasPriceVisitor;

        impl<'de> Visitor<'de> for GasPriceVisitor {
            type Value = GasPrice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 32 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<16>(v)
                    .map_err(serde::de::Error::custom)
                    .map(GasPrice::from_be_bytes)
            }
        }

        deserializer.deserialize_str(GasPriceVisitor)
    }
}

impl Error for HexParseError {}

const OVERFLOW_MSG: &str = "The maximum field value was exceeded.";

impl std::fmt::Display for HexParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNibble(n) => f.write_fmt(format_args!("Invalid nibble found: 0x{:x}", *n)),
            Self::InvalidLength { max, actual } => {
                f.write_fmt(format_args!("More than {} digits found: {}", *max, *actual))
            }
            Self::Overflow => f.write_str(OVERFLOW_MSG),
        }
    }
}

impl GasPrice {
    pub const ZERO: GasPrice = GasPrice(0u128);

    /// Returns the big-endian representation of this [GasPrice].
    pub fn to_be_bytes(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Constructs [GasPrice] from an array of bytes. Big endian byte order is
    /// assumed.
    pub fn from_be_bytes(src: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(src))
    }
}

/// A convenience function which parses a hex string into a byte array.
///
/// Supports both upper and lower case hex strings, as well as an
/// optional "0x" prefix.
fn bytes_from_hex_str<const N: usize>(hex_str: &str) -> Result<[u8; N], HexParseError> {
    fn parse_hex_digit(digit: u8) -> Result<u8, HexParseError> {
        match digit {
            b'0'..=b'9' => Ok(digit - b'0'),
            b'A'..=b'F' => Ok(digit - b'A' + 10),
            b'a'..=b'f' => Ok(digit - b'a' + 10),
            other => Err(HexParseError::InvalidNibble(other)),
        }
    }

    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    if hex_str.len() > N * 2 {
        return Err(HexParseError::InvalidLength {
            max: N * 2,
            actual: hex_str.len(),
        });
    }

    let mut buf = [0u8; N];

    // We want the result in big-endian so reverse iterate over each pair of
    // nibbles.
    let chunks = hex_str.as_bytes().rchunks_exact(2);

    // Handle a possible odd nibble remaining nibble.
    let odd_nibble = chunks.remainder();
    if !odd_nibble.is_empty() {
        let full_bytes = hex_str.len() / 2;
        buf[N - 1 - full_bytes] = parse_hex_digit(odd_nibble[0])?;
    }

    for (i, c) in chunks.enumerate() {
        // Indexing c[0] and c[1] are safe since chunk-size is 2.
        buf[N - 1 - i] = parse_hex_digit(c[0])? << 4 | parse_hex_digit(c[1])?;
    }

    Ok(buf)
}

/// Error returned by [Felt::from_hex_str] indicating an invalid hex string.
#[derive(Debug, PartialEq, Eq)]
pub enum HexParseError {
    InvalidNibble(u8),
    InvalidLength { max: usize, actual: usize },
    Overflow,
}

pub struct EthereumAddressAsHexStr;

impl<'de> DeserializeAs<'de, EthereumAddress> for EthereumAddressAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<EthereumAddress, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EthereumAddressVisitor;

        impl<'de> Visitor<'de> for EthereumAddressVisitor {
            type Value = EthereumAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 40 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<{ H160::len_bytes() }>(v)
                    .map_err(serde::de::Error::custom)
                    .map(|b| EthereumAddress(H160::from(b)))
            }
        }

        deserializer.deserialize_str(EthereumAddressVisitor)
    }
}

pub fn bytes_as_hex_str<'a>(bytes: &'a [u8], buf: &'a mut [u8]) -> &'a str {
    let expected_buf_len = bytes.len() * 2 + 2;
    assert!(
        buf.len() >= expected_buf_len,
        "buffer size is {}, expected at least {}",
        buf.len(),
        expected_buf_len
    );

    if !bytes.iter().any(|b| *b != 0) {
        return "0x0";
    }

    let (it, start, len) = skip_zeros(bytes);
    let res = it_to_hex_str(it, start, len, buf);
    // Unwrap is safe because `buf` holds valid UTF8 characters.
    std::str::from_utf8(res).unwrap()
}

/// The first stage of conversion - skip leading zeros
fn skip_zeros(bytes: &[u8]) -> (impl Iterator<Item = &u8>, usize, usize) {
    // Skip all leading zero bytes
    let it = bytes.iter().skip_while(|&&b| b == 0);
    let num_bytes = it.clone().count();
    let skipped = bytes.len() - num_bytes;
    // The first high nibble can be 0
    let start = if bytes[skipped] < 0x10 { 1 } else { 2 };
    // Number of characters to display
    let len = start + num_bytes * 2;
    (it, start, len)
}

/// The second stage of conversion - map bytes to hex str
fn it_to_hex_str<'a>(
    it: impl Iterator<Item = &'a u8>,
    start: usize,
    len: usize,
    buf: &'a mut [u8],
) -> &'a [u8] {
    const LUT: [u8; 16] = *b"0123456789abcdef";
    buf[0] = b'0';
    // Same small lookup table is ~25% faster than hex::encode_from_slice 🤷
    it.enumerate().for_each(|(i, &b)| {
        let idx = b as usize;
        let pos = start + i * 2;
        let x = [LUT[(idx & 0xf0) >> 4], LUT[idx & 0x0f]];
        buf[pos..pos + 2].copy_from_slice(&x);
    });
    buf[1] = b'x';
    &buf[..len]
}
