use node_template_runtime::{
    money::Coin, GenesisConfig, OuterVerifier, Output, SigCheck, ThresholdMultiSignature,
};
use sc_service::ChainType;
// use tuxedo_core::verifier::{SigCheck, ThresholdMultisignature};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        testnet_genesis,
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
        testnet_genesis,
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

//TODO Consider moving this function to the runtime.
// I originally put it here because this is where it lives in the node template.
// But it uses a lot of concrete types from the runtime
// and also from tuxedo core which was not previously depended on by the node
fn testnet_genesis() -> GenesisConfig {
    use hex_literal::hex;

    const SHAWN_PUB_KEY_BYTES: [u8; 32] =
        hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");
    const ANDREW_PUB_KEY_BYTES: [u8; 32] =
        hex!("baa81e58b1b4d053c2e86d93045765036f9d265c7dfe8b9693bbc2c0f048d93a");

    // Initial Config just for a Money UTXO
    GenesisConfig {
        genesis_utxos: vec![
            Output {
                verifier: OuterVerifier::SigCheck(SigCheck {
                    owner_pubkey: SHAWN_PUB_KEY_BYTES.into(),
                }),
                payload: Coin::<0>(100).into(),
            },
            Output {
                verifier: OuterVerifier::ThresholdMultiSignature(ThresholdMultiSignature {
                    threshold: 1,
                    signatories: vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()],
                }),
                payload: Coin::<0>(100).into(),
            },
        ],
    }
}
