//! The Tuxedo Template Runtime is an example runtime that uses
//! most of the pieces provided in the wardrobe.
//!
//! Runtime developers wishing to get started with Tuxedo should
//! consider copying this template.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;

use sp_api::impl_runtime_apis;
use sp_runtime::{
    create_runtime_str, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT},
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
    },
    ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;

use sp_core::OpaqueMetadata;
#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    tuxedo_constraint_checker, tuxedo_verifier,
    types::{Input, Transaction as TuxedoTransaction},
    verifier::{SigCheck, ThresholdMultiSignature, UpForGrabs},
};

pub use amoeba;
pub use kitties;
pub use money;
pub use poe;
pub use runtime_upgrade;

#[cfg(feature = "std")]
use tuxedo_core::types::OutputRef;

/// Target for logging from the template runtime.
/// Individual pieces should not use this target, nor should Tuxedo client or core.
const LOG_TARGET: &str = "tuxedo-template-runtime";

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;

    /// Opaque block type.
    pub type Block = sp_runtime::generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

    // This part is necessary for generating session keys in the runtime
    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: AuraAppPublic,
            pub grandpa: GrandpaAppPublic,
        }
    }

    // Typically these are not implemented manually, but rather for the pallet associated with the
    // keys. Here we are not using the pallets, and these implementations are trivial, so we just
    // re-write them.
    pub struct AuraAppPublic;
    impl BoundToRuntimeAppPublic for AuraAppPublic {
        type Public = AuraId;
    }

    pub struct GrandpaAppPublic;
    impl BoundToRuntimeAppPublic for GrandpaAppPublic {
        type Public = sp_consensus_grandpa::AuthorityId;
    }
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("tuxedo-template-runtime"),
    impl_name: create_runtime_str!("tuxedo-template-runtime"),
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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisConfig {
    pub genesis_utxos: Vec<Output>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
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
                    payload: DynamicallyTypedData {
                        data: 100u128.encode(),
                        type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                    },
                },
                Output {
                    verifier: OuterVerifier::ThresholdMultiSignature(ThresholdMultiSignature {
                        threshold: 1,
                        signatories: vec![SHAWN_PUB_KEY_BYTES.into(), ANDREW_PUB_KEY_BYTES.into()],
                    }),
                    payload: DynamicallyTypedData {
                        data: 100u128.encode(),
                        type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                    },
                },
            ],
        }

        // TODO: Initial UTXO for Kitties

        // TODO: Initial UTXO for Existence
    }
}

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        // we have nothing to put into storage in genesis, except this:
        storage.top.insert(
            sp_storage::well_known_keys::CODE.into(),
            WASM_BINARY.unwrap().to_vec(),
        );

        for (index, utxo) in self.genesis_utxos.iter().enumerate() {
            let output_ref = OutputRef {
                // Genesis UTXOs don't come from any real transaction, so just use the zero hash
                tx_hash: <Header as sp_api::HeaderT>::Hash::zero(),
                index: index as u32,
            };
            storage.top.insert(output_ref.encode(), utxo.encode());
        }

        Ok(())
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

/// A verifier checks that an individual input can be consumed. For example that it is signed properly
/// To begin playing, we will have two kinds. A simple signature check, and an anyone-can-consume check.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
#[tuxedo_verifier]
pub enum OuterVerifier {
    SigCheck(SigCheck),
    UpForGrabs(UpForGrabs),
    ThresholdMultiSignature(ThresholdMultiSignature),
}

impl poe::PoeConfig for Runtime {
    fn block_height() -> u32 {
        Executive::block_height()
    }
}

impl timestamp::TimestampConfig for Runtime {
    fn block_height() -> u32 {
        Executive::block_height()
    }
}

// Observation: For some applications, it will be invalid to simply delete
// a UTXO without any further processing. Therefore, we explicitly include
// AmoebaDeath and PoeRevoke on an application-specific basis

/// A constraint checker is a piece of logic that can be used to check a transaction.
/// For any given Tuxedo runtime there is a finite set of such constraint checkers.
/// For example, this may check that input token values exceed output token values.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
#[tuxedo_constraint_checker(OuterVerifier)]
pub enum OuterConstraintChecker {
    /// Checks monetary transactions in a basic fungible cryptocurrency
    Money(money::MoneyConstraintChecker<0>),
    /// Checks Free Kitty transactions
    FreeKittyConstraintChecker(kitties::FreeKittyConstraintChecker),
    /// Checks that an amoeba can split into two new amoebas
    AmoebaMitosis(amoeba::AmoebaMitosis),
    /// Checks that a single amoeba is simply removed from the state
    AmoebaDeath(amoeba::AmoebaDeath),
    /// Checks that a single amoeba is simply created from the void... and it is good
    AmoebaCreation(amoeba::AmoebaCreation),
    /// Checks that new valid proofs of existence are claimed
    PoeClaim(poe::PoeClaim<Runtime>),
    /// Checks that proofs of existence are revoked.
    PoeRevoke(poe::PoeRevoke),
    /// Checks that one winning claim came earlier than all the other claims, and thus
    /// the losing claims can be removed from storage.
    PoeDispute(poe::PoeDispute),
    /// Set the block's timestamp via an inherent extrinsic.
    SetTimestamp(timestamp::SetTimestamp<Runtime>),
    /// Upgrade the Wasm Runtime
    RuntimeUpgrade(runtime_upgrade::RuntimeUpgrade),
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
            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 In `apply_extrinsic`"
            );
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::close_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 In `inherent_extrinsics`."
            );

            // Extract the complete parent block from the inheret data
            use tuxedo_core::inherents::PARENT_INHERENT_IDENTIFIER;
            let parent: Block = data
                .get_data(&PARENT_INHERENT_IDENTIFIER)
                .expect("1")
                .expect("2");

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 The previous block had {} extrinsics.", parent.extrinsics().len()
            );

            // Extract the timestamp
            use timestamp::{BestTimestamp, NotedTimestamp};

            let timestamp_millis: u64 = data
                .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
                .expect("Inherent data should decode properly")
                .expect("Timestamp inherent data should be present.");
            let new_best_timestamp = BestTimestamp(timestamp_millis);
            let new_noted_timestamp = NotedTimestamp(timestamp_millis);

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 timestamp_millis:: {timestamp_millis}"
            );

            // We need to initialize the timestamp somehow, and right now the way we do it
            // is to allow a transaction that does not consume any previous best on block height 1 only.
            // A much more elegant solution would be to allow transactions in the genesis block, then we
            // could use the same scraping logic as always.
            use tuxedo_core::types::OutputRef;
            use sp_api::HashT;

            let mut inputs = Vec::new();
            if parent.header.number != 0 {
                let prev_set_timestamp = parent
                    .extrinsics()
                    .iter()
                    .find(|extrinsic| {
                        matches!(extrinsic.checker, OuterConstraintChecker::SetTimestamp(_))
                    })
                    .expect("SetTimestamp extrinsic should appear in every block.");

                let index = prev_set_timestamp
                    .outputs
                    .iter()
                    .position(|output| {
                        output.payload.extract::<BestTimestamp>().is_ok()
                    })
                    .expect("SetTimestamp extrinsic should have an output that decodes as a StorableTimestamp.")
                    .try_into()
                    .expect("There should not be more than u32::max_value transactions in a block.");

                let output_ref = OutputRef {
                    tx_hash: BlakeTwo256::hash_of(&prev_set_timestamp.encode()),
                    index,
                };

                let input = Input {
                    output_ref,
                    // The best time needs to be easily taken. For now I'll assume it is up for grabs.
                    // We can make this an eviction once that is implemented.
                    redeemer: Vec::new(),
                };

                inputs.push(input);
            }

            let best_output = Output {
                payload: new_best_timestamp.into(),
                verifier: OuterVerifier::UpForGrabs(UpForGrabs),
            };
            let noted_output = Output {
                payload: new_noted_timestamp.into(),
                verifier: OuterVerifier::UpForGrabs(UpForGrabs),
            };

            let timestamp_tx = Transaction {
                inputs,
                peeks: Vec::new(),
                outputs: vec![best_output, noted_output],
                checker: timestamp::SetTimestamp(Default::default()).into(),
            };

            // log::info!(
            //     target: LOG_TARGET,
            //     "🕰️🖴 Timestamp transaction is: \n{:#?}", timestamp_tx
            // );

            // Extract the Aura slot (although we are not using it yet)
            // Actually, I wonder how Aura is working so well without this inherent...
            // Maybe we are failing to check something important and aura-related.

            // I guess we can probably author in other peoples' slots...
            // This could be an expert exercise for Substrate students. We launch a chain that doesn't
            // check for the right author in each slot. Then start a chain, and author all the blocks,
            // and challenge students to figure out the problem and fix it.

            use sp_consensus_aura::Slot;

            let slot: Slot = data
                .get_data(&sp_consensus_aura::inherents::INHERENT_IDENTIFIER)
                .unwrap()
                .unwrap();

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 Aura slot is: {:#?}", slot
            );


            // Return just the timestamp extrinsic for now.
            // Later we will either handle Aura properly or switch to nimbus.
            // Soon we will add the parachain inherent in here.
            vec![timestamp_tx]
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData
        ) -> sp_inherents::CheckInherentsResult {

            use sp_inherents::CheckInherentsResult;
            let mut results = CheckInherentsResult::new();

            // Timestamp: We need to check that the timestamp in the block is close to the current time
            use timestamp::BestTimestamp;

            /// The maximum amount by which a valid block's timestamp may be ahead of our current local time.
            /// 1 minute.
            /// TODO make it part of the config trait.
            const MAX_DRIFT: u64 = 60_000;

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 In `check_inherents`"
            );

            // Extract the local view of time from the inherent data
            let local_timestamp: u64 = data
                .get_data(&sp_timestamp::INHERENT_IDENTIFIER)
                .expect("Inherent data should decode properly")
                .expect("Timestamp inherent data should be present.");

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 Local timestamp is:    {:#?}", local_timestamp
            );

            // Extract the timestamp from the block
            // I guess this is done by scraping the transactions, right?
            // TODO figure out Where this is done in FRAME world and make sure I'm not doing something stupid here.
            let set_timestamp_ext = block
                .extrinsics()
                .iter()
                .find(|extrinsic| {
                    matches!(extrinsic.checker, OuterConstraintChecker::SetTimestamp(_))
                })
                .expect("SetTimestamp extrinsic should appear in every block.");

            let on_chain_timestamp = set_timestamp_ext
                .outputs
                .iter()
                .find(|output| {
                    output.payload.extract::<BestTimestamp>().is_ok()
                })
                .expect("SetTimestamp extrinsic should have an output that decodes as a StorableTimestamp.")
                .payload
                // TODO sucks that we have to extract it twice. Is there some way to use the extracted one from before?
                .extract::<BestTimestamp>()
                .expect("It should decode because we already checked that.")
                .0;

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 In-block timestamp is: {:#?}", on_chain_timestamp
            );

            // Make the comparison for too far in future
            if on_chain_timestamp > local_timestamp  + MAX_DRIFT {
                log::info!(
                    target: LOG_TARGET,
                    "🕰️🖴 Block timestamp is too far in future. About to push an error"
                );
                results
                    .put_error(sp_timestamp::INHERENT_IDENTIFIER, &sp_timestamp::InherentError::TooFarInFuture)
                    .expect("Should be able to put some errors");
            }

            // Although FRAME makes the check for the minimum interval here, we don't.
            // We make that check in the on-chain constraint checker.
            // That's where we have easy access to the timestamp of the previous block
            // FRAME's checks: github.com/paritytech/polkadot-sdk/blob/945ebbbc/substrate/frame/timestamp/src/lib.rs#L299-L306

            log::info!(
                target: LOG_TARGET,
                "🕰️🖴 About to return from `check_inherents`. Results okay: {}, fatal_error: {}", results.ok(), results.fatal_error()
            );
            results
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {

            // Check if the transaction in question is an inherent. If so return invalid transaction.
            // Inherents are never valid in the pool
            match tx.checker {
                OuterConstraintChecker::SetTimestamp(_) => {
                    return Err(InvalidTransaction::Call.into());
                }
                _ => ()
            }


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
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Encode;
    use sp_core::testing::SR25519;
    use sp_keystore::testing::MemoryKeystore;
    use sp_keystore::{Keystore, KeystoreExt};

    use std::sync::Arc;

    // other random account generated with subkey
    const SHAWN_PHRASE: &str =
        "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    const ANDREW_PHRASE: &str =
        "monkey happy total rib lumber scrap guide photo country online rose diet";

    fn new_test_ext() -> sp_io::TestExternalities {
        let keystore = MemoryKeystore::new();

        let t = GenesisConfig::default()
            .build_storage()
            .expect("System builds valid default genesis config");

        let mut ext = sp_io::TestExternalities::from(t);
        ext.register_extension(KeystoreExt(Arc::new(keystore)));
        ext
    }

    #[test]
    fn utxo_money_test_genesis() {
        new_test_ext().execute_with(|| {
            let keystore = MemoryKeystore::new();
            let shawn_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(SHAWN_PHRASE))
                .unwrap();

            // Grab genesis value from storage and assert it is correct
            let genesis_utxo = Output {
                verifier: OuterVerifier::SigCheck(SigCheck {
                    owner_pubkey: shawn_pub_key.into(),
                }),
                payload: DynamicallyTypedData {
                    data: 100u128.encode(),
                    type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                },
            };

            let output_ref = OutputRef {
                // Genesis UTXOs don't come from any real transaction, so just uze the zero hash
                tx_hash: <Header as sp_api::HeaderT>::Hash::zero(),
                index: 0 as u32,
            };

            let encoded_utxo =
                sp_io::storage::get(&output_ref.encode()).expect("Retrieve Genesis UTXO");
            let utxo = Output::decode(&mut &encoded_utxo[..]).expect("Can Decode UTXO correctly");
            assert_eq!(utxo, genesis_utxo);
        })
    }

    #[test]
    fn utxo_money_multi_sig_genesis_test() {
        new_test_ext().execute_with(|| {
            let keystore = MemoryKeystore::new();
            let shawn_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(SHAWN_PHRASE))
                .unwrap();
            let andrew_pub_key = keystore
                .sr25519_generate_new(SR25519, Some(ANDREW_PHRASE))
                .unwrap();

            let genesis_multi_sig_utxo = Output {
                verifier: OuterVerifier::ThresholdMultiSignature(ThresholdMultiSignature {
                    threshold: 1,
                    signatories: vec![shawn_pub_key.into(), andrew_pub_key.into()],
                }),
                payload: DynamicallyTypedData {
                    data: 100u128.encode(),
                    type_id: <money::Coin<0> as UtxoData>::TYPE_ID,
                },
            };

            let output_ref = OutputRef {
                // Genesis UTXOs don't come from any real transaction, so just uze the zero hash
                tx_hash: <Header as sp_api::HeaderT>::Hash::zero(),
                index: 1 as u32,
            };

            let encoded_utxo =
                sp_io::storage::get(&output_ref.encode()).expect("Retrieve Genesis MultiSig UTXO");
            let utxo = Output::decode(&mut &encoded_utxo[..]).expect("Can Decode UTXO correctly");
            assert_eq!(utxo, genesis_multi_sig_utxo);
        })
    }
}
