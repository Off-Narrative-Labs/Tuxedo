//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::path::PathBuf;

use anyhow::anyhow;
use clap::Parser;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use tuxedo_core::{
    types::{Output, OutputRef},
    Verifier,
};

use sp_core::H256;

mod amoeba;
mod money;
mod rpc;
mod sync;
mod cli;
mod keystore;

use cli::{Cli, Command};

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9933";


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

    // Insert the example Shawn key so example transactions can be signed.
    crate::keystore::insert_default_key_for_this_session(&keystore)?;

    // Setup jsonrpsee and endpoint-related information.
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let client = HttpClientBuilder::default().build(cli.endpoint)?;

    // Read node's genesis block.
    let node_genesis_hash = rpc::node_get_block_hash(0, &client)
        .await?
        .expect("node should be able to return some genesis hash");
    let node_genesis_block = rpc::node_get_block(node_genesis_hash, &client)
        .await?
        .expect("node should be able to return some genesis block");
    println!("Node's Genesis block::{:?}", node_genesis_hash);

    // Open the local database
    let db = sync::open_db(db_path, node_genesis_hash, node_genesis_block)?;

    let num_blocks =
        sync::height(&db)?.expect("db should be initialized automatically when opening.");
    println!("Number of blocks in the db: {num_blocks}");

    // Synchronize the wallet with attached node.
    sync::synchronize(&db, &client, &keystore).await?;
    println!(
        "Wallet database synchronized with node to height {:?}",
        sync::height(&db)?
    );

    // Print entire unspent outputs tree
    println!("###### Unspent outputs ###########");
    sync::print_unspent_tree(&db)?;

    // Dispatch to proper subcommand
    match cli.command {
        Command::AmoebaDemo => amoeba::amoeba_demo(&client).await,
        // Command::MultiSigDemo => multi_sig::multi_sig_demo(&client).await,
        Command::VerifyCoin { output_ref } => {
            money::print_coin_from_storage(&output_ref, &client).await
        }
        Command::SpendCoins(args) => money::spend_coins(&db, &client, &keystore, args).await,
        Command::InsertKey { seed } => {
            crate::keystore::insert_key(&keystore, &seed)
        }
        Command::GenerateKey { password } => {
            crate::keystore::generate_key(&keystore, password)?;
            Ok(())
        }
        Command::ShowKeys => {
            crate::keystore::get_keys(&keystore)?
                .for_each(|pubkey| {
                    println!("key: 0x{}", hex::encode(pubkey));
                });

            Ok(())
        }
        Command::RemoveKey { pub_key } => {

            println!("CAUTION!!! About permanently remove {pub_key}. This action CANNOT BE REVERSED. Type \"proceed\" to confirm deletion.");

            let mut confirmation = String::new();
            std::io::stdin()
                .read_line(&mut confirmation)
                .expect("Failed to read line");

            if confirmation.trim() == "proceed" {
                crate::keystore::remove_key(&keystore_path, &pub_key)
            } else {
                println!("Deletion aborted. That was close.");
                Ok(())
            }
        }
        Command::SyncOnly => Ok(()),
    }
}

//TODO move to rpc.rs
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
