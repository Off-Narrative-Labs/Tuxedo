//! # Executive Module
//!
//! The executive is the main orchestrator for the entire runtime.
//! It has functions that implement the Core, BlockBuilder, and TxPool runtime APIs.
//!
//! It does all the reusable verification of UTXO transactions such as checking that there
//! are no duplicate inputs or outputs, and that the redeemers are satisfied.

use crate::{
    dynamic_typing::DynamicallyTypedData,
    ensure,
    redeemer::Redeemer,
    types::{DispatchResult, OutputRef, Transaction, UtxoError},
    utxo_set::TransparentUtxoSet,
    verifier::Verifier,
    EXTRINSIC_KEY, HEADER_KEY, LOG_TARGET,
};
use log::info;
use parity_scale_codec::{Decode, Encode};
use sp_api::{BlockT, HashT, HeaderT, TransactionValidity};
use sp_runtime::{
    traits::BlakeTwo256,
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionSource, TransactionValidityError,
        ValidTransaction,
    },
    ApplyExtrinsicResult, StateVersion,
};
use sp_std::marker::PhantomData;
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

/// The executive. Each runtime is encouraged to make a type alias called `Executive` that fills
/// in the proper generic types.
pub struct Executive<B, R, V>(PhantomData<(B, R, V)>);

impl<B: BlockT<Extrinsic = Transaction<R, V>>, R: Redeemer, V: Verifier> Executive<B, R, V> {
    /// Does pool-style validation of a tuxedo transaction.
    /// Does not commit anything to storage.
    /// This returns Ok even if some inputs are still missing because the tagged transaction pool can handle that.
    /// We later check that there are no missing inputs in `apply_tuxedo_transaction`
    pub fn validate_tuxedo_transaction(
        transaction: &Transaction<R, V>,
    ) -> Result<ValidTransaction, UtxoError<V::Error>> {
        // Make sure there are no duplicate inputs
        {
            let input_set: BTreeSet<_> = transaction.inputs.iter().map(|o| o.encode()).collect();
            ensure!(
                input_set.len() == transaction.inputs.len(),
                UtxoError::DuplicateInput
            );
        }

        // Make sure there are no duplicate outputs
        {
            let output_set: BTreeSet<_> = transaction.outputs.iter().map(|o| o.encode()).collect();
            ensure!(
                output_set.len() == transaction.outputs.len(),
                UtxoError::DuplicateOutput
            );
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
            if let Some(input_utxo) = TransparentUtxoSet::<R>::peek_utxo(&input.output_ref) {
                ensure!(
                    input_utxo
                        .redeemer
                        .redeem(&stripped_encoded, &input.witness),
                    UtxoError::RedeemerError
                );
            } else {
                missing_inputs.push(input.output_ref.clone().encode());
            }
        }

        // Make sure no outputs already exist in storage
        let tx_hash = BlakeTwo256::hash_of(&transaction.encode());
        for index in 0..transaction.outputs.len() {
            let output_ref = OutputRef {
                tx_hash,
                index: index as u32,
            };

            log::debug!(
                target: LOG_TARGET,
                "Checking for pre-existing output {:?}",
                output_ref
            );

            ensure!(
                !TransparentUtxoSet::<R>::peek_utxo(&output_ref).is_some(),
                UtxoError::PreExistingOutput
            );
        }

        // If any the inputs are missing, we cannot make any more progress
        // If they are all present, we may proceed to call the verifier
        if !missing_inputs.is_empty() {
            return Ok(ValidTransaction {
                requires: missing_inputs,
                provides: transaction.outputs.iter().map(|o| o.encode()).collect(),
                priority: 0,
                longevity: TransactionLongevity::max_value(),
                propagate: true,
            });
        }

        // Extract the contained data from each input and output
        // We do not yet remove anything from the utxo set. That will happen later
        // iff verification passes
        let input_data: Vec<DynamicallyTypedData> = transaction
            .inputs
            .iter()
            .map(|i| {
                TransparentUtxoSet::<R>::peek_utxo(&i.output_ref)
                    .expect("We just checked that all inputs were present.")
                    .payload
            })
            .collect();
        let output_data: Vec<DynamicallyTypedData> = transaction
            .outputs
            .iter()
            .map(|o| o.payload.clone())
            .collect();

        // Call the verifier
        transaction
            .verifier
            .verify(&input_data, &output_data)
            .map_err(|e| UtxoError::VerifierError(e))?;

        // Return the valid transaction
        // TODO in the future we need to prioritize somehow. Perhaps the best strategy
        // is to have the verifier return a priority
        Ok(ValidTransaction {
            requires: Vec::new(),
            provides: transaction.outputs.iter().map(|o| o.encode()).collect(),
            priority: 0,
            longevity: TransactionLongevity::max_value(),
            propagate: true,
        })
    }

    /// Does full verification and application of tuxedo transactions.
    /// Most of the validation happens in the call to `validate_tuxedo_transaction`.
    /// Once those chekcs are done we make sure there are no missing inputs and then update storage.
    pub fn apply_tuxedo_transaction(transaction: Transaction<R, V>) -> DispatchResult<V::Error> {
        log::debug!(
            target: LOG_TARGET,
            "applying tuxedo transaction {:?}",
            transaction
        );

        // Re-do the pre-checks. These should have been done in the pool, but we can't
        // guarantee that foreign nodes to these checks faithfully, so we need to check on-chain.
        let valid_transaction = Self::validate_tuxedo_transaction(&transaction)?;

        // If there are still missing inputs, so we cannot execute this,
        // although it would be valid in the pool
        ensure!(
            valid_transaction.requires.is_empty(),
            UtxoError::MissingInput
        );

        // At this point, all validation is complete, so we can commit the storage changes.
        Self::update_storage(transaction);

        Ok(())
    }

    /// Helper function to update the utxo set according to the given transaction.
    /// This function does absolutely no validation. It assumes that the transaction
    /// has already passed validation. Changes proposed by the transaction are written
    /// blindly to storage.
    fn update_storage(transaction: Transaction<R, V>) {
        // Remove redeemed UTXOs
        for input in &transaction.inputs {
            TransparentUtxoSet::<R>::consume_utxo(&input.output_ref);
        }

        // Write the newly created utxos
        for (index, output) in transaction.outputs.iter().enumerate() {
            let output_ref = OutputRef {
                tx_hash: BlakeTwo256::hash_of(&transaction.encode()),
                index: index as u32,
            };
            TransparentUtxoSet::<R>::store_utxo(output_ref, output);
        }
    }

    /// A helper function that allows tuxedo runtimes to read the current block height
    /// TODO This must be exposed to the pieces somehow. We need some kind of config system.
    /// We should probably steal it from FRAME a la https://github.com/paritytech/substrate/pull/104
    pub fn block_height() -> <<B as BlockT>::Header as HeaderT>::Number
    where
        B::Header: HeaderT,
    {
        *sp_io::storage::get(crate::HEADER_KEY)
            .and_then(|d| B::Header::decode(&mut &*d).ok())
            .expect("A header is always stored at the beginning of the block")
            .number()
    }

    // These next three methods are for the block authoring workflow.
    // Open the block, apply zero or more extrinsics, close the block

    pub fn open_block(header: &<B as BlockT>::Header) {
        info!(
            target: LOG_TARGET,
            "Entering initialize_block. header: {:?}", header
        );

        // Store the transient partial header for updating at the end of the block.
        // This will be removed from storage before the end of the block.
        sp_io::storage::set(&HEADER_KEY, &header.encode());
    }

    pub fn apply_extrinsic(extrinsic: <B as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
        info!(
            target: LOG_TARGET,
            "Entering apply_extrinsic: {:?}", extrinsic
        );

        // Append the current extrinsic to the transient list of extrinsics.
        // This will be used when we calculate the extrinsics root at the end of the block.
        let mut extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
            .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
            .unwrap_or_default();
        extrinsics.push(extrinsic.encode());
        sp_io::storage::set(EXTRINSIC_KEY, &extrinsics.encode());

        // Now actually
        Self::apply_tuxedo_transaction(extrinsic)
            .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(0)))?;

        Ok(Ok(()))
    }

    pub fn close_block() -> <B as BlockT>::Header {
        let mut header = sp_io::storage::get(HEADER_KEY)
            .and_then(|d| <B as BlockT>::Header::decode(&mut &*d).ok())
            .expect("We initialized with header, it never got mutated, qed");

        // the header itself contains the state root, so it cannot be inside the state (circular
        // dependency..). Make sure in execute block path we have the same rule.
        sp_io::storage::clear(&HEADER_KEY);

        let extrinsics = sp_io::storage::get(EXTRINSIC_KEY)
            .and_then(|d| <Vec<Vec<u8>>>::decode(&mut &*d).ok())
            .unwrap_or_default();
        let extrinsics_root = <<B as BlockT>::Header as HeaderT>::Hashing::ordered_trie_root(
            extrinsics,
            StateVersion::V1,
        );
        sp_io::storage::clear(&EXTRINSIC_KEY);
        header.set_extrinsics_root(extrinsics_root);

        let raw_state_root = &sp_io::storage::root(StateVersion::V1)[..];
        let state_root =
            <<B as BlockT>::Header as HeaderT>::Hash::decode(&mut &raw_state_root[..]).unwrap();
        header.set_state_root(state_root);

        info!(target: LOG_TARGET, "finalizing block {:?}", header);
        header
    }

    // This one is for the Core api. It is used to import blocks authored by foreign nodes.

    pub fn execute_block(block: B) {
        info!(
            target: LOG_TARGET,
            "Entering execute_block. block: {:?}", block
        );

        // Store the header. Although we don't need to mutate it, we do need to make
        // info, such as the block height, available to individual pieces. This will
        // be cleared before the end of the block
        sp_io::storage::set(&HEADER_KEY, &block.header().encode());

        // Apply each extrinsic
        for extrinsic in block.clone().extrinsics() {
            match Self::apply_tuxedo_transaction(extrinsic.clone()) {
                Ok(()) => info!(
                    target: LOG_TARGET,
                    "Successfully executed extrinsic: {:?}", extrinsic
                ),
                Err(e) => panic!("{:?}", e),
            }
        }

        // Clear the transient header out of storage
        sp_io::storage::clear(&HEADER_KEY);

        // Check state root
        let raw_state_root = &sp_io::storage::root(StateVersion::V1)[..];
        let state_root =
            <<B as BlockT>::Header as HeaderT>::Hash::decode(&mut &raw_state_root[..]).unwrap();
        assert_eq!(*block.header().state_root(), state_root);

        // Print state for quick debugging
        // let mut key = vec![];
        // while let Some(next) = sp_io::storage::next_key(&key) {
        //     let val = sp_io::storage::get(&next).unwrap().to_vec();
        //     log::trace!(
        //         target: LOG_TARGET,
        //         "{} <=> {}",
        //         HexDisplay::from(&next),
        //         HexDisplay::from(&val)
        //     );
        //     key = next;
        // }

        // Check extrinsics root.
        let extrinsics = block
            .extrinsics()
            .into_iter()
            .map(|x| x.encode())
            .collect::<Vec<_>>();
        let extrinsics_root = <<B as BlockT>::Header as HeaderT>::Hashing::ordered_trie_root(
            extrinsics,
            StateVersion::V1,
        );
        assert_eq!(*block.header().extrinsics_root(), extrinsics_root);
    }

    // This one is the pool api. It is used to make preliminary checks in the transaction pool

    pub fn validate_transaction(
        source: TransactionSource,
        tx: <B as BlockT>::Extrinsic,
        block_hash: <B as BlockT>::Hash,
    ) -> TransactionValidity {
        log::debug!(
            target: LOG_TARGET,
            "Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}",
            source,
            tx,
            block_hash
        );

        // TODO, we need a good way to map our UtxoError into the supposedly generic InvalidTransaction
        // https://paritytech.github.io/substrate/master/sp_runtime/transaction_validity/enum.InvalidTransaction.html
        // For now, I just make them all custom zero
        let r = Self::validate_tuxedo_transaction(&tx);

        log::debug!(
            target: LOG_TARGET,
            "Validation result: {:?}",
            r
        );

        r.map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Custom(0)))
    }
}
