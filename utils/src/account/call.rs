use starknet_crypto::FieldElement;

#[derive(Debug, Clone)]
pub struct Call {
    pub to: FieldElement,
    pub selector: FieldElement,
    pub calldata: Vec<FieldElement>,
}
