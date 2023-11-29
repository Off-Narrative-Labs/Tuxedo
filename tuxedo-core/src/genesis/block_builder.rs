//! Custom GenesisBlockBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.

use parity_scale_codec::{Decode, Encode};
use sc_chain_spec::BuildGenesisBlock;
use sc_client_api::backend::{Backend, BlockImportOperation};
use sc_executor::RuntimeVersionOf;
use sp_core::traits::CodeExecutor;
use sp_runtime::{
    traits::{Block as BlockT, Hash as HashT, Header as HeaderT, Zero},
    BuildStorage,
};
use std::sync::Arc;

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
