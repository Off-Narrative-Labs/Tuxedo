//! Helper module to build a genesis configuration for the template runtime.

#[cfg(feature = "std")]
pub use super::WASM_BINARY;
use super::{
    kitties::{KittyData, Parent},
    money::Coin,
    OuterConstraintChecker, Transaction,
};
use hex_literal::hex;
use sp_std::{vec, vec::Vec};
use tuxedo_core::{
    verifier::{Sr25519Signature, ThresholdMultiSignature, UpForGrabs},
    ConstraintChecker,
};

const SHAWN_PUB_KEY_BYTES: [u8; 32] =
    hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
const ANDREW_PUB_KEY_BYTES: [u8; 32] =
    hex!("baa81e58b1b4d053c2e86d93045765036f9d265c7dfe8b9693bbc2c0f048d93a");

/// This function returns a list of valid transactions to be included in the genesis block.
/// It is called by the `ChainSpec::build` method, via the `development_genesis_config` function.
/// The resulting transactions must be ordered: inherent first, then extrinsics.
pub fn development_genesis_transactions() -> Vec<Transaction> {
    let signatories = vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()];

    // The inherents are computed using the appropriate method, and placed before the extrinsics.
    let mut genesis_transactions = OuterConstraintChecker::genesis_transactions();

    genesis_transactions.extend([
        // Money Transactions
        Coin::<0>::mint(100, Sr25519Signature::new(SHAWN_PUB_KEY_BYTES)),
        Coin::<0>::mint(100, ThresholdMultiSignature::new(1, signatories)),
        // Kitty Transactions
        KittyData::mint(Parent::mom(), b"mother", UpForGrabs),
        KittyData::mint(Parent::dad(), b"father", UpForGrabs),
    ]);

    genesis_transactions
}

pub fn development_genesis_config() -> serde_json::Value {
    serde_json::json!(development_genesis_transactions())
}
