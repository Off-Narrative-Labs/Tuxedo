//! Helper module to build a genesis configuration for the template runtime.

use super::{OuterVerifier, ParachainConstraintChecker, WASM_BINARY};
use hex_literal::hex;
use inner_runtime::{money::Coin, OuterConstraintChecker as InnerConstraintChecker};
use tuxedo_parachain_core::tuxedo_core::{
    genesis::TuxedoGenesisConfig,
    verifier::{Sr25519Signature, ThresholdMultiSignature},
    ConstraintChecker,
};

/// Helper type for the ChainSpec.
pub type RuntimeGenesisConfig = TuxedoGenesisConfig<OuterVerifier, ParachainConstraintChecker>;

const SHAWN_PUB_KEY_BYTES: [u8; 32] =
    hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
const ANDREW_PUB_KEY_BYTES: [u8; 32] =
    hex!("baa81e58b1b4d053c2e86d93045765036f9d265c7dfe8b9693bbc2c0f048d93a");

pub fn development_genesis_config() -> RuntimeGenesisConfig {
    let signatories = vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()];

    let user_genesis_transactions = [
        // Money Transactions
        Coin::<0>::mint::<_, _, InnerConstraintChecker>(
            100,
            Sr25519Signature::new(SHAWN_PUB_KEY_BYTES),
        )
        .transform(),
        Coin::<0>::mint::<_, _, InnerConstraintChecker>(
            100,
            ThresholdMultiSignature::new(1, signatories),
        )
        .transform(),
        // No Kitty or anything else in this one. Keep it simple.
    ]
    .into_iter()
    .map(Into::into);

    // The inherents are computed using the appropriate method, and placed before the user transactions.
    // Ideally this will get better upstream eventually.
    let mut genesis_transactions = ParachainConstraintChecker::genesis_transactions();
    genesis_transactions.extend(user_genesis_transactions);

    RuntimeGenesisConfig::new(
        WASM_BINARY
            .expect("Runtime WASM binary must exist.")
            .to_vec(),
        genesis_transactions,
    )
}
