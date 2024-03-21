//! Strongly typed helper functions for communicating with the Node's
//! RPC endpoint.

use crate::strip_0x_prefix;
use anyhow::anyhow;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::{Decode, Encode};
use runtime::{opaque::Block as OpaqueBlock};
use sp_core::H256;
use sp_runtime::generic::Block;
use tuxedo_core::{
    types::{Output, OutputRef, Transaction}, ConstraintChecker, Verifier
};

/// Typed helper to get the Node's block hash at a particular height
pub async fn node_get_block_hash(height: u32, client: &HttpClient) -> anyhow::Result<Option<H256>> {
    let params = rpc_params![Some(height)];
    let rpc_response: Option<String> = client.request("chain_getBlockHash", params).await?;
    let maybe_hash = rpc_response.map(|s| crate::h256_from_string(&s).unwrap());
    Ok(maybe_hash)
}

/// Typed helper to get the node's full block at a particular hash
pub async fn node_get_block<Block: Decode>(hash: H256, client: &HttpClient) -> anyhow::Result<Option<Block>> {
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

/// Fetch an output from chain storage given an OutputRef
pub async fn fetch_storage<V: Verifier>(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<Output<V>> {
    let ref_hex = hex::encode(output_ref.encode());
    let params = rpc_params![ref_hex];
    let rpc_response: Result<Option<String>, _> = client.request("state_getStorage", params).await;

    let response_hex = rpc_response?.ok_or(anyhow!("Data cannot be retrieved from storage"))?;
    let response_hex = strip_0x_prefix(&response_hex);
    let response_bytes = hex::decode(response_hex)?;
    let utxo = Output::decode(&mut &response_bytes[..])?;

    Ok(utxo)
}
