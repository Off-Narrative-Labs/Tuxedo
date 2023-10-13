//! Strongly typed helper functions for communicating with the Node's
//! RPC endpoint.

use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::{Decode, Encode};
use runtime::{opaque::Block as OpaqueBlock, Block};
use sp_core::H256;

/// Typed helper to get the Node's block hash at a particular height
pub async fn node_get_block_hash(height: u32, client: &HttpClient) -> anyhow::Result<Option<H256>> {
    let params = rpc_params![Some(height)];
    let rpc_response: Option<String> = client.request("chain_getBlockHash", params).await?;
    let maybe_hash = rpc_response.map(|s| crate::h256_from_string(&s).unwrap());
    Ok(maybe_hash)
}

/// Typed helper to get the node's full block at a particular hash
pub async fn node_get_block(hash: H256, client: &HttpClient) -> anyhow::Result<Option<Block>> {
    let s = hex::encode(hash.0);
    let params = rpc_params![s];

    let maybe_rpc_response: Option<serde_json::Value> =
        client.request("chain_getBlock", params).await?;
    let rpc_response = maybe_rpc_response.unwrap();

    let json_opaque_block = rpc_response.get("block").cloned().unwrap();
    let opaque_block: OpaqueBlock = serde_json::from_value(json_opaque_block).unwrap();

    // I need a structured block, not an opaque one. To achieve that, I'll
    // scale encode it, then once again decode it.
    // Feels kind of like a hack, but I honestly don't know what else to do.
    // I don't see any way to get the bytes out of an OpaqueExtrinsic.
    let scale_bytes = opaque_block.encode();

    let structured_block = Block::decode(&mut &scale_bytes[..]).unwrap();

    Ok(Some(structured_block))
}
