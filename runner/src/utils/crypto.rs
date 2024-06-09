pub use starknet_crypto::{pedersen_hash, ExtendedSignature, Signature};
use starknet_crypto::{rfc6979_generate_k, sign, verify, FieldElement, SignError, VerifyError};

mod errors {
    use core::fmt::{Display, Formatter, Result};

    #[derive(Debug)]
    pub enum EcdsaSignError {
        MessageHashOutOfRange,
    }

    #[derive(Debug)]
    pub enum EcdsaVerifyError {
        MessageHashOutOfRange,
        InvalidPublicKey,
        SignatureROutOfRange,
        SignatureSOutOfRange,
    }

    #[cfg(feature = "std")]
    impl std::error::Error for EcdsaSignError {}

    impl Display for EcdsaSignError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::MessageHashOutOfRange => write!(f, "message hash out of range"),
            }
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for EcdsaVerifyError {}

    impl Display for EcdsaVerifyError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::MessageHashOutOfRange => write!(f, "message hash out of range"),
                Self::InvalidPublicKey => write!(f, "invalid public key"),
                Self::SignatureROutOfRange => write!(f, "signature r value out of range"),
                Self::SignatureSOutOfRange => write!(f, "signature s value out of range"),
            }
        }
    }
}
pub use errors::{EcdsaSignError, EcdsaVerifyError};

pub fn compute_hash_on_elements(data: &[FieldElement]) -> FieldElement {
    let mut current_hash = FieldElement::ZERO;

    for item in data.iter() {
        current_hash = pedersen_hash(&current_hash, item);
    }

    let data_len = FieldElement::from(data.len());
    pedersen_hash(&current_hash, &data_len)
}

pub fn ecdsa_sign(
    private_key: &FieldElement,
    message_hash: &FieldElement,
) -> Result<ExtendedSignature, EcdsaSignError> {
    // Seed-retry logic ported from `cairo-lang`
    let mut seed = None;
    loop {
        let k = rfc6979_generate_k(message_hash, private_key, seed.as_ref());

        match sign(private_key, message_hash, &k) {
            Ok(sig) => {
                return Ok(sig);
            }
            Err(SignError::InvalidMessageHash) => {
                return Err(EcdsaSignError::MessageHashOutOfRange)
            }
            Err(SignError::InvalidK) => {
                // Bump seed and retry
                seed = match seed {
                    Some(prev_seed) => Some(prev_seed + FieldElement::ONE),
                    None => Some(FieldElement::ONE),
                };
            }
        };
    }
}

pub fn ecdsa_verify(
    public_key: &FieldElement,
    message_hash: &FieldElement,
    signature: &Signature,
) -> Result<bool, EcdsaVerifyError> {
    match verify(public_key, message_hash, &signature.r, &signature.s) {
        Ok(result) => Ok(result),
        Err(VerifyError::InvalidMessageHash) => Err(EcdsaVerifyError::MessageHashOutOfRange),
        Err(VerifyError::InvalidPublicKey) => Err(EcdsaVerifyError::InvalidPublicKey),
        Err(VerifyError::InvalidR) => Err(EcdsaVerifyError::SignatureROutOfRange),
        Err(VerifyError::InvalidS) => Err(EcdsaVerifyError::SignatureSOutOfRange),
    }
}
