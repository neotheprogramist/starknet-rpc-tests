#[macro_export]
macro_rules! assert_no_panic {
    ($cond:expr) => {{
        if $cond {
            Ok(())
        } else {
            Err(
                $crate::macros::errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    "Assertion failed".to_string(),
                ),
            )
        }
    }};
    ($cond:expr, $msg:expr) => {{
        if $cond {
            Ok(())
        } else {
            Err(
                $crate::macros::errors::AssertionNoPanicError::AssertionNoPanicFailed(
                    $msg.to_string(),
                ),
            )
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::macros::errors::AssertionNoPanicError;

    #[test]
    fn test_assert_no_panic_success() {
        let result = assert_no_panic!(1 + 1 == 2);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[test]
    fn test_assert_no_panic_failure_default_message() {
        let result = assert_no_panic!(1 + 1 == 3);
        assert!(
            matches!(result, Err(AssertionNoPanicError::AssertionNoPanicFailed(ref msg)) if msg == "Assertion failed"),
            "Expected AssertionNoPanicFailed with default message, got {:?}",
            result
        );
    }

    #[test]
    fn test_assert_no_panic_failure_custom_message() {
        let custom_message = "Custom error message";
        let result = assert_no_panic!(1 + 1 == 3, custom_message);
        assert!(
            matches!(result, Err(AssertionNoPanicError::AssertionNoPanicFailed(ref msg)) if msg == custom_message),
            "Expected AssertionNoPanicFailed with custom message, got {:?}",
            result
        );
    }
}
