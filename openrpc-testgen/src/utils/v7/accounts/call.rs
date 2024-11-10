use cainome_cairo_serde_derive::CairoSerde;
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone, CairoSerde)]
pub struct Call {
    pub to: Felt,
    pub selector: Felt,
    pub calldata: Vec<Felt>,
}
