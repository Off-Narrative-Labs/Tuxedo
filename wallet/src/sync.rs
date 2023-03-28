//! This module is responsible for syncing the wallet's local database of blocks
//! and owned UTXOs to the canonical database reported by the node.

use parity_scale_codec::{Decode, Encode};
use sc_keystore::LocalKeystore;
use sled::{Db};
use sp_core::H256;
use sp_keystore::CryptoStore;
use tuxedo_core::{verifier::SigCheck, types::{OutputRef, Input}};
use crate::KEY_TYPE;

use super::h256_from_string;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient},
    rpc_params,
};
use runtime::{Block, Transaction, OuterVerifier, money::Coin};

//TODO this type alias should be public in the Runtime
type Output = tuxedo_core::types::Output<OuterVerifier>;

/// Typed helper to get the Node's block hash at a particular height
pub async fn node_get_block_hash(height: u32, client: &HttpClient) -> anyhow::Result<Option<H256>> {
    let params = rpc_params![Some(height)];
    let rpc_response: Option<String> = client.request("chain_getBlockHash", params).await?;
    let maybe_hash = rpc_response.map(|s| h256_from_string(&s).unwrap());
    Ok(maybe_hash)
}

/// Apply a block to the local database
pub(crate) async fn apply_block(db: &Db, b: Block, block_hash: H256, keystore: &LocalKeystore) -> anyhow::Result<()> {
    // Write the hash to the blocks table
    let wallet_blocks_tree = db.open_tree("blocks").expect("should be able to open blocks tree from sled db.");
    wallet_blocks_tree.insert(b.header.number.encode(), block_hash.encode())?;

    // Iterate through each transaction
    for tx in b.extrinsics {
        apply_transaction(db, tx, keystore).await?;
    }

    Ok(())

}

/// Apply a single transaction to the local database
/// The owner-specific tables are mappings from output_refs to coin amounts
async fn apply_transaction(db: &Db, tx: Transaction, keystore: &LocalKeystore) -> anyhow::Result<()> {
    let tx_hash = H256::zero();//TODO calculate this.
    println!("syncing transaction {tx_hash:?}");

    let unspent_tree = db.open_tree("unspent_outputs")?;
    let spent_tree = db.open_tree("spent_outputs")?;

    println!("about to insert new outputs");

    // Insert all new outputs
    for (index, output) in tx.outputs.iter().enumerate() {
        match output {
            Output {
                verifier: OuterVerifier::SigCheck(SigCheck{owner_pubkey}),
                payload,
            } if keystore.has_keys(&[(owner_pubkey.encode(), KEY_TYPE)]).await => {

                // For now the wallet only supports simple coins, so skip anything else
                let amount = match payload.extract::<Coin>() {
                    Ok(amount) => amount,
                    Err(_) => continue,
                };

                let output_ref = OutputRef {
                    tx_hash,
                    index: index as u32,
                };
                
                // Add it to the user's personal mapping
                let user_tree = db.open_tree(owner_pubkey)?;
                user_tree.insert(output_ref.encode(), amount.encode())?;

                // Add it to the global unspent_outputs table
                unspent_tree.insert(output_ref.encode(), (owner_pubkey, amount).encode())?;
            }
            v => {
                println!("Not recording output with verifier {v:?}");
            }
        }
    }

    println!("about to spend all inputs");
    // Spend all the inputs
    for Input{ output_ref, .. } in tx.inputs {
        let Some(ivec) = unspent_tree.remove(output_ref.encode())? else { continue };

        let (owner, amount) = <(H256, u128)>::decode(&mut &ivec[..])?;

        spent_tree.insert(output_ref.encode(), (owner, amount).encode())?;

        let user_tree = db.open_tree(owner)?;
        user_tree.remove(output_ref.encode())?;

    }

    Ok(())
}

/// Add a new output to the database updating all tables.
fn add_new_output(output_ref: OutputRef, ) {

}

/// Remove an output from the database updating all tables.
fn remove_unspent_output() {

}

/// Mark an existing output as spent. This does not purge all record of the output from the db
fn spend_output() {

}

/// Mark an output that was previously marked as spent back as unspent.
fn unspend_output() {

}

/// Docs TODO
fn unapply_transaction() {

}

/// Docs TODO
fn unapply_block() {

}

/// Typed helper to get the node's full block at a particular hash
pub async fn node_get_block(hash: H256, client: &HttpClient) -> anyhow::Result<Option<Block>> {
    println!("in node get block with hash {hash:?}");
    let s = hex::encode(hash.0.encode());
    println!("s in {s}");
    let params = rpc_params![s];
    println!("about to send request for block with params {params:?}");
    
    // TODO WTF!? Why isnt this RPC call working? This callworks on the cli both with and without the 0x prefix
    // curl http://127.0.0.1:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0", "id":1, "method":"chain_getBlock", "params": ["0x505914d65fcf1049e630441127ca8cba338ff7f048a06508e2f987514125d481"]}'
    // {"jsonrpc":"2.0","result":{"block":{"header":{"parentHash":"0x3b761bd2ff69f4fe670c0f0911b6de1d83ee615830aed0693421f9d95fcde6e3","number":"0x1","stateRoot":"0x55a6d4664a0bccd89e68e60b7de8644121ce03094fb0caa13a84490ab396191c","extrinsicsRoot":"0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314","digest":{"logs":["0x0661757261203ac5602100000000","0x05617572610101c6dcdb663e80f4d02c07f0d2266b24339a514e3e41c155e65eae978ec051e64d1978f647c12a6c2022feb2b9e2c7e1959ed1fb849b1cb2294dab729f4942248e"]}},"extrinsics":[]},"justifications":null},"id":1}

    let rpc_response: Result<Option<String>, _> = client.request("chain_getBlock", params).await;


    match rpc_response {
        Ok(Some(s)) => {
            println!("ok some {s}");
            let bytes = hex::decode(s).expect("Chain should provide valid hex to decode.");
            let b =Block::decode(&mut &bytes[..]).expect("scale decoding of block should work");
            Ok(Some(b))
        },
        Ok(None) => {println!("ok none"); Ok(None)},
        Err(e) => {println!("%%%Error: {e:?}"); panic!()},
    }
}

