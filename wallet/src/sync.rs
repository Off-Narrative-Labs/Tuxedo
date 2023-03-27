//! This module is responsible for syncing the wallet's local database of blocks
//! and owned UTXOs to the canonical database reported by the node.

use sp_core::H256;
use super::h256_from_string;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient},
    rpc_params,
};
use anyhow::anyhow;
use runtime::Block;

/// Typed helper to get the Node's block hash at a particular height
pub async fn node_get_block_hash(height: u32, client: &HttpClient) -> anyhow::Result<Option<H256>> {
    let params = rpc_params![Some(height)];
    let rpc_response: Option<String> = client.request("chain_getBlockHash", params).await?;
    let maybe_hash = rpc_response.map(|s| h256_from_string(&s).unwrap());
    Ok(maybe_hash)
}

/// Typed helper to get the node's full block at a particular hash
pub async fn node_get_block(hash: H256, client: &HttpClient) -> anyhow::Result<Block> {
    todo!()
}

