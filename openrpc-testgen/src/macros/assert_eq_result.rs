#[macro_export]
macro_rules! assert_eq_result {
    ($left:expr, $right:expr $(,)?) => {
        {
            let (left, right) = (&$left, &$right);
            if *left == *right {
            } else {
                Err($crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(format!(
                    "assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`",
                    left, right
                )))?
            }
        }
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        {
            let (left, right) = (&$left, &$right);
            if *left == *right {
            } else {
                Err($crate::macros::macros_errors::AssertionNoPanicError::AssertionNoPanicFailed(format!(
                    "assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                    left, right, format_args!($($arg)+))
                ))?
            }
        }
    };
}
