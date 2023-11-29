use parachain_template_runtime::genesis::*;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<(), Extensions>;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

// type AccountPublic = <Signature as Verify>::Signer;

// /// Generate collator keys from seed.
// ///
// /// This function's return type must always match the session keys of the chain in tuple format.
// pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
// 	get_from_seed::<AuraId>(seed)
// }

// /// Helper function to generate an account ID from seed
// pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
// where
// 	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
// {
// 	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
// }

// /// Generate the session keys from individual elements.
// ///
// /// The input must be a tuple of individual keys (a single arg for now since we have just one key).
// pub fn template_session_keys(keys: AuraId) -> parachain_template_runtime::SessionKeys {
// 	parachain_template_runtime::SessionKeys { aura: keys }
// }

pub fn development_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::builder(
        WASM_BINARY.expect("Development wasm not available"),
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            // CAUTION: This value is dupliocated in the runtime code. The value here must match, or...
            // This duplication is also present in the upstream template, but will hopefully be fixed as soon as SDK 1.4.0
            // github.com/paritytech/polkadot-sdk/blob/838a534d/cumulus/parachain-template/node/src/chain_spec.rs#L76-L109
            para_id: 2000,
        },
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(development_genesis_config())
    .build()
}

pub fn local_testnet_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::builder(
        WASM_BINARY.expect("Development wasm not available"),
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: 2000,
        },
    )
    .with_name("Local Testnet")
    .with_id("local_testnet")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_patch(development_genesis_config())
    .with_protocol_id("template-local")
    .with_properties(properties)
    .build()
}
