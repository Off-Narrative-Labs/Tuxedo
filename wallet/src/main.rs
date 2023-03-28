//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::path::PathBuf;

use anyhow::anyhow;
use clap::{ArgAction::Append, Args, Parser, Subcommand};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use sp_keystore::SyncCryptoStore;
use sp_runtime::{CryptoTypeId, KeyTypeId};
use tuxedo_core::{
    types::{Output, OutputRef},
    Verifier,
};

use sp_core::{
    crypto::{CryptoTypePublicPair, Pair as PairT},
    sr25519::Pair,
    H256,
};

mod amoeba;
mod money;
mod sync;

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9933";

/// A KeyTypeId to use in the keystore for Tuxedo transactions. We'll use this everywhere
/// until it becomes clear that there is a reason to use multiple of them
const KEY_TYPE: KeyTypeId = KeyTypeId(*b"_tux");

/// A default seed phrase for signing inputs when none is provided
/// Corresponds to the default pubkey.
const SHAWN_PHRASE: &str =
    "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

const SHAWN_PUB_KEY: &str = "d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67";

/// The wallet's main CLI struct
#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to
    endpoint: String,

    #[arg(long, short)]
    /// Path where the wallet data is stored. Wallet data is just keystore at the moment,
    /// but will contain a local database of UTXOs in the future.
    ///
    /// Default value is platform specific
    data_path: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

/// The tasks supported by the wallet
#[derive(Debug, Subcommand)]
enum Command {
    /// Demonstrate creating an amoeba and performing mitosis on it.
    AmoebaDemo,

    /// Verify that a particular coin exists in storage. Show its value and owner.
    VerifyCoin {
        /// A hex-encoded output reference
        #[arg(value_parser = output_ref_from_string)]
        output_ref: OutputRef,
    },

    /// Spend some coins. For now, all outputs go to the same recipient.
    SpendCoins(SpendArgs),

    /// Insert a private key into the keystore to later use when signing transactions.
    InsertKey {
        /// Seed phrase of the key to insert.
        seed: String,
    },

    /// Generate a private key using either some or no password and insert into the keystore
    GenerateKey {
        /// Initialize a public/private key pair with a password
        password: Option<String>,
    },

    /// Show public information about all the keys in the keystore.
    ShowKeys,

    /// Remove a specific key from the keystore.
    /// WARNING! This will permanently delete the private key information. Make sure your
    /// keys are backed up somewhere safe.
    RemoveKey {
        /// The public key to remove
        #[arg(value_parser = h256_from_string)]
        pub_key: H256,
    },

    /// Synchronizes the wallet up to the tip of the chain, and does nothing else.
    SyncOnly,
}

#[derive(Debug, Args)]
pub struct SpendArgs {
    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins.
    #[arg(long, short, value_parser = output_ref_from_string)]
    input: Vec<OutputRef>,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the recipient
    #[arg(long, short, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    recipient: H256,

    // The `action = Append` allows us to accept the same value multiple times.
    /// An output amount. For the transaction to be valid, the outputs must add up to less than
    /// the sum of the inputs. The wallet will not enforce this and will gladly send an invalid transaction
    /// which will then e rejected by the node.
    #[arg(long, short, action = Append)]
    output_amount: Vec<u128>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line args
    let cli = Cli::parse();

    // Setup the data paths.
    let data_path = cli.data_path.unwrap_or_else(default_data_path);
    let keystore_path = data_path.join("keystore");
    let db_path = data_path.join("wallet_database");

    // Setup the keystore
    let keystore = sc_keystore::LocalKeystore::open(keystore_path.clone(), None)?;

    // If the keystore is empty, insert the example Shawn key so example transactions can be signed.
    if keystore.keys(KEY_TYPE)?.is_empty() {
        println!("Keystore wsa empty. Inserting example key for THIS SESSION ONLY");

        // This only inserts it into memory. That should be fine for the example key since it can always be
        // re-inserted on each new run. But for user-provided keys, we want them to be persisted.
        // Hopefully insert_unknown will make that happen.
        keystore
            .sr25519_generate_new(KEY_TYPE, Some(SHAWN_PHRASE))
            .map_err(|e| anyhow!("{:?}", e))?;
    }

    // Setup jsonrpsee and endpoint-related information.
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let client = HttpClientBuilder::default().build(cli.endpoint)?;


    // Setup the sled database which tracks the block hashes the wallet is aware of as
    // well as the UTXOs owned by the keys in this wallet.

    // Read node's genesis block.
    let node_genesis = sync::node_get_block_hash(0, &client).await?.expect("node should be able to return some genesis hash");

    // Open the database
    let db = sled::open(db_path).expect("Database path should exist");
    // println!("{:?}", db);

    // This "blocks" table is a mapping from block number to block hash.
    let wallet_blocks_tree = db.open_tree("blocks").expect("should be able to open blocks tree from sled db.");
    let num_blocks = wallet_blocks_tree.len();
    println!("Number of entries in blocks table: {num_blocks}");

    // If there are no local blocks yet, read the genesis block from the node.
    if wallet_blocks_tree.is_empty() {
        println!("Found empty database.");
        println!("Initializing fresh sync from genesis {:?}", node_genesis);
        wallet_blocks_tree.insert(0u32.encode(), node_genesis.encode())?;
    } else {
        // There are database blocks, so do a quick precheck to make sure they use the same genesis block.
        let wallet_genesis_ivec = wallet_blocks_tree.get(0.encode())?.expect("We know there are some blocks, so there should be a 0th block.");
        let wallet_genesis = H256::decode(&mut &wallet_genesis_ivec[..])?;
        if node_genesis != wallet_genesis {
            Err(anyhow!("Node reports a different genesis block than wallet. Wallet: {wallet_genesis:?}. Node: {node_genesis:?}. Aborting all operations"))?;
        }
    }

    let num_blocks = wallet_blocks_tree.len();
    println!("Number of entries in blocks table: {num_blocks}");

    // initialize loop vars
    let mut height = 0u32;
    let mut wallet_hash = H256::repeat_byte(0);
    let mut node_hash = Some(H256::repeat_byte(16));

    // Check the most recent block hash known to the wallet
    let (height_ivec, hash_ivec) = wallet_blocks_tree.last()?.expect("db was initialized with at least one value");
    height = u32::decode(&mut &height_ivec[..])?;
    wallet_hash = H256::decode(&mut &hash_ivec[..])?;

    // Check the node's hash at that block
    node_hash = sync::node_get_block_hash(height, &client).await?;

    println!("about to start looping. wallet: {wallet_hash:?}. node: {node_hash:?}");

    // There may have been a re-org since the last time the node synced. So we loop backwards from the
    // best height the wallet knows about checking whether the wallet knows the same block as the node.
    // If not, we roll this block back on the wallet's local db, and then check the next ancestor.
    // When the wallet and the node agree on the best block, the wallet can re-sync following the node.
    // In the best case, where there is no re-org, this loop will execute zero times.
    while Some(wallet_hash) != node_hash {
        //TODO might need to manually decrement height here
        println!("Reorg Divergence at height {height}. Wallet: {wallet_hash:?}. Node: {node_hash:?}.");
        
        // Remove the invalid
        // TODO make this a function called eg roll back block.
        // The function will also rewind blocks once it is available.
        wallet_blocks_tree.remove(height.encode())?;

        // Check the most recent block hash known to the wallet
        let (height_ivec, hash_ivec) = wallet_blocks_tree.last()?.expect("db was initialized with at least one value");
        height = u32::decode(&mut &height_ivec[..])?;
        wallet_hash = H256::decode(&mut &hash_ivec[..])?;

        // Check the node's hash at that block
        node_hash = sync::node_get_block_hash(height, &client).await?;
    }

    // Orphaned blocks (if any) have been discarded at this point.
    // So we prepare our variables for forward syncing.
    println!("Resyncing from common ancestor {node_hash:?} - {wallet_hash:?}");
    height += 1;
    node_hash = sync::node_get_block_hash(height, &client).await?;

    // Now that we have checked for reorgs and rolled back any orphan blocks, we can go ahead and sync forward.
   while node_hash.is_some() {
        println!("Forward syncing height {height}, hash {node_hash:?}");

        // Eventually we will need the block in order to apply its transactions
        //let block = sync::node_get_block(hash, &client).await?;

        // Add the new block info
        // TODO make this a helper function
        // which also applies the new blocks
        wallet_blocks_tree.insert(height.encode(), node_hash.unwrap().encode())?;

        height += 1;

        node_hash = sync::node_get_block_hash(height, &client).await?;
    }
    
    println!("Done with forward sync up to {}", height - 1);

    // Now for good measure, print out the entire blocks table.
    for result in wallet_blocks_tree.iter() {
        let (height_ivec, hash_ivec) = result?;
        height = u32::decode(&mut &height_ivec[..])?;
        wallet_hash = H256::decode(&mut &hash_ivec[..])?;
        println!("{height:?}: {wallet_hash:?}");
    }


    // Dispatch to proper subcommand
    match cli.command {
        Command::AmoebaDemo => amoeba::amoeba_demo(&client).await,
        // Command::MultiSigDemo => multi_sig::multi_sig_demo(&client).await,
        Command::VerifyCoin { output_ref } => {
            money::print_coin_from_storage(&output_ref, &client).await
        }
        Command::SpendCoins(args) => money::spend_coins(&client, &keystore, args).await,
        Command::InsertKey { seed } => {
            // We need to provide a public key to the keystore manually, so let's calculate it.
            let public_key = Pair::from_phrase(&seed, None)?.0.public();
            println!("The generated public key is {:?}", public_key);
            keystore
                .insert_unknown(KEY_TYPE, &seed, public_key.as_ref())
                .map_err(|e| anyhow!("{:?}", e))?;
            Ok(())
        }
        Command::GenerateKey { password } => {
            let (pair, phrase, _) = Pair::generate_with_phrase(password.as_deref());
            println!("Generated public key is {:?}", pair.public());
            println!("Generated Phrase is {}", phrase);
            keystore
                .insert_unknown(KEY_TYPE, phrase.as_ref(), pair.public().as_ref())
                .map_err(|e| anyhow!("{:?}", e))?;
            Ok(())
        }
        Command::ShowKeys => {
            keystore
                .keys(KEY_TYPE)?
                .into_iter()
                .filter_map(|CryptoTypePublicPair(t, public)| {
                    // Since we insert with `insert_unknown`, each key is inserted three times.
                    // Here we filter out just the sr25519 variant so we don't print duplicates.
                    if t == CryptoTypeId(*b"sr25") {
                        Some(public)
                    } else {
                        None
                    }
                })
                .for_each(|pubkey| {
                    println!("key: 0x{}", hex::encode(pubkey));
                });

            Ok(())
        }
        Command::RemoveKey { pub_key } => {
            // The keystore doesn't provide an API for removing keys, so we
            // remove them from the filesystem directly
            let filename = format!("{}{}", hex::encode(KEY_TYPE.0), hex::encode(pub_key.0));
            let path = keystore_path.join(filename);

            println!("CAUTION!!! About permanently remove {pub_key}. This action CANNOT BE REVERSED. Type \"proceed\" to confirm deletion.");

            let mut confirmation = String::new();
            std::io::stdin()
                .read_line(&mut confirmation)
                .expect("Failed to read line");

            if confirmation.trim() == "proceed" {
                std::fs::remove_file(path)?;
            } else {
                println!("Deletion aborted. That was close.")
            }

            Ok(())
        }
        Command::SyncOnly => Ok(()),
    }
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

/// Parse a string into an H256 that represents a public key
pub(crate) fn h256_from_string(s: &str) -> anyhow::Result<H256> {
    let s = strip_0x_prefix(s);

    let mut bytes: [u8; 32] = [0; 32];
    hex::decode_to_slice(s, &mut bytes as &mut [u8])
        .map_err(|_| clap::Error::new(clap::error::ErrorKind::ValueValidation))?;
    Ok(H256::from(bytes))
}

/// Parse an output ref from a string
fn output_ref_from_string(s: &str) -> Result<OutputRef, clap::Error> {
    let s = strip_0x_prefix(s);
    let bytes =
        hex::decode(s).map_err(|_| clap::Error::new(clap::error::ErrorKind::ValueValidation))?;

    OutputRef::decode(&mut &bytes[..])
        .map_err(|_| clap::Error::new(clap::error::ErrorKind::ValueValidation))
}

/// Takes a string and checks for a 0x prefix. Returns a string without a 0x prefix.
fn strip_0x_prefix(s: &str) -> &str {
    if &s[..2] == "0x" {
        &s[2..]
    } else {
        s
    }
}

/// Generate the platform-specific default data path for the wallet
fn default_data_path() -> PathBuf {
    // This uses the directories crate.
    // https://docs.rs/directories/latest/directories/struct.ProjectDirs.html

    // Application developers may want to put actual qualifiers or organization here
    let qualifier = "";
    let organization = "";
    let application = env!("CARGO_PKG_NAME");

    directories::ProjectDirs::from(qualifier, organization, application)
        .expect("app directories exist on all supported platforms; qed")
        .config_dir()
        .into()
}
