use super::{
    key_pair::{SigningKey, VerifyingKey},
    signer::Signer,
};
use starknet_types_core::curve::{EcdsaSignError, Signature};
use starknet_types_rpc::Felt;
use std::fmt;

#[derive(Debug, Clone)]
pub struct LocalWallet {
    private_key: SigningKey,
}

#[derive(Debug, thiserror::Error)]
pub enum SignError {
    EcdsaSignError(EcdsaSignError),
}

impl fmt::Display for SignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignError::EcdsaSignError(e) => write!(f, "ECDSA signing error: {}", e),
        }
    }
}

impl LocalWallet {
    pub fn from_signing_key(key: SigningKey) -> Self {
        key.into()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Infallible {}

impl Signer for LocalWallet {
    type GetPublicKeyError = Infallible;
    type SignError = SignError;

    async fn get_public_key(&self) -> Result<VerifyingKey, Self::GetPublicKeyError> {
        Ok(self.private_key.verifying_key())
    }

    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, Self::SignError> {
        Ok(self.private_key.sign(hash)?)
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

impl From<SigningKey> for LocalWallet {
    fn from(value: SigningKey) -> Self {
        Self { private_key: value }
    }
}

impl From<EcdsaSignError> for SignError {
    fn from(value: EcdsaSignError) -> Self {
        Self::EcdsaSignError(value)
    }
}
