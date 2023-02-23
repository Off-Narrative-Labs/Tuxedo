#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_api::{impl_runtime_apis, HashT};
use sp_runtime::{
    create_runtime_str, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT},
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionPriority, TransactionSource,
        TransactionValidity, TransactionValidityError, ValidTransaction,
    },
    ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::{hexdisplay::HexDisplay, OpaqueMetadata, H256};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

mod amoeba;
mod poe;
mod runtime_upgrade;
use tuxedo_core::{Output, OutputRef, TypedData};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    // TODO: eventually you will have to change this.
    type OpaqueExtrinsic = Transaction;
    // type OpaqueExtrinsic = Vec<u8>;

    /// Opaque block header type.
    pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;

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
        type Public = sp_finality_grandpa::AuthorityId;
    }
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("frameless-runtime"),
    impl_name: create_runtime_str!("frameless-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    // Tuxedo only supports state version 1. You must always use version 1.
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

/// The type that provides the genesis storage values for a new chain
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
pub struct GenesisConfig;

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        // we have nothing to put into storage in genesis, except this:
        storage
            .top
            .insert(well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec());
        Ok(())
    }

    //TODO I guess we'll need some genesis config aggregation for each tuxedo piece
    // just like we have in FRAME
}

pub type Transaction = tuxedo_core::Transaction<OuterRedeemer, OuterVerifier>;
pub type BlockNumber = u32;
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, Transaction>;
pub type Executive = tuxedo_core::Executive<Block, OuterVerifier, OuterRedeemer>;

impl sp_runtime::traits::GetNodeBlockType for Runtime {
    type NodeBlock = opaque::Block;
}

impl sp_runtime::traits::GetRuntimeBlockType for Runtime {
    type RuntimeBlock = Block;
}

/// The Aura slot duration. When things are working well, this will also be the block time.
const BLOCK_TIME: u64 = 3000;

//TODO this should be implemented by the aggregation macro I guess
/// A redeemer checks that an individual input can be consumed. For example that it is signed properly
/// To begin playing, we will have two kinds. A simple signature check, and an anyone-can-consume check.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum OuterRedeemer {
    SigCheck(SigCheck),
    UpForGrabs(UpForGrabs),
}

//TODO this should be implemented by the aggregation macro I guess
impl Redeemer for OuterRedeemer {
    fn redeem(&self, simplified_tx: &[u8], witness: &[u8]) -> bool {
        match self {
            Self::SigCheck(sig_check) => sig_check.redeem(simplified_tx, witness),
            Self::UpForGrabs(up_for_grabs) => up_for_grabs.redeem(simplified_tx, witness),
        }
    }
}

// Observation: For some applications, it will be invalid to simply delete
// a UTXO without any further processing. Therefore, we explicitly include
// AmoebaDeath and PoeRevoke on an application-specific basis

//TODO this should be implemented by the aggregation macro I guess
/// A verifier is a piece of logic that can be used to check a transaction.
/// For any given Tuxedo runtime there is a finite set of such verifiers.
/// For example, this may check that input token values exceed output token values.
#[cfg_attr(
    feature = "std",
    derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf)
)]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum OuterVerifier {
    /// Verifies that an amoeba can split into two new amoebas
    AmoebaMitosis(amoeba::AmoebaMitosis),
    /// Verifies that a single amoeba is simply removed from the state
    AmoebaDeath(amoeba::AmoebaDeath),
    /// Verifies that a single amoeba is simply created from the void... and it is good
    AmoebaCreation(amoeba::AmoebaCreation),
    /// Verifies that new valid proofs of existence are claimed
    PoeClaim(poe::PoeClaim),
    /// Verifies that proofs of existence are revoked.
    PoeRevoke(poe::PoeRevoke),
    /// Verifies that one winning claim came earlier than all the other claims, and thus
    /// the losing claims can be removed from storage.
    PoeDispute(poe::PoeDispute),
    /// Upgrade the Wasm Runtime
    RuntimeUpgrade(runtime_upgrade::RuntimeUpgrade),
}

/// An aggregated error type with a variant for each tuxedo piece
/// TODO This should probably be macro generated
#[derive(Debug)]
pub enum OuterVerifierError {
    /// Error from the Amoeba piece
    Amoeba(amoeba::VerifierError),
    /// Error from the PoE piece
    Poe(poe::VerifierError),
    /// Error from the Runtime Upgrade piece
    RuntimeUpgrade(runtime_upgrade::VerifierError),
}

// We impl conversions from each of the inner error types to the outer error type.
// This should also be done by a macro

impl From<amoeba::VerifierError> for OuterVerifierError {
    fn from(e: amoeba::VerifierError) -> Self {
        Self::Amoeba(e)
    }
}

impl From<poe::VerifierError> for OuterVerifierError {
    fn from(e: poe::VerifierError) -> Self {
        Self::Poe(e)
    }
}

impl From<runtime_upgrade::VerifierError> for OuterVerifierError {
    fn from(e: runtime_upgrade::VerifierError) -> Self {
        Self::RuntimeUpgrade(e)
    }
}

impl Verifier for OuterVerifier {
    type Error = OuterVerifierError;

    fn verify(
        &self,
        input_data: &[TypedData],
        output_data: &[TypedData],
    ) -> Result<TransactionPriority, OuterVerifierError> {
        Ok(match self {
            Self::AmoebaMitosis(amoeba_mitosis) => {
                amoeba_mitosis.verify(input_data, output_data)?
            }
            Self::AmoebaDeath(amoeba_death) => amoeba_death.verify(input_data, output_data)?,
            Self::AmoebaCreation(amoeba_creation) => {
                amoeba_creation.verify(input_data, output_data)?
            }
            Self::PoeClaim(poe_claim) => poe_claim.verify(input_data, output_data)?,
            Self::PoeRevoke(poe_revoke) => poe_revoke.verify(input_data, output_data)?,
            Self::PoeDispute(poe_dispute) => poe_dispute.verify(input_data, output_data)?,
            Self::RuntimeUpgrade(runtime_upgrade) => {
                runtime_upgrade.verify(input_data, output_data)?
            }
        })
    }
}

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

#[derive(Debug)]
enum UtxoError<OuterVerifierError> {
    /// This transaction defines the same input multiple times
    DuplicateInput,
    /// This transaction defines the same output multiple times
    DuplicateOutput,
    /// This transaction defines an output that already existed in the UTXO set
    PreExistingOutput,
    /// The verifier errored.
    VerifierError(OuterVerifierError),
    /// The Redeemer errored.
    /// TODO determine whether it is useful to relay an inner error from the redeemer.
    /// So far, I haven't seen a case, although it seems reasonable to think there might be one.
    RedeemerError,
    /// One or more of the inputs required by this transaction is not present in the UTXO set
    MissingInput,
}

impl_runtime_apis! {
    // https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executiveexecute_block(block)
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
            // Tuxedo does not yet support inherents
            Default::default()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData
        ) -> sp_inherents::CheckInherentsResult {
            // Tuxedo does not yet support inherents
            Default::default()
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

    // Ignore everything after this.
    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            // Tuxedo does not yet support metadata
            OpaqueMetadata::new(Default::default())
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(_header: &<Block as BlockT>::Header) {
            // Tuxedo does not yet support offchain workers, and maybe never will.
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            info!(target: "frameless", "🖼️ Entering generate_session_keys. seed: {:?}", seed);
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
            // The only authority is Alice. This makes things work nicely in `--dev` mode
            use sp_application_crypto::ByteArray;

            vec![
                AuraId::from_slice(
                    &hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec()
                ).unwrap()
            ]
        }
    }

    impl sp_finality_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> sp_finality_grandpa::AuthorityList {
            use sp_application_crypto::ByteArray;
            vec![
                (
                    sp_finality_grandpa::AuthorityId::from_slice(
                        &hex_literal::hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee").to_vec()
                    ).unwrap(),
                    1
                )
            ]
        }

        fn current_set_id() -> sp_finality_grandpa::SetId {
            0u64
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_finality_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: sp_finality_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: sp_finality_grandpa::SetId,
            _authority_id: sp_finality_grandpa::AuthorityId,
        ) -> Option<sp_finality_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Encode;
    use sp_core::hexdisplay::HexDisplay;

    #[test]
    fn host_function_call_works() {
        sp_io::TestExternalities::new_empty().execute_with(|| {
            sp_io::storage::get(&HEADER_KEY);
        })
    }

    #[test]
    fn encode_examples() {
        // run with `cargo test -p frameless-runtime -- --nocapture`
        let extrinsic = BasicExtrinsic::new_unsigned(Call::SetValue(42));
        println!("ext {:?}", HexDisplay::from(&extrinsic.encode()));
        println!("key {:?}", HexDisplay::from(&VALUE_KEY));
    }
}
