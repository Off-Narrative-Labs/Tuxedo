//! This module is responsible for syncing the wallet's local database of blocks
//! and owned UTXOs to the canonical database reported by the node.
//! 
//! ## Schema
//! 
//! There are 4 tables in the database
//! BlockHashes     block_number:u32 => block_hash:H256
//! Blocks          block_hash:H256 => block:Block
//! UnspentOutputs  output_ref => (owner_pubkey, amount)
//! SpentOutputs    output_ref => (owner_pubkey, amount)

use parity_scale_codec::{Decode, Encode};
use sc_keystore::LocalKeystore;
use sled::{Db};
use sp_core::H256;
use sp_keystore::CryptoStore;
use tuxedo_core::{verifier::SigCheck, types::{OutputRef, Input}};
use sp_runtime::traits::{BlakeTwo256, Hash};
use crate::KEY_TYPE;

use super::h256_from_string;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient},
    rpc_params,
};
use runtime::{Block, Transaction, OuterVerifier, money::Coin, Output};

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
    let tx_hash = BlakeTwo256::hash_of(&tx.encode());
    println!("syncing transaction {tx_hash:?}");

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
                    Ok(Coin(amount)) => amount,
                    Err(_) => continue,
                };

                let output_ref = OutputRef {
                    tx_hash,
                    index: index as u32,
                };

                // Add it to the global unspent_outputs table
                add_unspent_output(db, &output_ref, owner_pubkey, &amount)?;
            }
            v => {
                println!("Not recording output with verifier {v:?}");
            }
        }
    }

    println!("about to spend all inputs");
    // Spend all the inputs
    for Input{ output_ref, .. } in tx.inputs {
        spend_output(db, &output_ref)?;
    }

    Ok(())
}

/// Add a new output to the database updating all tables.
fn add_unspent_output(db: &Db, output_ref: &OutputRef, owner_pubkey: &H256, amount: &u128) -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;
    unspent_tree.insert(output_ref.encode(), (owner_pubkey, amount).encode())?;

    Ok(())
}

/// Remove an output from the database updating all tables.
fn remove_unspent_output(db: &Db, output_ref: &OutputRef)  -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;

    todo!()
}

/// Mark an existing output as spent. This does not purge all record of the output from the db
fn spend_output(db: &Db, output_ref: &OutputRef) -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;
    let spent_tree = db.open_tree("spent_outputs")?;

    let Some(ivec) = unspent_tree.remove(output_ref.encode())? else { return Ok(())};
    let (owner, amount) = <(H256, u128)>::decode(&mut &ivec[..])?;
    spent_tree.insert(output_ref.encode(), (owner, amount).encode())?;

    Ok(())
}

/// Mark an output that was previously marked as spent back as unspent.
fn unspend_output(db: &Db, output_ref: &OutputRef) -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;
    let spent_tree = db.open_tree("spent_outputs")?;

    todo!()
}

/// Docs TODO
fn unapply_transaction() -> anyhow::Result<()> {
    todo!()
}

/// Docs TODO
pub(crate) async fn unapply_block() -> anyhow::Result<()> {
    todo!()
}
