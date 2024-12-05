/// Assert that the given error is a Starknet error from a
/// [`AccountError`](starknet::accounts::AccountError).
#[macro_export]
macro_rules! assert_account_starknet_err {
    ($err:expr, $api_err:pat) => {
        assert_matches!(
            $err,
            AccountError::Provider(ProviderError::StarknetError($api_err))
        )
    };
}

#[macro_export]
macro_rules! assert_matches_result {
    ($e:expr, $($pat:pat_param)|+) => {
        match $e {
            $($pat)|+ => (),
            ref e => Err(
                $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    format!(
                        "assertion failed: `{:?}` does not match `{}`",
                        e, stringify!($($pat)|+)
                    )
                )
            )?,
        }
    };

    ($e:expr, $($pat:pat_param)|+ if $cond:expr) => {
        match $e {
            $($pat)|+ if $cond => (),
            ref e => Err(
                $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    format!(
                        "assertion failed: `{:?}` does not match `{}`",
                        e, stringify!($($pat)|+ if $cond)
                    )
                )
            )?,
        }
    };

    ($e:expr, $($pat:pat_param)|+ , $($arg:tt)*) => {
        match $e {
            $($pat)|+ => (),
            ref e => Err(
                $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    format!(
                        "assertion failed: `{:?}` does not match `{}`: {}",
                        e, stringify!($($pat)|+), format_args!($($arg)*)
                    )
                )
            )?,
        }
    };

    ($e:expr, $($pat:pat_param)|+ if $cond:expr , $($arg:tt)*) => {
        match $e {
            $($pat)|+ if $cond => (),
            ref e => Err(
                $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    format!(
                        "assertion failed: `{:?}` does not match `{}`: {}",
                        e, stringify!($($pat)|+ if $cond), format_args!($($arg)*)
                    )
                )
            )?,
        }
    };
    ($e:expr, $($pat:pat_param)|+ => $block:block) => {
        match $e {
            $($pat)|+ => {
                $block
            },
            ref e => {
                Err(
                    $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                        format!(
                            "assertion failed: `{:?}` does not match `{}`",
                            e, stringify!($($pat)|+)
                        )
                    )
                )?
            }
        }
    };
    ($e:expr, $($pat:pat_param)|+ if $cond:expr => $block:block) => {
        match $e {
            $($pat)|+ if $cond => {
                $block
            },
            ref e => {
                Err(
                    $crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(
                        format!(
                            "assertion failed: `{:?}` does not match `{}` when condition `{}` is true",
                            e, stringify!($($pat)|+), stringify!($cond)
                        )
                    )
                )?
            }
        }
    };
}
