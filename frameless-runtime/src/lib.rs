#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use amoeba::{AmoebaCreation, AmoebaMitosis};
use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;

use sp_api::impl_runtime_apis;
use sp_runtime::{
    create_runtime_str, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
use sp_std::vec::Vec;

use sp_core::OpaqueMetadata;
#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod amoeba;
mod poe;
mod runtime_upgrade;
//TODO kitties piece needs ported for merge
//mod kitties;
pub mod money;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    redeemer::{SigCheck, UpForGrabs},
    types::{Output, Transaction as TuxedoTransaction},
    Redeemer, Verifier,
};

#[cfg(feature = "std")]
use tuxedo_core::types::OutputRef;

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
        type Public = sp_consensus_grandpa::AuthorityId;
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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisConfig {
    pub genesis_utxos: Vec<Output<OuterRedeemer>>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        use hex_literal::hex;

        const SHAWN_PUB_KEY_BYTES: [u8; 32] =
            hex!("d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67");

        // Initial Config just for a Money UTXO
        GenesisConfig {
            genesis_utxos: vec![Output {
                redeemer: OuterRedeemer::SigCheck(SigCheck {
                    owner_pubkey: SHAWN_PUB_KEY_BYTES.into(),
                }),
                payload: DynamicallyTypedData {
                    data: 100u128.encode(),
                    type_id: <money::Coin as UtxoData>::TYPE_ID,
                },
            }],
        }

        // TODO: Initial UTXO for Kitties

        // TODO: Initial UTXO for Existence
    }
}

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        // we have nothing to put into storage in genesis, except this:
        storage
            .top
            .insert(sp_storage::well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec());

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

pub type Transaction = TuxedoTransaction<OuterRedeemer, OuterVerifier>;
pub type BlockNumber = u32;
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, Transaction>;
pub type Executive = tuxedo_core::Executive<Block, OuterRedeemer, OuterVerifier>;

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

impl From<UpForGrabs> for OuterRedeemer {
    fn from(value: UpForGrabs) -> Self {
        Self::UpForGrabs(value)
    }
}

impl From<SigCheck> for OuterRedeemer {
    fn from(value: SigCheck) -> Self {
        Self::SigCheck(value)
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
    /// Verifies monetary transactions in a basic fungible cryptocurrency
    Money(money::MoneyVerifier),
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
    /// Error from the Money piece
    Money(money::VerifierError),
    /// Error from the Amoeba piece
    Amoeba(amoeba::VerifierError),
    /// Error from the PoE piece
    Poe(poe::VerifierError),
    /// Error from the Runtime Upgrade piece
    RuntimeUpgrade(runtime_upgrade::VerifierError),
}

// We impl conversions from each of the inner error types to the outer error type.
// This should also be done by a macro

impl From<money::VerifierError> for OuterVerifierError {
    fn from(e: money::VerifierError) -> Self {
        Self::Money(e)
    }
}

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
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, OuterVerifierError> {
        Ok(match self {
            Self::Money(money) => money.verify(input_data, output_data)?,
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

impl From<AmoebaCreation> for OuterVerifier {
    fn from(value: AmoebaCreation) -> Self {
        Self::AmoebaCreation(value)
    }
}

impl From<AmoebaMitosis> for OuterVerifier {
    fn from(value: AmoebaMitosis) -> Self {
        Self::AmoebaMitosis(value)
    }
}

//TODO the rest of these impl blocks. For now I'm only doing these two
// because they are the only two I use in my wallet prototype

/// The main struct in this module. In frame this comes from `construct_runtime!`
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
        .map(|hex| (GrandpaId::from_slice(hex.as_ref()).expect("Valid Grandpa authority hex was provided"), 1))
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

        fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            // Tuxedo does not yet support inherents
            Default::default()
        }

        fn check_inherents(
            _block: Block,
            _data: sp_inherents::InherentData
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
	use sp_core::hexdisplay::HexDisplay;
	use sp_core::{H512, testing::SR25519};
	use sp_keystore::testing::KeyStore;
	use sp_keystore::{KeystoreExt, SyncCryptoStore};
	use hex_literal::hex;

	use std::sync::Arc;

	// other random account generated with subkey
	const SHAWN_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	const GENESIS_UTXO_MONEY: [u8; 32] = hex!("79eabcbd5ef6e958c6a7851b36da07691c19bda1835a08f875aa286911800999");

	fn new_test_ext() -> sp_io::TestExternalities {

		let keystore = KeyStore::new();
		let shawn_pub_key =
			keystore.sr25519_generate_new(SR25519, Some(SHAWN_PHRASE)).unwrap();

		let mut t = GenesisConfig::default()
			.build_storage()
			.expect("Frameless system builds valid default genesis config");

		let mut ext = sp_io::TestExternalities::from(t);
		ext.register_extension(KeystoreExt(Arc::new(keystore)));
		ext
	}

	#[test]
	fn utxo_money_test_genesis() {
		new_test_ext().execute_with(|| {
			let keystore = KeyStore::new();
			let shawn_pub_key =
				keystore.sr25519_generate_new(SR25519, Some(SHAWN_PHRASE)).unwrap();

			// Grab genesis value from storage and assert it is correct
			let genesis_utxo = Output {
				redeemer: OuterRedeemer::SigCheck(SigCheck{
					owner_pubkey: shawn_pub_key.into()
				}),
				payload: DynamicallyTypedData {
					data: 100u128.encode(),
					type_id: <money::Coin as UtxoData>::TYPE_ID,
				},
			};

			let output_ref = OutputRef {
                // Genesis UTXOs don't come from any real transaction, so just uze the zero hash
                tx_hash: <Header as sp_api::HeaderT>::Hash::zero(),
                index: 0 as u32,
            };

			let encoded_utxo =
				sp_io::storage::get(&output_ref.encode()).expect("Retrieve Genesis UTXO");
			let utxo = Output::<OuterRedeemer>::decode(&mut &encoded_utxo[..]).expect("Can Decode UTXO correctly");
			assert_eq!(utxo, genesis_utxo);
		})
	}
}
