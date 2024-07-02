pub mod account;
pub mod account_balance;
pub mod jsonrpc;
pub mod provider;
pub use account::{
    Account, AccountError, ConnectedAccount, DeclarationV2, DeclarationV3, ExecutionEncoder,
    ExecutionV1, ExecutionV3, LegacyDeclaration, PreparedDeclarationV2, PreparedDeclarationV3,
    PreparedExecutionV1, PreparedExecutionV3, PreparedLegacyDeclaration, RawDeclarationV2,
    RawDeclarationV3, RawExecutionV1, RawExecutionV3, RawLegacyDeclaration,
};

mod call;
pub use call::Call;

pub mod single_owner;
pub use single_owner::{ExecutionEncoding, SingleOwnerAccount};

#[derive(Debug, thiserror::Error)]
#[error("Not all fields are prepared")]
pub struct NotPreparedError;

pub mod endpoints;
