/// Assert that the given error is a Starknet error from a
/// [`ProviderError`](starknet::providers::ProviderError).
#[macro_export]
macro_rules! assert_provider_starknet_err {
    ($err:expr, $api_err:pat) => {
        assert_matches!($err, ProviderError::StarknetError($api_err))
    };
}
