use num_bigint::BigUint;
use starknet_types_core::felt::Felt;

use super::errors::ConversionsError;

fn felts_to_biguint(felts: [Felt; 2]) -> BigUint {
    let high = felts[0].to_biguint() << 128;
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
