#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_api::{impl_runtime_apis, HashT};
use sp_runtime::{
	create_runtime_str,
	impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Extrinsic},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
		ValidTransaction,
	},
	ApplyExtrinsicResult, BoundToRuntimeAppPublic,
};
use sp_std::prelude::*;
use sp_std::{vec::Vec, collections::btree_set::BTreeSet};

use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::{hexdisplay::HexDisplay, OpaqueMetadata, H256};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

mod tuxedo_types;
mod redeemer;
mod support_macros;
use tuxedo_types::*;
use redeemer::*;
use support_macros::*;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datas-tructures.
pub mod opaque {
	use super::*;
	// TODO: eventually you will have to change this.
	type OpaqueExtrinsic = BasicExtrinsic;
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
	state_version: 0,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

/// The type that provides the genesis storage values for a new chain
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
pub struct GenesisConfig;

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		// we have nothing to put into storage in genesis, except this:
		storage.top.insert(
			well_known_keys::CODE.into(),
			WASM_BINARY.unwrap().to_vec()
		);
		Ok(())
	}

	//TODO I guess we'll need some genesis config aggregation for each tuxedo piece
	// just like we have in FRAME
}

pub type BlockNumber = u32;
pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = sp_runtime::generic::Block<Header, BasicExtrinsic>;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Call {
	Transact(tuxedo_types::Transaction),
	Upgrade(Vec<u8>),
}

// this extrinsic type does nothing other than fulfill the compiler.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic(Call);

#[cfg(test)]
impl BasicExtrinsic {
	fn new_unsigned(call: Call) -> Self {
		<Self as Extrinsic>::new(call, None).unwrap()
	}
}

impl sp_runtime::traits::Extrinsic for BasicExtrinsic {
	type Call = Call;
	type SignaturePayload = ();

	fn new(data: Self::Call, _: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(Self(data))
	}
}

impl sp_runtime::traits::GetNodeBlockType for Runtime {
	type NodeBlock = opaque::Block;
}

impl sp_runtime::traits::GetRuntimeBlockType for Runtime {
	type RuntimeBlock = Block;
}

const LOG_TARGET: &'static str = "frameless";
const BLOCK_TIME: u64 = 3000;

const HEADER_KEY: &[u8] = b"header"; // 686561646572
const EXTRINSIC_KEY: &[u8] = b"extrinsics";
// const VALUE_KEY: &[u8] = b"VALUE_KEY";


// just FYI:
// :code => 3a636f6465

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

enum UtxoError {
	/// This transaction defines the same output multiple times
	DuplicateOutput,
	/// This transaction defines an output that already existed in the UTXO set
	PreExistingOutput,
	/// The verifier errored.
	/// TODO find a way to pass internal error information from the verifier to here
	VerifierError,
	/// The Redeemer errored.
	/// TODO find a way to pass internal error information from the redeemer to here
	RedeemerError,
}
type DispatchResult = Result<(), UtxoError>;

impl Runtime {
	fn print_state() {
		let mut key = vec![];
		while let Some(next) = sp_io::storage::next_key(&key) {
			let val = sp_io::storage::get(&next).unwrap().to_vec();
			log::trace!(
				target: LOG_TARGET,
				"{} <=> {}",
				HexDisplay::from(&next),
				HexDisplay::from(&val)
			);
			key = next;
		}
	}

	// These next few functions comprize what I sketched out in the UtxoSet trait in element the other day
	// For now, this is complicated enough, so I'll just leave them here. In the future it may be wise to
	// abstract away the utxo set. Especially if we start doing zk stuff and need the nullifiers.

	/// Fetch a utxo from the set.
	fn peek_utxo(output_ref: OutputRef) -> Option<Output> {
		sp_io::storage::get(output_ref).and_then(|d| T::decode(&mut &*d).ok())
	}

	/// Consume a Utxo from the set.
	fn consume_utxo(output_ref: OutputRef) -> Option<Output> {
		let result = peek_utxo(output_ref);
		sp_io::storage::clear(output_ref);
		result
	}

	/// Add a utxo into the set.
	/// This will overwrite any utxo that already exists at this OutputRef. It should never be the
	/// case that there are collisions though. Right??
	fn store_utxo(output_ref: OutputRef, output: Output) {
		sp_io::storage::set(output_ref, output);
	}

	// fn mutate_state<T: Decode + Encode + Default>(key: &[u8], update: impl FnOnce(&mut T)) {
	// 	let mut value = Self::get_state(key).unwrap_or_default();
	// 	update(&mut value);
	// 	sp_io::storage::set(key, &value.encode());
	// }

	fn dispatch_extrinsic(ext: BasicExtrinsic) -> DispatchResult {
		log::debug!(target: LOG_TARGET, "dispatching {:?}", ext);

		Self::mutate_state::<Vec<Vec<u8>>>(EXTRINSIC_KEY, |s| s.push(ext.encode()));

		// execute it
		match ext.0 {
			Call::Transact(transaction) => {
				let valid_transaction = validate_tuxedo_transaction(&transaction)?;
				// There are still missing inputs, so we cannot execute this,
				// although it would be valid in the pool
				if !valid_transaction.requires.is_empty() {
					return Err(());
				}

				update_storage(transaction)
			}
			Call::Upgrade(new_wasm_code) => {
				// NOTE: make sure to upgrade your spec-version!
				sp_io::storage::set(well_known_keys::CODE, &new_wasm_code);
			}
		}

		Ok(())
	}

	fn validate_tuxedo_transaction(transaction: &tuxedo_types::Transaction) -> Result<ValidTransaction, UtxoError> {
		// There should be no duplicate inputs because we used a BTreeSet
		// TODO verify that this assumption is true. What if I encode a Vec and then decode it into a BTreeSet
		// Perhaps it is safer to just use a Vec for inputs here too like we did in the old utxo workshop
		
		// Make sure there are no duplicate outputs
		{
			let output_set: BTreeSet<_> = transaction.outputs.iter().collect();
			ensure!(output_set.len() == transaction.outputs.len(), UtxoError::DuplicateOutput);
		}

		// Build the stripped transaction (with the witnesses stripped) and encode it
		// This will be passed to the redeemers
		let mut stripped = transaction.clone();
		for input in stripped.inputs.iter_mut() {
			input.witness = Vec::new();
		}
		let stripped_encoded = stripped.encode();

		// Check that the redeemers of all inputs are satisfied
		// Keep track of any missing inputs for use in the tagged transaction pool
		let mut missing_inputs = Vec::new();
		for input in transaction.inputs.iter() {
			if let Some(input_utxo) = Self::peek_utxo(&input.output_ref) {
				if !input_utxo.redeemer.redeem(stripped_encoded, input.witness) {
					return Err(UtxoError::RedeemerError);
				}
			} else {
				missing_inputs.push(input.outpoint.clone().as_fixed_bytes().to_vec());
			}
		}

		// Make sure no outputs already exist in storage
		// TODO Actually I don't think we need to do this. It _does_ appear in the original utxo workshop,
		// but I don't see how we could ever have an output collision.

		// Extract the contained data from each input and output

		// Call the verifier

		// Update storage
	}

	fn do_initialize_block(header: &<Block as BlockT>::Header) {
		info!(
			target: LOG_TARGET,
			"Entering initialize_block. header: {:?} / version: {:?}", header, VERSION.spec_version
		);
		sp_io::storage::set(&HEADER_KEY, &header.encode());
	}

	fn do_finalize_block() -> <Block as BlockT>::Header {
		let mut header = Self::get_state::<<Block as BlockT>::Header>(HEADER_KEY)
			.expect("We initialized with header, it never got mutated, qed");

		// the header itself contains the state root, so it cannot be inside the state (circular
		// dependency..). Make sure in execute block path we have the same rule.
		sp_io::storage::clear(&HEADER_KEY);

		let extrinsics = Self::get_state::<Vec<Vec<u8>>>(EXTRINSIC_KEY).unwrap_or_default();
		let extrinsics_root =
			BlakeTwo256::ordered_trie_root(extrinsics, sp_runtime::StateVersion::V0);
		sp_io::storage::clear(&EXTRINSIC_KEY);
		header.extrinsics_root = extrinsics_root;

		let raw_state_root = &sp_io::storage::root(VERSION.state_version())[..];
		header.state_root = sp_core::H256::decode(&mut &raw_state_root[..]).unwrap();

		info!(target: LOG_TARGET, "finalizing block {:?}", header);
		header
	}

	fn do_execute_block(block: Block) {
		info!(target: LOG_TARGET, "Entering execute_block. block: {:?}", block);

		for extrinsic in block.clone().extrinsics {
			// block import cannot fail.
			Runtime::dispatch_extrinsic(extrinsic).unwrap();
		}

		// check state root
		let raw_state_root = &sp_io::storage::root(VERSION.state_version())[..];
		let state_root = H256::decode(&mut &raw_state_root[..]).unwrap();
		Self::print_state();
		assert_eq!(block.header.state_root, state_root);

		// check extrinsics root.
		let extrinsics =
			block.extrinsics.into_iter().map(|x| x.encode()).collect::<Vec<_>>();
		let extrinsics_root =
			BlakeTwo256::ordered_trie_root(extrinsics, sp_core::storage::StateVersion::V0);
		assert_eq!(block.header.extrinsics_root, extrinsics_root);
	}

	fn do_apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
		info!(target: LOG_TARGET, "Entering apply_extrinsic: {:?}", extrinsic);

		Self::dispatch_extrinsic(extrinsic)
			.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(0)))?;

		Ok(Ok(()))
	}

	fn do_validate_transaction(
		source: TransactionSource,
		tx: <Block as BlockT>::Extrinsic,
		block_hash: <Block as BlockT>::Hash,
	) -> TransactionValidity {
		log::debug!(
			target: LOG_TARGET,
			"Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}",
			source,
			tx,
			block_hash
		);

		// we don't know how to validate this -- It should be fine??

		let data = tx.0;
		Ok(ValidTransaction { provides: vec![data.encode()], ..Default::default() })
	}

	fn do_inherent_extrinsics(_: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
		log::debug!(target: LOG_TARGET, "Entering do_inherent_extrinsics");
		Default::default()
	}

	fn do_check_inherents(
		_: Block,
		_: sp_inherents::InherentData,
	) -> sp_inherents::CheckInherentsResult {
		log::debug!(target: LOG_TARGET, "Entering do_check_inherents");
		Default::default()
	}
}

impl_runtime_apis! {
	// https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Self::do_execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Self::do_initialize_block(header)
		}
	}

	// https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Self::do_apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Self::do_finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			Self::do_inherent_extrinsics(data)
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData
		) -> sp_inherents::CheckInherentsResult {
			Self::do_check_inherents(block, data)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Self::do_validate_transaction(source, tx, block_hash)
		}
	}

	// Ignore everything after this.
	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Default::default())
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(_header: &<Block as BlockT>::Header) {
			// we do not do anything.
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
