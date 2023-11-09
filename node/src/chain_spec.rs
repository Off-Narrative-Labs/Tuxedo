use hex_literal::hex;
use node_template_runtime::{
    kitties::{KittyData, Parent},
    money::Coin,
    OuterConstraintChecker, OuterConstraintCheckerInherentHooks, OuterVerifier, WASM_BINARY,
};
use sc_service::ChainType;
use tuxedo_core::{
    genesis::TuxedoGenesisConfig,
    inherents::InherentInternal,
    verifier::{SigCheck, ThresholdMultiSignature, UpForGrabs},
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec =
    sc_service::GenericChainSpec<TuxedoGenesisConfig<OuterVerifier, OuterConstraintChecker>>;

// /// Generate a crypto pair from seed.
// pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
// 	TPublic::Pair::from_string(&format!("//{}", seed), None)
// 		.expect("static values are valid; qed")
// 		.public()
// }

// type AccountPublic = <Signature as Verify>::Signer;

// /// Generate an account ID from seed.
// pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
// where
// 	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
// {
// 	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
// }

// /// Generate an Aura authority key.
// pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
// 	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
// }

const SHAWN_PUB_KEY_BYTES: [u8; 32] =
    hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
const ANDREW_PUB_KEY_BYTES: [u8; 32] =
    hex!("baa81e58b1b4d053c2e86d93045765036f9d265c7dfe8b9693bbc2c0f048d93a");

fn development_genesis_config() -> TuxedoGenesisConfig<OuterVerifier, OuterConstraintChecker> {
    let signatories = vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()];

    // The inherents are computed using the appropriate method, and placed before the extrinsics.
    let mut genesis_transactions = OuterConstraintCheckerInherentHooks::genesis_transactions();

    genesis_transactions.extend([
        // Money Transactions
        Coin::<0>::mint(100, SigCheck::new(SHAWN_PUB_KEY_BYTES)),
        Coin::<0>::mint(100, ThresholdMultiSignature::new(1, signatories)),
        // Kitty Transactions
        KittyData::mint(Parent::mom(), b"mother", UpForGrabs),
        KittyData::mint(Parent::dad(), b"father", UpForGrabs),
        // TODO: Initial Transactions for Existence
    ]);

    TuxedoGenesisConfig::new(
        WASM_BINARY
            .expect("Runtime WASM binary must exist.")
            .to_vec(),
        genesis_transactions,
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        // TuxedoGenesisConfig
        development_genesis_config,
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        // TuxedoGenesisConfig
        development_genesis_config,
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        None,
        // Extensions
        None,
    ))
}
