#[derive(Drop, Copy, Serde)]
enum SignerSignature {
    Starknet: (StarknetSigner, StarknetSignature),
}

/// @param pubkey the public key as felt252 for a starknet signature. Cannot be zero
#[derive(Drop, Copy, Serde, PartialEq)]
struct StarknetSigner {
    pubkey: felt252,
}

/// @notice The starknet signature using the stark-curve
#[derive(Drop, Copy, Serde, PartialEq)]
struct StarknetSignature {
    r: felt252,
    s: felt252,
}
