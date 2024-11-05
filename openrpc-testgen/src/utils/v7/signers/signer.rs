use auto_impl::auto_impl;
use crypto_utils::curve::signer::Signature;
use starknet_types_core::felt::Felt;

use std::error::Error;

use crate::utils::v7::signers::key_pair::VerifyingKey;

#[auto_impl(&, Box, Arc)]
pub trait Signer {
    type GetPublicKeyError: Error + Send + Sync;
    type SignError: Error + Send + Sync;

    fn get_public_key(
        &self,
    ) -> impl std::future::Future<Output = Result<VerifyingKey, Self::GetPublicKeyError>> + Send;

    fn sign_hash(
        &self,
        hash: &Felt,
    ) -> impl std::future::Future<Output = Result<Signature, Self::SignError>> + Send;

    /// Whether the underlying signer implementation is interactive, such as a hardware wallet.
    /// Implementations should return `true` if the signing operation is very expensive, even if not
    /// strictly "interactive" as in requiring human input.
    ///
    /// This mainly affects the transaction simulation strategy used by higher-level types. With
    /// non-interactive signers, it's fine to sign multiple times for getting the most accurate
    /// estimation/simulation possible; but with interactive signers, they would accept less
    /// accurate results to minimize signing requests.
    fn is_interactive(&self) -> bool;
}
