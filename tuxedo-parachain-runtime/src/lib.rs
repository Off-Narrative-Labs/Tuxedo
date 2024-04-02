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
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;

use sp_api::impl_runtime_apis;
use sp_core::OpaqueMetadata;
use sp_inherents::InherentData;
use sp_runtime::{
    create_runtime_str,
    traits::{BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use tuxedo_core::{
    tuxedo_constraint_checker, types::Transaction as TuxedoTransaction, InherentAdapter,
};
use tuxedo_parachain_core::tuxedo_core;

// We use the same aggregate verifier from the inner_runtime.
// The verifier does not contain anything parachain specific.
use inner_runtime::OuterVerifier;

// Reuse all the same opaque types from the inner runtime.
pub use inner_runtime::opaque;

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

pub type Transaction = TuxedoTransaction<OuterVerifier, OuterConstraintChecker>;
pub type BlockNumber = u32;
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, Transaction>;
pub type Executive = tuxedo_core::Executive<Block, OuterVerifier, OuterConstraintChecker>;
pub type Output = tuxedo_core::types::Output<OuterVerifier>;

impl sp_runtime::traits::GetNodeBlockType for Runtime {
    type NodeBlock = opaque::Block;
}

impl sp_runtime::traits::GetRuntimeBlockType for Runtime {
    type RuntimeBlock = Block;
}

/// The Aura slot duration. When things are working well, this will also be the block time.
const BLOCK_TIME: u64 = 3000;

impl parachain_piece::ParachainPieceConfig for Runtime {
    // Use the para ID 2_000 which is the first available in the rococo-local runtime.
    // This is the default value, so this could be omitted, but explicit is better.
    const PARA_ID: u32 = 2_000;

    type SetRelayParentNumberStorage = tuxedo_parachain_core::RelayParentNumberStorage;
}

/// The Outer / Aggregate Constraint Checker for the Parachain runtime.
///
/// It is comprized of two individual chekers:
///   First, the parachain inherent piece
///   Second, the constraint checker from the normal Tuxedo Template Runtime.
///
/// That second checker, the normal tuxedo template runtime, is itself an aggregate
/// constraint checker aggregated from idividual pieces such as money, amoeba, and others.
/// Therefore, this crate shows:
///   Generally, how to perform recursive aggregation of constraint checkers.
///   Specifically, how to elegantly transform a sovereign runtime into a parachain runtime by wrapping.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
#[tuxedo_constraint_checker(OuterVerifier)]
pub enum OuterConstraintChecker {
    /// All other calls are delegated to the normal Tuxedo Template Runtime.
    Inner(inner_runtime::OuterConstraintChecker),

    // TODO This one is last for now so that I can write a hacky algorithm to scrape
    // the inherent data and assume it is last.
    /// Set some parachain related information via an inherent extrinsic.
    ParachainInfo(InherentAdapter<parachain_piece::SetParachainInfo<Runtime>>),
}

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

    ///Grandpa Authority IDs - All equally weighted
    fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
        use hex_literal::hex;
        use sp_application_crypto::ByteArray;

        [
            // Alice
            hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee"),
            // Bob
            // hex!("d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae69"),
            // Charlie
            // hex!("439660b36c6c03afafca027b910b4fecf99801834c62a5e6006f27d978de234f"),
            // Dave
            // hex!("5e639b43e0052c47447dac87d6fd2b6ec50bdd4d0f614e4299c665249bbd09d9"),
            // Eve
            // hex!("1dfe3e22cc0d45c70779c1095f7489a8ef3cf52d62fbd8c2fa38c9f1723502b5"),
            // Ferdie
            // hex!("568cb4a574c6d178feb39c27dfc8b3f789e5f5423e19c71633c748b9acf086b5"),
        ]
        .iter()
        .map(|hex| {
            (
                GrandpaId::from_slice(hex.as_ref())
                    .expect("Valid Grandpa authority hex was provided"),
                1,
            )
        })
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

    impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
            Self::grandpa_authorities()
        }

        fn current_set_id() -> sp_consensus_grandpa::SetId {
            0u64
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_consensus_grandpa::SetId,
            _authority_id: sp_consensus_grandpa::AuthorityId,
        ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }

    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
            use tuxedo_parachain_core::ParachainExecutiveExtension;
            Executive::collect_collation_info(header)
        }
    }
}

// Register the `validate_block` function that Polkadot validators will call to verify this parachain block.
tuxedo_parachain_core::register_validate_block!(Block, OuterVerifier, OuterConstraintChecker);
