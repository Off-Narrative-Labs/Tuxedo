//! # Executive Module
//! 
//! The executive is the main orchestrator for the entire runtime.
//! It has functions that implement the Core, BlockBuilder, and TxPool runtime APIs.
//! 
//! It does all the reusable verification of UTXO transactions such as checking that there
//! are no duplicate inputs or outputs, and that the redeemers are satisfied.

use std::marker::PhantomData;
use parity_scale_codec::{Encode, Decode};
use sp_api::{HashT, BlockT, HeaderT};
use sp_runtime::{traits::BlakeTwo256, transaction_validity::{ValidTransaction, TransactionLongevity}};
use sp_std::{vec::Vec, collections::btree_set::BTreeSet};
use crate::{utxo_set::TransparentUtxoSet, redeemer::Redeemer, verifier::Verifier, types::{DispatchResult, Transaction, UtxoError, OutputRef, TypedData}, ensure, fail};

/// The executive. Each runtime is encouraged to make a type alias called `Executive` that fills
/// in the proper generic types.
pub struct Executive<B, R, V>(PhantomData<(B, R, V)>);

impl <
    B: BlockT,
    R: Redeemer,
    V: Verifier,
> Executive<B, R, V> {
    /// Does pool-style validation of a tuxedo transaction.
    /// Does not commit anything to storage.
    /// This returns Ok even if some inputs are still missing because the tagged transaction pool can handle that.
    /// We later check that there are no missing inputs in `apply_tuxedo_transaction`
    pub fn validate_tuxedo_transaction(
        transaction: &Transaction<R, V>,
    ) -> Result<ValidTransaction, UtxoError<V::Error>> {
        // Make sure there are no duplicate inputs
        {
            let input_set: BTreeSet<_> = transaction.outputs.iter().map(|o| o.encode()).collect();
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
        // TODO Actually I don't think we need to do this. It _does_ appear in the original utxo workshop,
        // but I don't see how we could ever have an output collision.
        for index in 0..transaction.outputs.len() {
            let output_ref = OutputRef {
                tx_hash: BlakeTwo256::hash_of(&transaction.encode()),
                index: index as u32,
            };

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
        let input_data: Vec<TypedData> = transaction
            .inputs
            .iter()
            .map(|i| {
                TransparentUtxoSet::<R>::peek_utxo(&i.output_ref)
                    .expect("We just checked that all inputs were present.")
                    .payload
            })
            .collect();
        let output_data: Vec<TypedData> = transaction
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
        // log::debug!(
        //     target: LOG_TARGET,
        //     "applying tuxedo transaction {:?}",
        //     transaction
        // );

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
        where B::Header: HeaderT
    {
        //TODO this is copied from lib.rs. Figure out the right separation between
        // tuxedo core and the runtime template
        const HEADER_KEY: &[u8] = b"header";

        //TODO The header type is also copied.
        *sp_io::storage::get(HEADER_KEY)
            .and_then(|d| B::Header::decode(&mut &*d).ok())
            .expect("A header is always stored at the beginning of the block")
            .number()
    }
}