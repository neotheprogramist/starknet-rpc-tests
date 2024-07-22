use crate::v5::rpc::providers::provider::Provider;
use crate::v5::rpc::signers::signer::Signer;

use starknet_types_rpc::{BlockId, BlockTag, Felt};
use tracing::info;

use super::{AccountFactory, PreparedAccountDeploymentV1, RawAccountDeploymentV1};

pub struct OpenZeppelinAccountFactory<S, P> {
    class_hash: Felt,
    chain_id: Felt,
    public_key: Felt,
    signer: S,
    provider: P,
    block_id: BlockId,
}

impl<S, P> OpenZeppelinAccountFactory<S, P>
where
    S: Signer,
{
    pub async fn new(
        class_hash: Felt,
        chain_id: Felt,
        signer: S,
        provider: P,
    ) -> Result<Self, S::GetPublicKeyError> {
        let public_key = signer.get_public_key().await?;
        Ok(Self {
            class_hash,
            chain_id,
            public_key: public_key.scalar(),
            signer,
            provider,
            block_id: BlockId::Tag(BlockTag::Latest),
        })
    }

    pub fn set_block_id(&mut self, block_id: BlockId) -> &Self {
        self.block_id = block_id;
        self
    }
}

impl<S, P> AccountFactory for OpenZeppelinAccountFactory<S, P>
where
    S: Signer + Sync + Send,
    P: Provider + Sync + Send,
{
    type Provider = P;
    type SignError = S::SignError;

    fn class_hash(&self) -> Felt {
        self.class_hash
    }

    fn calldata(&self) -> Vec<Felt> {
        vec![self.public_key]
    }

    fn chain_id(&self) -> Felt {
        self.chain_id
    }

    fn provider(&self) -> &Self::Provider {
        &self.provider
    }

    fn is_signer_interactive(&self) -> bool {
        self.signer.is_interactive()
    }

    fn block_id(&self) -> BlockId {
        self.block_id
    }

    async fn sign_deployment_v1(
        &self,
        deployment: &RawAccountDeploymentV1,
        query_only: bool,
    ) -> Result<Vec<Felt>, Self::SignError> {
        info!("sign deployment v1 start, counting tx hash ");
        let tx_hash = PreparedAccountDeploymentV1::from_raw(deployment.clone(), self)
            .transaction_hash(query_only);
        info!("tx hash: {}", tx_hash);
        info!("starting signature");
        let signature = self.signer.sign_hash(&tx_hash).await?;
        info!("signature {:?}", signature);

        Ok(vec![signature.r, signature.s])
    }

    // async fn sign_deployment_v3(
    //     &self,
    //     deployment: &RawAccountDeploymentV3,
    //     query_only: bool,
    // ) -> Result<Vec<Felt>, Self::SignError> {
    //     let tx_hash = PreparedAccountDeploymentV3::from_raw(deployment.clone(), self)
    //         .transaction_hash(query_only);
    //     let signature = self.signer.sign_hash(&tx_hash).await?;

    //     Ok(vec![signature.r, signature.s])
    // }
}
