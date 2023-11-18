//! Custom GenesisBlockBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.

use crate::{
    types::{Output, OutputRef, Transaction},
    utxo_set, ConstraintChecker, Verifier, EXTRINSIC_KEY,
};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use sc_chain_spec::BuildGenesisBlock;
#[cfg(feature = "std")]
use sc_client_api::backend::{Backend, BlockImportOperation};
#[cfg(feature = "std")]
use sc_executor::RuntimeVersionOf;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use sp_core::traits::CodeExecutor;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Hash as HashT, Header as HeaderT, Zero};
#[cfg(feature = "std")]
use sp_runtime::BuildStorage;
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use sp_storage::Storage;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "std")]
pub struct TuxedoGenesisBlockBuilder<
    'a,
    Block: BlockT,
    B: Backend<Block>,
    E: RuntimeVersionOf + CodeExecutor,
> {
    build_genesis_storage: &'a dyn BuildStorage,
    commit_genesis_state: bool,
    backend: Arc<B>,
    executor: E,
    _phantom: std::marker::PhantomData<Block>,
}

#[cfg(feature = "std")]
impl<'a, Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    TuxedoGenesisBlockBuilder<'a, Block, B, E>
{
    pub fn new(
        build_genesis_storage: &'a dyn BuildStorage,
        commit_genesis_state: bool,
        backend: Arc<B>,
        executor: E,
    ) -> sp_blockchain::Result<Self> {
        Ok(Self {
            build_genesis_storage,
            commit_genesis_state,
            backend,
            executor,
            _phantom: Default::default(),
        })
    }
}

#[cfg(feature = "std")]
impl<'a, Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    BuildGenesisBlock<Block> for TuxedoGenesisBlockBuilder<'a, Block, B, E>
{
    type BlockImportOperation = <B as Backend<Block>>::BlockImportOperation;

    /// Build the genesis block, including the extrinsics found in storage at EXTRINSIC_KEY.
    /// The extrinsics are not checked for validity, nor executed, so the values in storage must be placed manually.
    /// This can be done by using the `assimilate_storage` function.
    fn build_genesis_block(self) -> sp_blockchain::Result<(Block, Self::BlockImportOperation)> {
        // We build it here to gain mutable access to the storage.
        let mut genesis_storage = self
            .build_genesis_storage
            .build_storage()
            .map_err(sp_blockchain::Error::Storage)?;

        let state_version =
            sc_chain_spec::resolve_state_version_from_wasm(&genesis_storage, &self.executor)?;

        let extrinsics = match genesis_storage.top.remove(crate::EXTRINSIC_KEY) {
            Some(v) => <Vec<<Block as BlockT>::Extrinsic>>::decode(&mut &v[..]).unwrap_or_default(),
            None => Vec::new(),
        };

        let extrinsics_root =
            <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::ordered_trie_root(
                extrinsics.iter().map(Encode::encode).collect(),
                state_version,
            );

        let mut op = self.backend.begin_operation()?;
        let state_root =
            op.set_genesis_state(genesis_storage, self.commit_genesis_state, state_version)?;

        let block = Block::new(
            HeaderT::new(
                Zero::zero(),
                extrinsics_root,
                state_root,
                Default::default(),
                Default::default(),
            ),
            extrinsics,
        );

        Ok((block, op))
    }
}

#[derive(Serialize, Deserialize)]
/// The `TuxedoGenesisConfig` struct is used to configure the genesis state of the runtime.
/// It expects the wasm binary and a list of transactions to be included in the genesis block, and stored along with their outputs.
/// They must not contain any inputs or peeks. These transactions will not be validated by the corresponding ConstraintChecker or Verifier.
/// Make sure to pass the inherents before the extrinsics.
pub struct TuxedoGenesisConfig<V, C> {
    // wasm_binary: Vec<u8>,
    genesis_transactions: Vec<Transaction<V, C>>,
}

impl<V, C> TuxedoGenesisConfig<V, C> {
    /// Create a new `TuxedoGenesisConfig` from a WASM binary and a list of transactions.
    /// Make sure to pass the transactions in order: the inherents should be first, then the extrinsics.
    pub fn new(genesis_transactions: Vec<Transaction<V, C>>) -> Self {
        Self {
            // wasm_binary,
            genesis_transactions,
        }
    }

    pub fn get_transaction(&self, i: usize) -> Option<&Transaction<V, C>> {
        self.genesis_transactions.get(i)
    }
}

// This is the method for the new genesis builder api.
// I wonder if it can entirely replace the BuildStorage implementation below?
impl<V, C> TuxedoGenesisConfig<V, C>
where
    V: Verifier,
    C: ConstraintChecker<V>,
    Transaction<V, C>: Encode,
    Output<V>: Encode,
{
    /// Writes all the genesis config stuff to storage.
    pub fn build_storage(&self) {
        // The transactions are stored under a special key.
        sp_io::storage::set(EXTRINSIC_KEY, &self.genesis_transactions.encode());

        for tx in &self.genesis_transactions {
            // Transactions are not actually executed and are not really required to be valid in any way
            // We consider them valid by virtue of being in the genesis block.
            // All outputs are directly written to storage.
            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            for (index, output) in tx.outputs.iter().enumerate() {
                let output_ref = OutputRef {
                    tx_hash,
                    index: index as u32,
                };
                utxo_set::TransparentUtxoSet::store_utxo(output_ref, output)
            }
        }
    }
}

//TODO can this be removed now?? Seems not.
#[cfg(feature = "std")]
impl<V, C> BuildStorage for TuxedoGenesisConfig<V, C>
where
    V: Verifier,
    C: ConstraintChecker<V>,
    Transaction<V, C>: Encode,
    Output<V>: Encode,
{
    /// Assimilate the storage into the genesis block.
    /// This is done by inserting the genesis extrinsics into the genesis block, along with their outputs.
    fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
        // The transactions are stored under a special key.
        storage
            .top
            .insert(EXTRINSIC_KEY.to_vec(), self.genesis_transactions.encode());

        let mut finished_with_opening_inherents = false;

        for tx in self.genesis_transactions.iter() {
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
            // Insert the outputs into the storage.
            let tx_hash = BlakeTwo256::hash_of(&tx.encode());
            for (index, utxo) in tx.outputs.iter().enumerate() {
                let output_ref = OutputRef {
                    tx_hash,
                    index: index as u32,
                };
                storage.top.insert(output_ref.encode(), utxo.encode());
            }
        }

        Ok(())
    }
}
