use sha3::{Digest, Keccak256};
use starknet_crypto::{pedersen_hash, FieldElement};

#[allow(dead_code)]
const DEFAULT_ENTRY_POINT_NAME: &str = "__default__";

#[allow(dead_code)]
const DEFAULT_L1_ENTRY_POINT_NAME: &str = "__l1_default__";

// 2 ** 251 - 256
const ADDR_BOUND: FieldElement = FieldElement::from_mont([
    18446743986131443745,
    160989183,
    18446744073709255680,
    576459263475590224,
]);

#[allow(dead_code)]
// Cairo string of "STARKNET_CONTRACT_ADDRESS"
const CONTRACT_ADDRESS_PREFIX: FieldElement = FieldElement::from_mont([
    3829237882463328880,
    17289941567720117366,
    8635008616843941496,
    533439743893157637,
]);

#[allow(dead_code)]
/// The uniqueness settings for UDC deployments.
#[derive(Debug, Clone)]
pub enum UdcUniqueness {
    NotUnique,
    Unique(UdcUniqueSettings),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UdcUniqueSettings {
    pub deployer_address: FieldElement,
    pub udc_contract_address: FieldElement,
}

#[allow(dead_code)]
mod errors {
    use core::fmt::{Display, Formatter, Result};

    #[derive(Debug)]
    pub struct NonAsciiNameError;

    #[derive(Debug)]
    pub enum CairoShortStringToFeltError {
        NonAsciiCharacter,
        StringTooLong,
    }

    #[derive(Debug)]
    pub enum ParseCairoShortStringError {
        ValueOutOfRange,
        UnexpectedNullTerminator,
    }

    #[cfg(feature = "std")]
    impl std::error::Error for NonAsciiNameError {}

    impl Display for NonAsciiNameError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "the provided name contains non-ASCII characters")
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for CairoShortStringToFeltError {}

    impl Display for CairoShortStringToFeltError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::NonAsciiCharacter => {
                    write!(f, "Cairo string can only contain ASCII characters")
                }
                Self::StringTooLong => {
                    write!(f, "short string exceeds maximum length of 31 characters")
                }
            }
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for ParseCairoShortStringError {}

    impl Display for ParseCairoShortStringError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::ValueOutOfRange => write!(f, "field element value out of range"),
                Self::UnexpectedNullTerminator => write!(f, "unexpected null terminator"),
            }
        }
    }
}
pub use errors::{CairoShortStringToFeltError, NonAsciiNameError, ParseCairoShortStringError};
use tokio::io::AsyncReadExt;

use crate::{
    errors::RunnerError,
    transports::{http::HttpTransport, JsonRpcClient},
};

use super::{
    codegen::FlattenedSierraClass,
    contract::{CompiledClass, SierraClass},
    crypto::compute_hash_on_elements,
};

/// A variant of eth-keccak that computes a value that fits in a Starknet field element.
pub fn starknet_keccak(data: &[u8]) -> FieldElement {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut hash = hasher.finalize();

    // Remove the first 6 bits
    hash[0] &= 0b00000011;

    // Because we know hash is always 32 bytes
    FieldElement::from_bytes_be(unsafe { &*(hash[..].as_ptr() as *const [u8; 32]) }).unwrap()
}

#[allow(dead_code)]
pub fn get_selector_from_name(func_name: &str) -> Result<FieldElement, NonAsciiNameError> {
    if func_name == DEFAULT_ENTRY_POINT_NAME || func_name == DEFAULT_L1_ENTRY_POINT_NAME {
        Ok(FieldElement::ZERO)
    } else {
        let name_bytes = func_name.as_bytes();
        if name_bytes.is_ascii() {
            Ok(starknet_keccak(name_bytes))
        } else {
            Err(NonAsciiNameError)
        }
    }
}

#[allow(dead_code)]
pub fn get_storage_var_address(
    var_name: &str,
    args: &[FieldElement],
) -> Result<FieldElement, NonAsciiNameError> {
    let var_name_bytes = var_name.as_bytes();
    if var_name_bytes.is_ascii() {
        let mut res = starknet_keccak(var_name_bytes);
        for arg in args.iter() {
            res = pedersen_hash(&res, arg);
        }
        Ok(normalize_address(res))
    } else {
        Err(NonAsciiNameError)
    }
}

#[allow(dead_code)]
/// Converts Cairo short string to [FieldElement].
pub fn cairo_short_string_to_felt(str: &str) -> Result<FieldElement, CairoShortStringToFeltError> {
    if !str.is_ascii() {
        return Err(CairoShortStringToFeltError::NonAsciiCharacter);
    }
    if str.len() > 31 {
        return Err(CairoShortStringToFeltError::StringTooLong);
    }

    let ascii_bytes = str.as_bytes();

    let mut buffer = [0u8; 32];
    buffer[(32 - ascii_bytes.len())..].copy_from_slice(ascii_bytes);

    // The conversion will never fail
    Ok(FieldElement::from_bytes_be(&buffer).unwrap())
}

#[allow(dead_code)]
/// Converts [FieldElement] to Cairo short string.
pub fn parse_cairo_short_string(felt: &FieldElement) -> Result<String, ParseCairoShortStringError> {
    if felt == &FieldElement::ZERO {
        return Ok(String::new());
    }

    let be_bytes = felt.to_bytes_be();
    if be_bytes[0] > 0 {
        return Err(ParseCairoShortStringError::ValueOutOfRange);
    }

    let mut buffer = String::with_capacity(31);
    for byte in be_bytes.into_iter() {
        if byte == 0u8 {
            if !buffer.is_empty() {
                return Err(ParseCairoShortStringError::UnexpectedNullTerminator);
            }
        } else {
            buffer.push(byte as char)
        }
    }
    Ok(buffer)
}

#[allow(dead_code)]
/// Computes the target contract address of a "native" contract deployment. Use
/// `get_udc_deployed_address` instead if you want to compute the target address for deployments
/// through the Universal Deployer Contract.
pub fn get_contract_address(
    salt: FieldElement,
    class_hash: FieldElement,
    constructor_calldata: &[FieldElement],
    deployer_address: FieldElement,
) -> FieldElement {
    normalize_address(compute_hash_on_elements(&[
        CONTRACT_ADDRESS_PREFIX,
        deployer_address,
        salt,
        class_hash,
        compute_hash_on_elements(constructor_calldata),
    ]))
}

#[allow(dead_code)]
/// Computes the target contract address for deployments through the Universal Deploy Contract.
pub fn get_udc_deployed_address(
    salt: FieldElement,
    class_hash: FieldElement,
    uniqueness: &UdcUniqueness,
    constructor_calldata: &[FieldElement],
) -> FieldElement {
    match uniqueness {
        UdcUniqueness::NotUnique => {
            get_contract_address(salt, class_hash, constructor_calldata, FieldElement::ZERO)
        }
        UdcUniqueness::Unique(settings) => {
            let unique_salt = pedersen_hash(&settings.deployer_address, &salt);
            get_contract_address(
                unique_salt,
                class_hash,
                constructor_calldata,
                settings.udc_contract_address,
            )
        }
    }
}

pub fn normalize_address(address: FieldElement) -> FieldElement {
    address % ADDR_BOUND
}

#[allow(dead_code)]
pub fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    let rpc_url: String = std::env::var("STARKNET_RPC").unwrap_or("http://localhost:5050/".into());
    JsonRpcClient::new(HttpTransport::new(url::Url::parse(&rpc_url).unwrap()))
}

#[allow(dead_code)]
pub async fn get_compiled_contract(
    sierra_path: &str,
    casm_path: &str,
) -> Result<(FlattenedSierraClass, FieldElement), RunnerError> {
    let mut file = tokio::fs::File::open(sierra_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut sierra = String::default();
    file.read_to_string(&mut sierra)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let mut file = tokio::fs::File::open(casm_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            RunnerError::ReadFileError(
                "Contract json file not found, please execute scarb build command".to_string(),
            )
        } else {
            RunnerError::ReadFileError(e.to_string())
        }
    })?;
    let mut casm = String::default();
    file.read_to_string(&mut casm)
        .await
        .map_err(|e| RunnerError::ReadFileError(e.to_string()))?;

    let contract_artifact: SierraClass = serde_json::from_str(&sierra)?;
    let compiled_class: CompiledClass = serde_json::from_str(&casm)?;
    let casm_class_hash = compiled_class.class_hash().unwrap();
    let flattened_class = contract_artifact.clone().flatten().unwrap();
    Ok((flattened_class, casm_class_hash))
}
