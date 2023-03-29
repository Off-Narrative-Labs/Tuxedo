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

use std::path::PathBuf;

use anyhow::anyhow;
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

/// Open a database at the given location intended for the given genesis block.
/// 
/// If the database is already populated, make sure it is based on the expected genesis
/// If an empty database is opened, it is initialized with the expected genesis hash and genesis block
pub(crate) async fn open_db(db_path: PathBuf, expected_genesis: H256, client: &HttpClient) -> anyhow::Result<Db> {
    let db = sled::open(db_path)?;

    // Open the tables we'll need
    let wallet_block_hashes_tree = db.open_tree("block_hashes")?;
    let wallet_blocks_tree = db.open_tree("blocks")?;

    // If the database is already populated, just make sure it is for the same genesis block
    if height(&db)? != 0 {
        // There are database blocks, so do a quick precheck to make sure they use the same genesis block.
        let wallet_genesis_ivec = wallet_block_hashes_tree.get(0.encode())?.expect("We know there are some blocks, so there should be a 0th block.");
        let wallet_genesis = H256::decode(&mut &wallet_genesis_ivec[..])?;
        println!("Found existing database.");
        if expected_genesis != wallet_genesis {
            println!("Wallet's genesis does not match expected. Aborting database opening.");
            println!("wallet: {wallet_genesis:?}, expected: {expected_genesis:?}");
            return Err(anyhow!("Node reports a different genesis block than wallet. Wallet: {wallet_genesis:?}. Expected: {expected_genesis:?}. Aborting all operations"));
        }
        return Ok(db)
    }

    // If there are no local blocks yet, initialize the tables
    println!("Found empty database.");
    println!("Initializing fresh sync from genesis {:?}", expected_genesis);

    // TODO better not to pull the block straight from the node here,
    // but instead have a callback to get the genesis block if needed.
    let genesis_block = crate::rpc::node_get_block(expected_genesis, client).await?;

    // Update both tables
    wallet_block_hashes_tree.insert(0u32.encode(), expected_genesis.encode())?;
    wallet_blocks_tree.insert(expected_genesis.encode(), genesis_block.encode());

    Ok(db)
}

/// Gets the block hash from the block height. Similar the Node's RPC.
pub(crate) fn get_block(height: u32) -> anyhow::Result<H256> {
    todo!()
}

/// Apply a block to the local database
pub(crate) async fn apply_block(db: &Db, b: Block, block_hash: H256, keystore: &LocalKeystore) -> anyhow::Result<()> {
    // Write the hash to the block_hashess table
    let wallet_block_hashes_tree = db.open_tree("block_hashes").expect("should be able to open block hashes tree from sled db.");
    wallet_block_hashes_tree.insert(b.header.number.encode(), block_hash.encode())?;

    // Write the block to the blocks table
    let wallet_blocks_tree = db.open_tree("blocks").expect("should be able to open blocks table");
    wallet_blocks_tree.insert(block_hash.encode(), b.encode())?;

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

/// Mark an existing output as spent. This does not purge all record of the output from the db.
/// It just moves the record from the unspent table to the spent table
fn spend_output(db: &Db, output_ref: &OutputRef) -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;
    let spent_tree = db.open_tree("spent_outputs")?;

    let Some(ivec) = unspent_tree.remove(output_ref.encode())? else { return Ok(())};
    let (owner, amount) = <(H256, u128)>::decode(&mut &ivec[..])?;
    spent_tree.insert(output_ref.encode(), (owner, amount).encode())?;

    Ok(())
}

/// Mark an output that was previously spent back as unspent.
fn unspend_output(db: &Db, output_ref: &OutputRef) -> anyhow::Result<()> {
    let unspent_tree = db.open_tree("unspent_outputs")?;
    let spent_tree = db.open_tree("spent_outputs")?;

    let Some(ivec) = spent_tree.remove(output_ref.encode())? else { return Ok(())};
    let (owner, amount) = <(H256, u128)>::decode(&mut &ivec[..])?;
    unspent_tree.insert(output_ref.encode(), (owner, amount).encode())?;

    Ok(())
}

/// Run a transaction backwards against a database. Mark all of the Inputs
/// as unspent, and drop all of the outputs.
fn unapply_transaction(db: &Db, tx: &Transaction) -> anyhow::Result<()> {
    
    // Loop through the inputs moving each from spent to unspent
    for Input { output_ref, .. } in &tx.inputs {
        unspend_output(db, output_ref)?;
    }

    // Loop through the outputs pruning them from unspent and dropping all record
    let tx_hash = BlakeTwo256::hash_of(&tx.encode());

    for i in 0..tx.outputs.len() {
        let output_ref = OutputRef {
            tx_hash,
            index: i as u32,
        };
        remove_unspent_output(db, &output_ref)?;
    }

    Ok(())
}

/// Docs TODO
pub(crate) async fn unapply_highest_block(db: &Db) -> anyhow::Result<Block> {
    let wallet_blocks_tree = db.open_tree("blocks")
        .expect("should be able to open blocks tree from sled db.");
    let wallet_block_hashes_tree = db.open_tree("block_hashes")
        .expect("should be able to open block hashes tree from sled db.");

    // Find the best height
    let height = height(db)?;

    // Take the hash from the blockhashes tables
    let Some(ivec) = wallet_block_hashes_tree.remove(height.encode())? else {
        return Err(anyhow!("No block hash found at height reported as best. DB is inconsistent."))
    };
    let hash = H256::decode(&mut &ivec[..])?;

    // Take the block from the blocks table
    let Some(ivec) = wallet_blocks_tree.remove(hash.encode())? else {
        return Err(anyhow!("Block was not present in bd but block has ws. DB is corrpted."));
    };
       
    let block = Block::decode(&mut &ivec[..])?;

    // Loop through the transactions in reverse order calling unapply
    for tx in block.extrinsics.iter().rev() {
        unapply_transaction(db, tx )?;
    }

    Ok(block)
}

/// Get the block height that the wallet is currently synced to
pub(crate) fn height(db: &Db) -> anyhow::Result<u32> {
    let wallet_block_hashes_tree = db.open_tree("block_hashes")?;
    let num_blocks = wallet_block_hashes_tree.len();
    Ok(num_blocks as u32)
}
