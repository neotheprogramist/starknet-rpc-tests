use num_bigint::BigUint;
use starknet_types_core::felt::Felt;

use super::errors::ConversionsError;

/// Combines two Felt values into a single BigUint by treating them as high and low 128-bit parts.
/// The first Felt is shifted left by 128 bits and combined with the second Felt.
const FELT_BITS: u32 = 128;

fn felts_to_biguint(felts: [Felt; 2]) -> BigUint {
    let high = felts[0].to_biguint() << FELT_BITS;
    let low = felts[1].to_biguint();
    high + low
}

pub fn felts_slice_to_biguint(
    felts_slice: impl AsRef<[Felt]>,
) -> Result<BigUint, ConversionsError> {
    let felts_slice = felts_slice.as_ref();
    if felts_slice.len() != 2 {
        return Err(ConversionsError::FeltVecToBigUintError(
            "Felts vector needs to be the size of 2".to_string(),
        ));
    }
    let felts_array: [Felt; 2] = [felts_slice[0], felts_slice[1]];
    Ok(felts_to_biguint(felts_array))
}

#[cfg(test)]
mod test {
    use super::*;
    use num_bigint::BigUint;
    use starknet_types_core::felt::Felt;
    use std::str::FromStr;

    #[test]
    fn test_felts_to_biguint() {
        // Test with non-zero values
        let high = Felt::from_str("123456789").unwrap();
        let low = Felt::from_str("987654321").unwrap();
        let result = felts_to_biguint([high, low]);
        let expected = (BigUint::from_str("123456789").unwrap() << FELT_BITS)
            + BigUint::from_str("987654321").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_felts_to_biguint_zero() {
        // Test with both values zero
        let high = Felt::from(0);
        let low = Felt::from(0);
        let result = felts_to_biguint([high, low]);
        let expected = BigUint::from(0u32);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_felts_slice_to_biguint() {
        // Test with a slice of two elements
        let high = Felt::from_str("123456789").unwrap();
        let low = Felt::from_str("987654321").unwrap();
        let result = felts_slice_to_biguint([high, low].as_ref()).unwrap();
        let expected = (BigUint::from_str("123456789").unwrap() << FELT_BITS)
            + BigUint::from_str("987654321").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_felts_slice_to_biguint_error() {
        // Test with a slice that doesn't have exactly 2 elements
        let single_element = [Felt::from(1)];
        let result = felts_slice_to_biguint(single_element.as_ref());
        assert!(result.is_err());

        if let Err(ConversionsError::FeltVecToBigUintError(message)) = result {
            assert_eq!(message, "Felts vector needs to be the size of 2");
        } else {
            panic!("Expected FeltVecToBigUintError with specific message");
        }

        let three_elements = [Felt::from(1), Felt::from(2), Felt::from(3)];
        let result = felts_slice_to_biguint(three_elements.as_ref());
        assert!(result.is_err());

        if let Err(ConversionsError::FeltVecToBigUintError(message)) = result {
            assert_eq!(message, "Felts vector needs to be the size of 2");
        } else {
            panic!("Expected FeltVecToBigUintError with specific message");
        }
    }
}
