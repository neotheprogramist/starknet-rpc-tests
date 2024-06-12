use prefix_hex;
use serde::Serializer;
use starknet_crypto::FieldElement;

pub fn serialize_field_element<S>(value: &FieldElement, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes = value.to_bytes_be();
    let hex_string = prefix_hex::encode(bytes);
    serializer.serialize_str(&hex_string)
}
