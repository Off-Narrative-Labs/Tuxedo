//! Strongly typed helper functions for communicating with the Node's
//! RPC endpoint.

use jsonrpsee::{http_client::HttpClient, rpc_params, core::client::ClientT};
use runtime::Block;
use sp_core::H256;
use crate::h256_from_string;


/// Typed helper to get the Node's block hash at a particular height
pub async fn node_get_block_hash(height: u32, client: &HttpClient) -> anyhow::Result<Option<H256>> {
    let params = rpc_params![Some(height)];
    let rpc_response: Option<String> = client.request("chain_getBlockHash", params).await?;
    let maybe_hash = rpc_response.map(|s| h256_from_string(&s).unwrap());
    Ok(maybe_hash)
}

/// Typed helper to get the node's full block at a particular hash
pub async fn node_get_block(hash: H256, client: &HttpClient) -> anyhow::Result<Option<Block>> {
    let s = hex::encode(hash.0);
    let params = rpc_params![s];
    
    let rpc_response: Option<serde_json::Value> = client.request("chain_getBlock", params).await?;

    Ok(
        rpc_response
        .and_then(|value| value.get("block").cloned())
        .and_then(|maybe_block| serde_json::from_value(maybe_block).unwrap_or(None))
    )
}