//! The Tuxedo Template Runtime is an example runtime that uses
//! most of the pieces provided in the wardrobe.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "std")]
pub mod genesis;

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::OpaqueMetadata;
use sp_inherents::InherentData;
use sp_runtime::{
    create_runtime_str,
    traits::Block as BlockT,
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use tuxedo_core::{
    genesis::TuxedoGenesisConfigBuilder, tuxedo_constraint_checker, types::Block as TuxedoBlock,
    types::Transaction as TuxedoTransaction, InherentAdapter,
};
use tuxedo_parachain_core::tuxedo_core;

// We use the same aggregate verifier and opaque types from the inner_runtime.
// They do not contain anything parachain specific.
pub use inner_runtime::{opaque, OuterConstraintChecker as InnerConstraintChecker, OuterVerifier};

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("tuxedo-parachain-runtime"),
    impl_name: create_runtime_str!("tuxedo-parachain-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// The Aura slot duration. When things are working well, this will also be the block time.
const BLOCK_TIME: u64 = 3000;

// This creates an enum `ParachainConstraintChecker` that implements `ParachainConstraintChecker`
tuxedo_parachain_core::parachainify!(OuterVerifier, InnerConstraintChecker, 2000);

pub type Transaction = TuxedoTransaction<OuterVerifier, ParachainConstraintChecker>;
pub type Block = TuxedoBlock<OuterVerifier, ParachainConstraintChecker>;
pub type Executive = tuxedo_core::Executive<OuterVerifier, ParachainConstraintChecker>;

/// The main struct in this module.
#[derive(Encode, Decode, PartialEq, Eq, Clone, TypeInfo)]
pub struct Runtime;

// Here we hard-code consensus authority IDs for the well-known identities that work with the CLI flags
// Such as `--alice`, `--bob`, etc. Only Alice is enabled by default which makes things work nicely
// in a `--dev` node. You may enable more authorities to test more interesting networks, or replace
// these IDs entirely.
impl Runtime {
    /// Aura authority IDs
    fn aura_authorities() -> Vec<AuraId> {
        use hex_literal::hex;
        use sp_application_crypto::ByteArray;

        [
            // Alice
            hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
            // Bob
            // hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
            // Charlie
            // hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"),
            // Dave
            // hex!("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20"),
            // Eve
            // hex!("e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e"),
            // Ferdie
            // hex!("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"),
        ]
        .iter()
        .map(|hex| AuraId::from_slice(hex.as_ref()).expect("Valid Aura authority hex was provided"))
        .collect()
    }
}

impl_runtime_apis! {
    // https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::open_block(header)
        }
    }

    // https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::close_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            Executive::inherent_extrinsics(data)
        }

        fn check_inherents(
            block: Block,
            data: InherentData
        ) -> sp_inherents::CheckInherentsResult {
            Executive::check_inherents(block, data)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    // Tuxedo does not yet support metadata
    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Default::default())
        }

        fn metadata_at_version(_version: u32) -> Option<OpaqueMetadata> {
            None
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Default::default()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(BLOCK_TIME)
        }

        fn authorities() -> Vec<AuraId> {
            Self::aura_authorities()
        }
    }

    //TODO This was copied directly from the sovereign runtime while resolving the update.
    // Make sure it still makes sense here.
    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn create_default_config() -> Vec<u8> {
            serde_json::to_vec(&genesis::development_genesis_transactions())
                .expect("Development genesis transactions are valid.")
        }

        fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
            let genesis_transactions = serde_json::from_slice::<Vec<Transaction>>(config.as_slice())
                .map_err(|_| "The input JSON is not a valid list of Transactions.")?;
            TuxedoGenesisConfigBuilder::build(genesis_transactions)
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            use tuxedo_parachain_core::ParachainExecutiveExtension;
            Executive::collect_collation_info(header)
        }
    }
}
