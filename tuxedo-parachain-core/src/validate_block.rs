//! The actual implementation of the validate block functionality.

use super::{
    trie_cache, MemoryOptimizedValidationParams, ParachainInherentDataUtxo,
    RelayParentNumberStorage, GetRelayParentNumberStorage,
};
use cumulus_primitives_core::{
    relay_chain::Hash as RHash, ParachainBlockData, PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;

use polkadot_parachain_primitives::primitives::{
    HeadData, RelayChainBlockNumber, ValidationResult,
};
use tuxedo_core::{types::Transaction, ConstraintChecker, Executive, Verifier};

use cumulus_primitives_core::ParaId;
use parity_scale_codec::{Decode, Encode};
use sp_core::storage::{ChildInfo, StateVersion};
use sp_externalities::{set_and_run_with_externalities, Externalities};
use sp_io::KillStorageResult;
use sp_runtime::traits::{Block as BlockT, Extrinsic, HashingFor, Header as HeaderT};
use sp_std::prelude::*;
use sp_trie::MemoryDB;

type TrieBackend<B> = sp_state_machine::TrieBackend<
    MemoryDB<HashingFor<B>>,
    HashingFor<B>,
    trie_cache::CacheProvider<HashingFor<B>>,
>;

type Ext<'a, B> = sp_state_machine::Ext<'a, HashingFor<B>, TrieBackend<B>>;

fn with_externalities<F: FnOnce(&mut dyn Externalities) -> R, R>(f: F) -> R {
    sp_externalities::with_externalities(f).expect("Environmental externalities not set.")
}

/// Validate the given parachain block.
///
/// This function is doing roughly the following:
///
/// 1. We decode the [`ParachainBlockData`] from the `block_data` in `params`.
///
/// 2. We are doing some security checks like checking that the `parent_head` in `params`
/// is the parent of the block we are going to check. We also ensure that the `set_validation_data`
/// inherent is present in the block and that the validation data matches the values in `params`.
///
/// 3. We construct the sparse in-memory database from the storage proof inside the block data and
/// then ensure that the storage root matches the storage root in the `parent_head`.
///
/// 4. We replace all the storage related host functions with functions inside the wasm blob.
/// This means instead of calling into the host, we will stay inside the wasm execution. This is
/// very important as the relay chain validator hasn't the state required to verify the block. But
/// we have the in-memory database that contains all the values from the state of the parachain
/// that we require to verify the block.
///
/// 5. We are going to run `check_inherents`. This is important to check stuff like the timestamp
/// matching the real world time.
///
/// 6. The last step is to execute the entire block in the machinery we just have setup. Executing
/// the blocks include running all transactions in the block against our in-memory database and
/// ensuring that the final storage root matches the storage root in the header of the block. In the
/// end we return back the [`ValidationResult`] with all the required information for the validator.
#[doc(hidden)]
pub fn validate_block<B, V, C>(
    MemoryOptimizedValidationParams {
        block_data,
        parent_head,
        relay_parent_number,
        relay_parent_storage_root,
    }: MemoryOptimizedValidationParams,
) -> ValidationResult
where
    B: BlockT<Extrinsic = Transaction<V, C>>,
    V: Verifier,
    C: ConstraintChecker<V>, // + Into<SetParachainInfo<V>>,
{
    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE");
    sp_runtime::runtime_logger::RuntimeLogger::init();
    log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️Entering validate_block implementation");
    // Step 1: Decode block data
    let block_data = parity_scale_codec::decode_from_bytes::<ParachainBlockData<B>>(block_data)
        .expect("Invalid parachain block data");

    // Step 2: Security Checks
    log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️ Step 2");
    let parent_header = parity_scale_codec::decode_from_bytes::<B::Header>(parent_head.clone())
        .expect("Invalid parent head");

    let (header, extrinsics, storage_proof) = block_data.deconstruct();

    let block = B::new(header, extrinsics);
    assert!(
        parent_header.hash() == *block.header().parent_hash(),
        "Invalid parent hash"
    );

    let inherent_data = extract_parachain_inherent_data(&block);

    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE past inherent data scraping");

    validate_validation_data(
        &inherent_data.validation_data,
        relay_parent_number,
        relay_parent_storage_root,
        parent_head,
    );

    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE validation data successfully validated");

    // Step 3: Create the sparse in-memory db
    log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️ Step 3");
    let db = match storage_proof.to_memory_db(Some(parent_header.state_root())) {
        Ok((db, _)) => db,
        Err(_) => panic!("Compact proof decoding failure."),
    };

    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE made db");

    sp_std::mem::drop(storage_proof);

    let cache_provider = trie_cache::CacheProvider::new();
    // We use the storage root of the `parent_head` to ensure that it is the correct root.
    // This is already being done above while creating the in-memory db, but let's be paranoid!!
    let backend = sp_state_machine::TrieBackendBuilder::new_with_cache(
        db,
        *parent_header.state_root(),
        cache_provider,
    )
    .build();

    // Okay, turns out we can get this one, just need to be patient.
    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE backend built");

    // Step 4: Replace host functions
    log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️ Step 4");
    let _guard = (
        // Replace storage calls with our own implementations
        sp_io::storage::host_read.replace_implementation(host_storage_read),
        sp_io::storage::host_set.replace_implementation(host_storage_set),
        sp_io::storage::host_get.replace_implementation(host_storage_get),
        sp_io::storage::host_exists.replace_implementation(host_storage_exists),
        sp_io::storage::host_clear.replace_implementation(host_storage_clear),
        sp_io::storage::host_root.replace_implementation(host_storage_root),
        sp_io::storage::host_clear_prefix.replace_implementation(host_storage_clear_prefix),
        sp_io::storage::host_append.replace_implementation(host_storage_append),
        sp_io::storage::host_next_key.replace_implementation(host_storage_next_key),
        sp_io::storage::host_start_transaction
            .replace_implementation(host_storage_start_transaction),
        sp_io::storage::host_rollback_transaction
            .replace_implementation(host_storage_rollback_transaction),
        sp_io::storage::host_commit_transaction
            .replace_implementation(host_storage_commit_transaction),
        sp_io::default_child_storage::host_get
            .replace_implementation(host_default_child_storage_get),
        sp_io::default_child_storage::host_read
            .replace_implementation(host_default_child_storage_read),
        sp_io::default_child_storage::host_set
            .replace_implementation(host_default_child_storage_set),
        sp_io::default_child_storage::host_clear
            .replace_implementation(host_default_child_storage_clear),
        sp_io::default_child_storage::host_storage_kill
            .replace_implementation(host_default_child_storage_storage_kill),
        sp_io::default_child_storage::host_exists
            .replace_implementation(host_default_child_storage_exists),
        sp_io::default_child_storage::host_clear_prefix
            .replace_implementation(host_default_child_storage_clear_prefix),
        sp_io::default_child_storage::host_root
            .replace_implementation(host_default_child_storage_root),
        sp_io::default_child_storage::host_next_key
            .replace_implementation(host_default_child_storage_next_key),
        sp_io::offchain_index::host_set.replace_implementation(host_offchain_index_set),
        sp_io::offchain_index::host_clear.replace_implementation(host_offchain_index_clear),
    );

    // Step 5: Check inherents.
    // TODO For now I'm skipping this entirely to try to make something "work"
    // run_with_externalities::<B, _, _>(&backend, || {
    // 	let relay_chain_proof = super::RelayChainStateProof::new(
    // 		PID::get(),
    // 		inherent_data.validation_data.relay_parent_storage_root,
    // 		inherent_data.relay_chain_state.clone(),
    // 	)
    // 	.expect("Invalid relay chain state proof");

    // 	let res = CI::check_inherents(&block, &relay_chain_proof);

    // 	if !res.ok() {
    // 		if log::log_enabled!(log::Level::Error) {
    // 			res.into_errors().for_each(|e| {
    // 				log::error!("Checking inherent with identifier `{:?}` failed", e.0)
    // 			});
    // 		}

    // 		panic!("Checking inherents failed");
    // 	}
    // });

    // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE about to enter externalities closure");

    run_with_externalities::<B, _, _>(&backend, || {
        // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE in externalities closure");
        log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️ In the run_with_externalities closure");
        let head_data = HeadData(block.header().encode());

        Executive::<B, V, C>::execute_block(block);

        // panic!("MADE IT HEREEEEEEEEEEEEEEEEEEE returned from execute_block");

        log::info!(target: "tuxvb", "🕵️🕵️🕵️🕵️ returned from execute block");

        // Seems like we could call the existing collect_collation_info api to get this information here
        // instead of tightly coupling to pallet parachain system
        // let new_validation_code = crate::NewValidationCode::<PSC>::get();
        // let upward_messages = crate::UpwardMessages::<PSC>::get().try_into().expect(
        // 	"Number of upward messages should not be greater than `MAX_UPWARD_MESSAGE_NUM`",
        // );
        // let processed_downward_messages = crate::ProcessedDownwardMessages::<PSC>::get();
        // let horizontal_messages = crate::HrmpOutboundMessages::<PSC>::get().try_into().expect(
        // 	"Number of horizontal messages should not be greater than `MAX_HORIZONTAL_MESSAGE_NUM`",
        // );
        // let hrmp_watermark = crate::HrmpWatermark::<PSC>::get();

        // let head_data =
        // 	if let Some(custom_head_data) = crate::CustomValidationHeadData::<PSC>::get() {
        // 		HeadData(custom_head_data)
        // 	} else {
        // 		head_data
        // 	};

        // Get the relay parent number out of storage so we can advance the hrmp watermark
        let hrmp_watermark = RelayParentNumberStorage::get();

        ValidationResult {
            head_data,
            new_validation_code: None, //new_validation_code.map(Into::into),
            upward_messages: Default::default(),
            processed_downward_messages: 0,
            horizontal_messages: Default::default(),
            hrmp_watermark,
        }
    })
}

/// Extract the [`ParachainInherentData`] from a parachain block.
/// The data has to be extracted from the extrinsics themselves.
/// I want the runtime to expose a method to do this, and I also want it to
/// be nice and flexible by searching for the right transactions.
/// For now I have a hacky implementation that assumes the parachain inherent is first
fn extract_parachain_inherent_data<B, V, C>(block: &B) -> ParachainInherentData
where
    B: BlockT<Extrinsic = Transaction<V, C>>,
    V: Verifier,
    C: ConstraintChecker<V>,
{
    // The commented stuff is Basti's algo.
    // It is nicer than my hack because it searches the transactions,
    // But it is still not good enough because it lived right here in this file as
    // opposed to with the runtime.

    // block
    // 	.extrinsics()
    // 	.iter()
    // 	// Inherents are at the front of the block and are unsigned.
    // 	//
    // 	// If `is_signed` is returning `None`, we keep it safe and assume that it is "signed".
    // 	// We are searching for unsigned transactions anyway.
    // 	.take_while(|e| !e.is_signed().unwrap_or(true))
    // 	.filter_map(|e| e.call().is_sub_type())
    // 	.find_map(|c| match c {
    // 		crate::Call::set_validation_data { data: validation_data } => Some(validation_data),
    // 		_ => None,
    // 	})
    // 	.expect("Could not find `set_validation_data` inherent")

    block
        .extrinsics()
        .get(0)
        .expect("There should be  at least one extrinsic.")
        .outputs
        .get(0)
        .expect("Parachain inherent should be first and should have exactly one output.")
        .payload
        .extract::<ParachainInherentDataUtxo>()
        .expect("Should decode to proper type based on the position in the block.")
        .into()
}

/// Validate the given [`PersistedValidationData`] against the [`MemoryOptimizedValidationParams`].
fn validate_validation_data(
    validation_data: &PersistedValidationData,
    relay_parent_number: RelayChainBlockNumber,
    relay_parent_storage_root: RHash,
    parent_head: bytes::Bytes,
) {
    assert_eq!(
        parent_head, validation_data.parent_head.0,
        "Parent head doesn't match"
    );
    assert_eq!(
        relay_parent_number, validation_data.relay_parent_number,
        "Relay parent number doesn't match",
    );
    assert_eq!(
        relay_parent_storage_root, validation_data.relay_parent_storage_root,
        "Relay parent storage root doesn't match",
    );
}

/// Run the given closure with the externalities set.
fn run_with_externalities<B: BlockT, R, F: FnOnce() -> R>(
    backend: &TrieBackend<B>,
    execute: F,
) -> R {
    let mut overlay = sp_state_machine::OverlayedChanges::default();
    let mut ext = Ext::<B>::new(&mut overlay, backend);

    set_and_run_with_externalities(&mut ext, || execute())
}

fn host_storage_read(key: &[u8], value_out: &mut [u8], value_offset: u32) -> Option<u32> {
    match with_externalities(|ext| ext.storage(key)) {
        Some(value) => {
            let value_offset = value_offset as usize;
            let data = &value[value_offset.min(value.len())..];
            let written = sp_std::cmp::min(data.len(), value_out.len());
            value_out[..written].copy_from_slice(&data[..written]);
            Some(value.len() as u32)
        }
        None => None,
    }
}

fn host_storage_set(key: &[u8], value: &[u8]) {
    with_externalities(|ext| ext.place_storage(key.to_vec(), Some(value.to_vec())))
}

fn host_storage_get(key: &[u8]) -> Option<bytes::Bytes> {
    with_externalities(|ext| ext.storage(key).map(|value| value.into()))
}

fn host_storage_exists(key: &[u8]) -> bool {
    with_externalities(|ext| ext.exists_storage(key))
}

fn host_storage_clear(key: &[u8]) {
    with_externalities(|ext| ext.place_storage(key.to_vec(), None))
}

fn host_storage_root(version: StateVersion) -> Vec<u8> {
    with_externalities(|ext| ext.storage_root(version))
}

fn host_storage_clear_prefix(prefix: &[u8], limit: Option<u32>) -> KillStorageResult {
    with_externalities(|ext| ext.clear_prefix(prefix, limit, None).into())
}

fn host_storage_append(key: &[u8], value: Vec<u8>) {
    with_externalities(|ext| ext.storage_append(key.to_vec(), value))
}

fn host_storage_next_key(key: &[u8]) -> Option<Vec<u8>> {
    with_externalities(|ext| ext.next_storage_key(key))
}

fn host_storage_start_transaction() {
    with_externalities(|ext| ext.storage_start_transaction())
}

fn host_storage_rollback_transaction() {
    with_externalities(|ext| ext.storage_rollback_transaction().ok())
        .expect("No open transaction that can be rolled back.");
}

fn host_storage_commit_transaction() {
    with_externalities(|ext| ext.storage_commit_transaction().ok())
        .expect("No open transaction that can be committed.");
}

fn host_default_child_storage_get(storage_key: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.child_storage(&child_info, key))
}

fn host_default_child_storage_read(
    storage_key: &[u8],
    key: &[u8],
    value_out: &mut [u8],
    value_offset: u32,
) -> Option<u32> {
    let child_info = ChildInfo::new_default(storage_key);
    match with_externalities(|ext| ext.child_storage(&child_info, key)) {
        Some(value) => {
            let value_offset = value_offset as usize;
            let data = &value[value_offset.min(value.len())..];
            let written = sp_std::cmp::min(data.len(), value_out.len());
            value_out[..written].copy_from_slice(&data[..written]);
            Some(value.len() as u32)
        }
        None => None,
    }
}

fn host_default_child_storage_set(storage_key: &[u8], key: &[u8], value: &[u8]) {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| {
        ext.place_child_storage(&child_info, key.to_vec(), Some(value.to_vec()))
    })
}

fn host_default_child_storage_clear(storage_key: &[u8], key: &[u8]) {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.place_child_storage(&child_info, key.to_vec(), None))
}

fn host_default_child_storage_storage_kill(
    storage_key: &[u8],
    limit: Option<u32>,
) -> KillStorageResult {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.kill_child_storage(&child_info, limit, None).into())
}

fn host_default_child_storage_exists(storage_key: &[u8], key: &[u8]) -> bool {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.exists_child_storage(&child_info, key))
}

fn host_default_child_storage_clear_prefix(
    storage_key: &[u8],
    prefix: &[u8],
    limit: Option<u32>,
) -> KillStorageResult {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| {
        ext.clear_child_prefix(&child_info, prefix, limit, None)
            .into()
    })
}

fn host_default_child_storage_root(storage_key: &[u8], version: StateVersion) -> Vec<u8> {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.child_storage_root(&child_info, version))
}

fn host_default_child_storage_next_key(storage_key: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    let child_info = ChildInfo::new_default(storage_key);
    with_externalities(|ext| ext.next_child_storage_key(&child_info, key))
}

fn host_offchain_index_set(_key: &[u8], _value: &[u8]) {}

fn host_offchain_index_clear(_key: &[u8]) {}
