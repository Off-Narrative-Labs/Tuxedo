//! Custom GenesisConfigBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.

use crate::{
    ensure,
    types::{OutputRef, Transaction},
    ConstraintChecker, Verifier, EXTRINSIC_KEY, HEIGHT_KEY,
};
use parity_scale_codec::Encode;
use sp_runtime::traits::Hash as HashT;
use sp_std::vec::Vec;

pub struct TuxedoGenesisConfigBuilder<V, C>(sp_std::marker::PhantomData<(V, C)>);

impl<V, C> TuxedoGenesisConfigBuilder<V, C>
where
    V: Verifier,
    C: ConstraintChecker,
    Transaction<V, C>: Encode,
{
    /// This function expects a list of transactions to be included in the genesis block,
    /// and stored along with their outputs. They must not contain any inputs or peeks.
    /// The input transactions must be ordered: inherents first, then extrinsics.
    /// The genesis transactions will not be validated by the corresponding ConstraintChecker or Verifier.
    pub fn build(genesis_transactions: Vec<Transaction<V, C>>) -> sp_genesis_builder::Result {
        // The transactions are stored under a special key.
        sp_io::storage::set(EXTRINSIC_KEY, &genesis_transactions.encode());

        //TODO This was added in during merge conflicts. Make sure inherents are working even in real parachains.
        // Initialize the stored block number to 0
        sp_io::storage::set(HEIGHT_KEY, &0u32.encode());

        let mut finished_with_opening_inherents = false;

        for tx in genesis_transactions.into_iter() {
            // Enforce that inherents are in the right place
            let current_tx_is_inherent = tx.checker.is_inherent();
            if current_tx_is_inherent && finished_with_opening_inherents {
                return Err(
                    "Tried to execute opening inherent after switching to non-inherents.".into(),
                );
            }
            if !current_tx_is_inherent && !finished_with_opening_inherents {
                // This is the first non-inherent, so we update our flag and continue.
                finished_with_opening_inherents = true;
            }
            // Enforce that transactions do not have any inputs or peeks.
            ensure!(
                tx.inputs.is_empty() && tx.peeks.is_empty(),
                "Genesis transactions must not have any inputs or peeks."
            );
            // Insert the outputs into the storage.
            let tx_hash = sp_runtime::traits::BlakeTwo256::hash_of(&tx.encode());
            for (index, utxo) in tx.outputs.iter().enumerate() {
                let output_ref = OutputRef {
                    tx_hash,
                    index: index as u32,
                };
                sp_io::storage::set(&output_ref.encode(), &utxo.encode());
            }
        }

        Ok(())
    }
}
