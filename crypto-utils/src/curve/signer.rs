use core::fmt::{Display, Formatter, Result as CoreResult};
use core::ops::{Add, Mul};
use crypto_bigint::{ArrayEncoding, ByteArray, Integer as CryptoInteger, U256};
use hmac::digest::Digest;
use lambdaworks_math::elliptic_curve::short_weierstrass::curves::stark_curve::StarkCurve;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Zero};
use sha2::digest::{crypto_common::BlockSizeUser, FixedOutputReset, HashMarker};
use starknet_types_core::curve::{AffinePoint, ProjectivePoint};
use starknet_types_core::felt::Felt;
use starknet_types_core::hash::{Pedersen, StarkHash};
use std::error::Error as StdError;
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

const EC_ORDER: Felt = Felt::from_raw([
    369010039416812937,
    9,
    1143265896874747514,
    8939893405601011193,
]);

const EC_ORDER_2: U256 =
    U256::from_be_hex("0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f");

const ELEMENT_UPPER_BOUND: Felt = Felt::from_raw([
    576459263475450960,
    18446744073709255680,
    160989183,
    18446743986131435553,
]);

const GENERATOR: AffinePoint = AffinePoint::new_unchecked(
    Felt::from_raw([
        232005955912912577,
        299981207024966779,
        5884444832209845738,
        14484022957141291997,
    ]),
    Felt::from_raw([
        405578048423154473,
        18147424675297964973,
        664812301889158119,
        6241159653446987914,
    ]),
);

pub const ALPHA: Felt = Felt::from_raw([
    576460752303422960,
    18446744073709551615,
    18446744073709551615,
    18446744073709551585,
]);

pub const BETA: Felt = Felt::from_raw([
    88155977965380735,
    12360725113329547591,
    7432612994240712710,
    3863487492851900874,
]);

pub trait Signer {
    fn ecdsa_sign(
        private_key: &Felt,
        message_hash: &Felt,
    ) -> Result<ExtendedSignature, EcdsaSignError>;
}

impl Signer for StarkCurve {
    fn ecdsa_sign(
        private_key: &Felt,
        message_hash: &Felt,
    ) -> Result<ExtendedSignature, EcdsaSignError> {
        let mut seed = None;
        loop {
            let k = generate_k(private_key, message_hash, seed.as_ref());

            match sign(private_key, message_hash, &k) {
                Ok(sig) => {
                    return Ok(sig);
                }
                Err(SignError::InvalidMessageHash) => {
                    return Err(EcdsaSignError::MessageHashOutOfRange)
                }
                Err(SignError::InvalidK) => {
                    // Bump seed and retry
                    seed = match seed {
                        Some(prev_seed) => Some(prev_seed + Felt::ONE),
                        None => Some(Felt::ONE),
                    };
                }
            };
        }
    }
}

#[derive(Debug)]
pub enum EcdsaSignError {
    MessageHashOutOfRange,
}

#[cfg(feature = "std")]
impl std::error::Error for EcdsaSignError {}

impl Display for EcdsaSignError {
    fn fmt(&self, f: &mut Formatter<'_>) -> CoreResult {
        match self {
            Self::MessageHashOutOfRange => write!(f, "message hash out of range"),
        }
    }
}

/// Stark ECDSA signature
#[derive(Debug)]
pub struct Signature {
    /// The `r` value of a signature
    pub r: Felt,
    /// The `s` value of a signature
    pub s: Felt,
}
/// Stark ECDSA signature with `v`
#[derive(Debug)]
pub struct ExtendedSignature {
    /// The `r` value of a signature
    pub r: Felt,
    /// The `s` value of a signature
    pub s: Felt,
    /// The `v` value of a signature
    pub v: Felt,
}

impl From<ExtendedSignature> for Signature {
    fn from(value: ExtendedSignature) -> Self {
        Self {
            r: value.r,
            s: value.s,
        }
    }
}

#[derive(Debug)]
pub enum SignError {
    InvalidMessageHash,
    InvalidK,
}

#[derive(Debug)]
pub enum VerifyError {
    /// The public key is not a valid point on the STARK curve.
    InvalidPublicKey,
    /// The message hash is not in the range of `[0, 2^251)`.
    InvalidMessageHash,
    /// The `r` value is not in the range of `[0, 2^251)`.
    InvalidR,
    /// The `s` value is not in the range of `[0, 2^251)`.
    InvalidS,
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerifyError::InvalidPublicKey => {
                write!(f, "The public key is not a valid point on the STARK curve.")
            }
            VerifyError::InvalidMessageHash => {
                write!(f, "The message hash is not in the valid range [0, 2^251).")
            }
            VerifyError::InvalidR => {
                write!(f, "The 'r' value is not in the valid range [0, 2^251).")
            }
            VerifyError::InvalidS => {
                write!(f, "The 's' value is not in the valid range [0, 2^251).")
            }
        }
    }
}

impl StdError for VerifyError {}

#[derive(Debug)]
pub enum RecoverError {
    /// The message hash is not in the range of `[0, 2^251)`.
    InvalidMessageHash,
    /// The `r` value is not in the range of `[0, 2^251)`.
    InvalidR,
    /// The `s` value is not in the range of `[0,
    /// 0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f)`.
    InvalidS,
    /// The `v` value is neither `0` nor `1`.
    InvalidV,
}

impl fmt::Display for RecoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoverError::InvalidMessageHash => {
                write!(f, "The message hash is not in the valid range [0, 2^251).")
            }
            RecoverError::InvalidR => {
                write!(f, "The 'r' value is not in the valid range [0, 2^251).")
            }
            RecoverError::InvalidS => {
                write!(
                    f,
                    "The 's' value is not in the valid range [0, \
                     0x0800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f)."
                )
            }
            RecoverError::InvalidV => {
                write!(f, "The 'v' value is neither '0' nor '1'.")
            }
        }
    }
}

impl StdError for RecoverError {}

pub fn compute_hash_on_elements<'a, ESI, II>(data: II) -> Felt
where
    ESI: ExactSizeIterator<Item = &'a Felt>,
    II: IntoIterator<IntoIter = ESI>,
{
    let mut current_hash = Felt::ZERO;
    let data_iter = data.into_iter();
    let data_len = Felt::from(data_iter.len());

    for elem in data_iter {
        current_hash = Pedersen::hash(&current_hash, elem);
    }

    Pedersen::hash(&current_hash, &data_len)
}

/// Computes the public key given a Stark private key.
///
/// ### Arguments
///
/// * `private_key`: The private key
pub fn get_public_key(private_key: &Felt) -> Felt {
    mul_by_bits(&GENERATOR, private_key)
        .to_affine()
        .unwrap()
        .x()
}

/// Computes ECDSA signature given a Stark private key and message hash.
///
/// ### Arguments
///
/// * `private_key`: The private key
/// * `message`: The message hash
/// * `k`: A random `k` value. You **MUST NOT** use the same `k` on different signatures
pub fn sign(private_key: &Felt, message: &Felt, k: &Felt) -> Result<ExtendedSignature, SignError> {
    if message >= &ELEMENT_UPPER_BOUND {
        return Err(SignError::InvalidMessageHash);
    }
    if k == &Felt::ZERO {
        return Err(SignError::InvalidK);
    }

    let full_r = mul_by_bits(&GENERATOR, k).to_affine().unwrap();
    let r = full_r.x();
    if r == Felt::ZERO || r >= ELEMENT_UPPER_BOUND {
        return Err(SignError::InvalidK);
    }

    let k_inv = mod_inverse(k, &EC_ORDER);

    let s = mul_mod_floor(&r, private_key, &EC_ORDER);
    let s = add_unbounded(&s, message);
    let s = bigint_mul_mod_floor(s, &k_inv, &EC_ORDER);
    if s == Felt::ZERO || s >= ELEMENT_UPPER_BOUND {
        return Err(SignError::InvalidK);
    }

    Ok(ExtendedSignature {
        r,
        s,
        v: (full_r.y().to_bigint() & Felt::ONE.to_bigint()).into(),
    })
}

pub fn add_unbounded(augend: &Felt, addend: &Felt) -> BigInt {
    let augend = BigInt::from_bytes_be(num_bigint::Sign::Plus, &augend.to_bytes_be());
    let addend = BigInt::from_bytes_be(num_bigint::Sign::Plus, &addend.to_bytes_be());
    augend.add(addend)
}

pub fn mul_mod_floor(multiplicand: &Felt, multiplier: &Felt, modulus: &Felt) -> Felt {
    let multiplicand = BigInt::from_bytes_be(num_bigint::Sign::Plus, &multiplicand.to_bytes_be());
    bigint_mul_mod_floor(multiplicand, multiplier, modulus)
}

pub fn bigint_mul_mod_floor(multiplicand: BigInt, multiplier: &Felt, modulus: &Felt) -> Felt {
    let multiplier = BigInt::from_bytes_be(num_bigint::Sign::Plus, &multiplier.to_bytes_be());
    let modulus = BigInt::from_bytes_be(num_bigint::Sign::Plus, &modulus.to_bytes_be());

    let result = multiplicand.mul(multiplier).mod_floor(&modulus);

    let (_, buffer) = result.to_bytes_be();
    let mut result = [0u8; 32];
    result[(32 - buffer.len())..].copy_from_slice(&buffer[..]);

    Felt::from_bytes_be(&result)
}

#[inline(always)]
fn mul_by_bits(x: &AffinePoint, y: &Felt) -> ProjectivePoint {
    &ProjectivePoint::from_affine(x.x(), x.y()).unwrap() * *y
}

pub fn mod_inverse(operand: &Felt, modulus: &Felt) -> Felt {
    let operand = BigInt::from_bytes_be(num_bigint::Sign::Plus, &operand.to_bytes_be());
    let modulus = BigInt::from_bytes_be(num_bigint::Sign::Plus, &modulus.to_bytes_be());

    // Ported from:
    //   https://github.com/dignifiedquire/num-bigint/blob/56576b592fea6341b7e1711a1629e4cc1bfc419c/src/algorithms/mod_inverse.rs#L11
    let extended_gcd = operand.extended_gcd(&modulus);
    if extended_gcd.gcd != BigInt::one() {
        panic!("GCD must be one");
    }
    let result = if extended_gcd.x < BigInt::zero() {
        extended_gcd.x + modulus
    } else {
        extended_gcd.x
    };

    let (_, buffer) = result.to_bytes_be();
    let mut result = [0u8; 32];
    result[(32 - buffer.len())..].copy_from_slice(&buffer[..]);

    Felt::from_bytes_be(&result)
}

/// Deterministically generate ephemeral scalar `k` based on RFC 6979.
fn generate_k(private_key: &Felt, message_hash: &Felt, seed: Option<&Felt>) -> Felt {
    let message_hash = U256::from_be_slice(&message_hash.to_bytes_be()).to_be_byte_array();
    let private_key = U256::from_be_slice(&private_key.to_bytes_be());

    let seed_bytes = match seed {
        Some(seed) => seed.to_bytes_be(),
        None => [0u8; 32],
    };

    let mut first_non_zero_index = 32;
    for (ind, element) in seed_bytes.iter().enumerate() {
        if *element != 0u8 {
            first_non_zero_index = ind;
            break;
        }
    }

    let k = generate_k_shifted::<sha2::Sha256, _>(
        &private_key,
        &EC_ORDER_2,
        &message_hash,
        &seed_bytes[first_non_zero_index..],
    );

    let mut buffer = [0u8; 32];
    buffer[..].copy_from_slice(&k.to_be_byte_array()[..]);

    Felt::from_bytes_be(&buffer)
}

// Modified from upstream `rfc6979::generate_k` with a hard-coded right bit shift. The more
// idiomatic way of doing this seems to be to implement `U252` which handles bit truncation
// interally.
// TODO: change to use upstream `generate_k` directly.
#[inline]
fn generate_k_shifted<D, I>(x: &I, n: &I, h: &ByteArray<I>, data: &[u8]) -> Zeroizing<I>
where
    D: Default + Digest + BlockSizeUser + FixedOutputReset + HashMarker,
    I: ArrayEncoding + CryptoInteger + Zeroize,
{
    let mut x = x.to_be_byte_array();
    let mut hmac_drbg = rfc6979::HmacDrbg::<D>::new(&x, h, data);
    x.zeroize();

    loop {
        let mut bytes = ByteArray::<I>::default();
        hmac_drbg.fill_bytes(&mut bytes);
        let k = I::from_be_byte_array(bytes) >> 4;

        if (!k.is_zero() & k.ct_lt(n)).into() {
            return Zeroizing::new(k);
        }
    }
}

pub fn verify(public_key: &Felt, message: &Felt, r: &Felt, s: &Felt) -> Result<bool, VerifyError> {
    if message >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidMessageHash);
    }
    if r == &Felt::ZERO || r >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidR);
    }
    if s == &Felt::ZERO || s >= &ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidS);
    }

    let full_public_key = AffinePoint::new(
        *public_key,
        (public_key.square() * public_key + ALPHA * public_key + BETA)
            .sqrt()
            .ok_or(VerifyError::InvalidPublicKey)?,
    )
    .unwrap();

    let w = mod_inverse(s, &EC_ORDER);
    if w == Felt::ZERO || w >= ELEMENT_UPPER_BOUND {
        return Err(VerifyError::InvalidS);
    }

    let zw = mul_mod_floor(message, &w, &EC_ORDER);
    let zw_g = mul_by_bits(&GENERATOR, &zw);

    let rw = mul_mod_floor(r, &w, &EC_ORDER);
    let rw_q = mul_by_bits(&full_public_key, &rw);

    Ok((&zw_g + &rw_q).to_affine().unwrap().x() == *r
        || (&zw_g - &rw_q).to_affine().unwrap().x() == *r)
}

pub fn recover(message: &Felt, r: &Felt, s: &Felt, v: &Felt) -> Result<Felt, RecoverError> {
    if message >= &ELEMENT_UPPER_BOUND {
        return Err(RecoverError::InvalidMessageHash);
    }
    if r == &Felt::ZERO || r >= &ELEMENT_UPPER_BOUND {
        return Err(RecoverError::InvalidR);
    }
    if s == &Felt::ZERO || s >= &EC_ORDER {
        return Err(RecoverError::InvalidS);
    }
    if v > &Felt::ONE {
        return Err(RecoverError::InvalidV);
    }

    let full_r = AffinePoint::new(
        *r,
        (r * r * r + ALPHA * r + BETA)
            .sqrt()
            .ok_or(RecoverError::InvalidR)?,
    )
    .unwrap();

    let mut full_r_y = full_r.y();

    let mut bits = [false; 256];

    for (i, (&a, &b)) in full_r
        .y()
        .to_bits_le()
        .iter()
        .zip(Felt::ONE.to_bits_le().iter())
        .enumerate()
    {
        bits[i] = a && b;
    }

    if bits != v.to_bits_le() {
        full_r_y = -full_r.y();
    }

    let full_rs = mul_by_bits(&AffinePoint::new(full_r.x(), full_r_y).unwrap(), s);
    let zg = mul_by_bits(&GENERATOR, message);

    let r_inv = mod_inverse(r, &EC_ORDER);

    let rs_zg = &full_rs - &zg;

    let k = mul_by_bits(&rs_zg.to_affine().unwrap(), &r_inv);

    Ok(k.to_affine().unwrap().x())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test cases ported from:
    //   https://github.com/starkware-libs/crypto-cpp/blob/95864fbe11d5287e345432dbe1e80dea3c35fc58/src/starkware/crypto/ffi/crypto_lib_test.go

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_public_key_1() {
        let private_key = Felt::from_hex_unchecked(
            "03c1e9550e66958296d11b60f8e8e7a7ad990d07fa65d5f7652c4a6c87d4e3cc",
        );
        let expected_key = Felt::from_hex_unchecked(
            "077a3b314db07c45076d11f62b6f9e748a39790441823307743cf00d6597ea43",
        );

        assert_eq!(get_public_key(&private_key), expected_key);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_public_key_2() {
        let private_key = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000012",
        );
        let expected_key = Felt::from_hex_unchecked(
            "019661066e96a8b9f06a1d136881ee924dfb6a885239caa5fd3f87a54c6b25c4",
        );

        assert_eq!(get_public_key(&private_key), expected_key);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_verify_valid_message() {
        let stark_key = Felt::from_hex_unchecked(
            "01ef15c18599971b7beced415a40f0c7deacfd9b0d1819e03d723d8bc943cfca",
        );
        let msg_hash = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000002",
        );
        let r_bytes = Felt::from_hex_unchecked(
            "0411494b501a98abd8262b0da1351e17899a0c4ef23dd2f96fec5ba847310b20",
        );
        let s_bytes = Felt::from_hex_unchecked(
            "0405c3191ab3883ef2b763af35bc5f5d15b3b4e99461d70e84c654a351a7c81b",
        );

        assert!(verify(&stark_key, &msg_hash, &r_bytes, &s_bytes).unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_verify_invalid_message() {
        let stark_key = Felt::from_hex_unchecked(
            "077a4b314db07c45076d11f62b6f9e748a39790441823307743cf00d6597ea43",
        );
        let msg_hash = Felt::from_hex_unchecked(
            "0397e76d1667c4454bfb83514e120583af836f8e32a516765497823eabe16a3f",
        );
        let r_bytes = Felt::from_hex_unchecked(
            "0173fd03d8b008ee7432977ac27d1e9d1a1f6c98b1a2f05fa84a21c84c44e882",
        );
        let s_bytes = Felt::from_hex_unchecked(
            "01f2c44a7798f55192f153b4c48ea5c1241fbb69e6132cc8a0da9c5b62a4286e",
        );

        assert!(!verify(&stark_key, &msg_hash, &r_bytes, &s_bytes).unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_verify_invalid_public_key() {
        let stark_key = Felt::from_hex_unchecked(
            "03ee9bffffffffff26ffffffff60ffffffffffffffffffffffffffff004accff",
        );
        let msg_hash = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000002",
        );
        let r_bytes = Felt::from_hex_unchecked(
            "0411494b501a98abd8262b0da1351e17899a0c4ef23dd2f96fec5ba847310b20",
        );
        let s_bytes = Felt::from_hex_unchecked(
            "0405c3191ab3883ef2b763af35bc5f5d15b3b4e99461d70e84c654a351a7c81b",
        );

        match verify(&stark_key, &msg_hash, &r_bytes, &s_bytes) {
            Err(VerifyError::InvalidPublicKey) => {}
            _ => panic!("unexpected result"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_sign() {
        let private_key = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        let message = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000002",
        );
        let k = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000003",
        );

        let signature = sign(&private_key, &message, &k).unwrap();
        let public_key = get_public_key(&private_key);

        assert!(verify(&public_key, &message, &signature.r, &signature.s).unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_recover() {
        let private_key = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000001",
        );
        let message = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000002",
        );
        let k = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000003",
        );

        let signature = sign(&private_key, &message, &k).unwrap();
        let public_key = recover(&message, &signature.r, &signature.s, &signature.v).unwrap();

        assert_eq!(get_public_key(&private_key), public_key);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_recover_invalid_r() {
        let message = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000002",
        );
        let r = Felt::from_hex_unchecked(
            "03ee9bffffffffff26ffffffff60ffffffffffffffffffffffffffff004accff",
        );
        let s = Felt::from_hex_unchecked(
            "0405c3191ab3883ef2b763af35bc5f5d15b3b4e99461d70e84c654a351a7c81b",
        );
        let v = Felt::from_hex_unchecked(
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        match recover(&message, &r, &s, &v) {
            Err(RecoverError::InvalidR) => {}
            _ => panic!("unexpected result"),
        }
    }
}
