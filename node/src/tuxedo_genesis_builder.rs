/// Custom GenesisBlockBuilder for Tuxedo, to allow extrinsics to be added to the genesis block.
use parity_scale_codec::Encode;
use sc_chain_spec::BuildGenesisBlock;
use sc_client_api::backend::Backend;
use sc_executor::RuntimeVersionOf;
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
    pub extrinsics: Vec<<Block as BlockT>::Extrinsic>,
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor>
    TuxedoGenesisBlockBuilder<Block, B, E>
{
    pub fn new(
        build_genesis_storage: &dyn BuildStorage,
        commit_genesis_state: bool,
        backend: Arc<B>,
        executor: E,
        extrinsics: Vec<<Block as BlockT>::Extrinsic>,
    ) -> sp_blockchain::Result<Self> {
        let genesis_storage = build_genesis_storage
            .build_storage()
            .map_err(sp_blockchain::Error::Storage)?;

        Ok(Self {
            genesis_storage,
            commit_genesis_state,
            backend,
            executor,
            extrinsics,
        })
    }

    fn extrinsics_root(&self) -> sp_blockchain::Result<<Block as BlockT>::Hash> {
        let state_version =
            sc_chain_spec::resolve_state_version_from_wasm(&self.genesis_storage, &self.executor)?;

        Ok(
            <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::ordered_trie_root(
                self.extrinsics.iter().map(Encode::encode).collect(),
                state_version,
            ),
        )
    }

    fn state_root(&self) -> Block::Hash {
        todo!()
    }
}

impl<Block: BlockT, B: Backend<Block>, E: RuntimeVersionOf + CodeExecutor> BuildGenesisBlock<Block>
    for TuxedoGenesisBlockBuilder<Block, B, E>
{
    type BlockImportOperation = <B as Backend<Block>>::BlockImportOperation;

    fn build_genesis_block(self) -> sp_blockchain::Result<(Block, Self::BlockImportOperation)> {
        let block = Block::new(
            HeaderT::new(
                Zero::zero(),
                self.extrinsics_root()?,
                self.state_root(),
                Default::default(),
                Default::default(),
            ),
            self.extrinsics,
        );

        todo!()
    }
}
