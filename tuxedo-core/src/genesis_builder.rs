//! Custom GenesisBlockBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.

use parity_scale_codec::{Decode, Encode};
use sc_client_api::backend::{Backend, BlockImportOperation};
use sc_executor::RuntimeVersionOf;
use sc_service::BuildGenesisBlock;
use sp_core::{storage::Storage, traits::CodeExecutor};
use sp_runtime::{
    traits::{Block as BlockT, Hash as HashT, Header as HeaderT, Zero},
    BuildStorage,
};
use std::sync::Arc;

pub struct TuxedoGenesisBlockBuilder<
    Block: BlockT,
    B: Backend<Block>,
    E: RuntimeVersionOf + CodeExecutor,
> {
    genesis_storage: Storage,
    commit_genesis_state: bool,
    backend: Arc<B>,
    executor: E,
    _phantom: std::marker::PhantomData<Block>,
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    TuxedoGenesisBlockBuilder<Block, B, E>
{
    pub fn new(
        build_genesis_storage: &dyn BuildStorage,
        commit_genesis_state: bool,
        backend: Arc<B>,
        executor: E,
    ) -> sp_blockchain::Result<Self> {
        let genesis_storage = build_genesis_storage
            .build_storage()
            .map_err(sp_blockchain::Error::Storage)?;

        Ok(Self {
            genesis_storage,
            commit_genesis_state,
            backend,
            executor,
            _phantom: Default::default(),
        })
    }
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor> BuildGenesisBlock<Block>
    for TuxedoGenesisBlockBuilder<Block, B, E>
{
    type BlockImportOperation = <B as Backend<Block>>::BlockImportOperation;

    fn build_genesis_block(self) -> sp_blockchain::Result<(Block, Self::BlockImportOperation)> {
        let state_version =
            sc_service::resolve_state_version_from_wasm(&self.genesis_storage, &self.executor)?;

        let extrinsics = match self.genesis_storage.top.get(crate::EXTRINSIC_KEY) {
            Some(v) => <Vec<<Block as BlockT>::Extrinsic>>::decode(&mut &v[..]).unwrap_or_default(),
            None => Vec::new(),
        };

        // TODO: clear extrinsic key

        let extrinsics_root =
            <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::ordered_trie_root(
                extrinsics.iter().map(Encode::encode).collect(),
                state_version,
            );

        let mut op = self.backend.begin_operation()?;
        let state_root = op.set_genesis_state(
            self.genesis_storage,
            self.commit_genesis_state,
            state_version,
        )?;

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
