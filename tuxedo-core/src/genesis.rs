//! Custom GenesisBlockBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.

use crate::{
    ensure,
    types::{Output, OutputRef, Transaction},
    ConstraintChecker, Verifier, EXTRINSIC_KEY,
};
use parity_scale_codec::{Decode, Encode};
use sc_chain_spec::BuildGenesisBlock;
use sc_client_api::backend::{Backend, BlockImportOperation};
use sc_executor::RuntimeVersionOf;
use serde::{Deserialize, Serialize};
use sp_core::{storage::Storage, traits::CodeExecutor};
use sp_runtime::{
    traits::{BlakeTwo256, Block as BlockT, Hash as HashT, Header as HeaderT, Zero},
    BuildStorage,
};
use std::sync::Arc;

pub struct TuxedoGenesisBlockBuilder<
    'a,
    Block: BlockT,
    B: Backend<Block>,
    E: RuntimeVersionOf + CodeExecutor,
> {
    genesis_config: &TuxedoGenesisConfig,
    commit_genesis_state: bool,
    backend: Arc<B>,
    executor: E,
    _phantom: std::marker::PhantomData<Block>,
}

impl<'a, Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    TuxedoGenesisBlockBuilder<'a, Block, B, E>
{
    pub fn new(
        genesis_config: &TuxedoGenesisConfig,
        commit_genesis_state: bool,
        backend: Arc<B>,
        executor: E,
    ) -> sp_blockchain::Result<Self> {
        Ok(Self {
            genesis_config,
            commit_genesis_state,
            backend,
            executor,
            _phantom: Default::default(),
        })
    }
}

impl<'a, Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    BuildGenesisBlock<Block> for TuxedoGenesisBlockBuilder<'a, Block, B, E>
{
    type BlockImportOperation = <B as Backend<Block>>::BlockImportOperation;

    /// Build the genesis block, including the extrinsics from the genesis config.
    fn build_genesis_block(self) -> sp_blockchain::Result<(Block, Self::BlockImportOperation)> {
        
        let genesis_storage = self
            .genesis_config
            .build_storage()
            .map_err(sp_blockchain::Error::Storage)?;

        let genesis_state_version =
            sc_chain_spec::resolve_state_version_from_wasm(&genesis_storage, &self.executor)?;
        
        let mut op = self.backend.begin_operation()?;

        let state_root =
            op.set_genesis_state(genesis_storage, self.commit_genesis_state, genesis_state_version)?;
        

        let extrinsics_root =
            <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::ordered_trie_root(
                self.genesis_config.genesis_transactions.iter().map(Encode::encode).collect(),
                genesis_state_version,
            );

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
    wasm_binary: Vec<u8>,
    genesis_transactions: Vec<Transaction<V, C>>,
}

impl<V, C> TuxedoGenesisConfig<V, C> {
    /// Create a new `TuxedoGenesisConfig` from a WASM binary and a list of transactions.
    /// Make sure to pass the transactions in order: the inherents should be first, then the extrinsics.
    pub fn new(wasm_binary: Vec<u8>, genesis_transactions: Vec<Transaction<V, C>>) -> Self {
        Self {
            wasm_binary,
            genesis_transactions,
        }
    }

    pub fn get_transaction(&self, i: usize) -> Option<&Transaction<V, C>> {
        self.genesis_transactions.get(i)
    }
}

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
        // The wasm binary is stored under a special key.
        storage.top.insert(
            sp_storage::well_known_keys::CODE.into(),
            self.wasm_binary.clone(),
        );

        // I'm not so sure we need this in the genesis builder, but
        // I also don't see any problem with it. Let's leave it for now.
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
            // Enforce that transactions do not have any inputs or peeks.
            ensure!(
                tx.inputs.is_empty() && tx.peeks.is_empty(),
                "Genesis transactions must not have any inputs or peeks."
            );
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
