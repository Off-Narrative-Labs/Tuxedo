//! A simple CLI wallet. For now it is a toy just to start testing things out.


use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::{http_client::HttpClient};
use parity_scale_codec::{Decode};
use runtime::OuterVerifier;
use std::path::PathBuf;
use sled::Db;
//use crate::kitty::{create_kitty,list_kitty_for_sale};
use tuxedo_core::{types::OutputRef, verifier::*};
use sp_core::H256;
use sc_keystore::LocalKeystore;

//mod amoeba;
mod cli;
mod keystore;
mod kitty;
mod money;
mod output_filter;
mod rpc;
mod sync;
mod timestamp;

//use moneyServicehandler::{MintCoinsRequest, MintCoinsResponse};
mod serviceHandlers {
    
    pub mod blockHandler {
        pub mod blockServicehandler;
    }

    pub mod moneyHandler {
        pub mod moneyServicehandler;
    }

    pub mod kittyHandler {
        pub mod kittyServicehandler;
    }

    pub mod keyHandler {
        pub mod keyServicehandler;
    }
}

use serviceHandlers::keyHandler::keyServicehandler::{
    debug_generate_key,
    debug_get_keys,
};

use serviceHandlers::moneyHandler::moneyServicehandler::{
    mint_coins,
    get_all_coins,
    get_owned_coins,
};

use serviceHandlers::kittyHandler::kittyServicehandler::{
    create_kitty,
    get_txn_and_inpututxolist_for_list_kitty_for_sale,
    list_kitty_for_sale,
    get_txn_and_inpututxolist_for_delist_kitty_from_sale,
    delist_kitty_from_sale,
    get_txn_and_inpututxolist_for_kitty_name_update,
    update_kitty_name,
    get_txn_and_inpututxolist_for_td_kitty_name_update,
    update_td_kitty_name,
    get_txn_and_inpututxolist_for_td_kitty_price_update,
    update_td_kitty_price,
    get_kitty_by_dna,
    get_td_kitty_by_dna,
    get_all_kitty_list,
    get_all_td_kitty_list,
    get_owned_kitty_list,
    get_owned_td_kitty_list,
    get_txn_and_inpututxolist_for_breed_kitty,
    breed_kitty,
    get_txn_and_inpututxolist_for_buy_kitty,
    buy_kitty,

    
    /*delist_kitty_from_sale,
    buy_kitty,
    breed_kitty,*/
};

use serviceHandlers::blockHandler::blockServicehandler::{  get_block};

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9944";

use axum::{routing::{get, post, put}, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};


#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/get-block", get(get_block)) 
        .route("/mint-coins", post(mint_coins))
        .route("/create-kitty", post(create_kitty))
        .route("/get-txn-and-inpututxolist-for-listkitty-forsale", get(get_txn_and_inpututxolist_for_list_kitty_for_sale))
        .route("/listkitty-for-sale", post(list_kitty_for_sale))
        .route("/get-txn-and-inpututxolist-for-delist-kitty-from-sale", get(get_txn_and_inpututxolist_for_delist_kitty_from_sale))
        .route("/delist-kitty-from-sale", post(delist_kitty_from_sale))
        .route("/get-txn-and-inpututxolist-for-kitty-name-update", get(get_txn_and_inpututxolist_for_kitty_name_update))
        .route("/update-kitty-name", post(update_kitty_name))
        .route("/get-txn-and-inpututxolist-for-td-kitty-name-update", get(get_txn_and_inpututxolist_for_td_kitty_name_update))
        .route("/update-td-kitty-name", post(update_td_kitty_name))
        .route("/get-txn-and-inpututxolist-for-td-kitty-price-update", get(get_txn_and_inpututxolist_for_td_kitty_price_update))
        .route("/get-txn-and-inpututxolist-for-breed-kitty", get(get_txn_and_inpututxolist_for_breed_kitty))
        .route("/breed-kitty", post(breed_kitty))
        .route("/get-txn-and-inpututxolist-for-buy-kitty", get(get_txn_and_inpututxolist_for_buy_kitty))
        .route("/buy-kitty", post(buy_kitty))
        .route("/update-td-kitty-price", post(update_td_kitty_price))
        .route("/get-kitty-by-dna", get(get_kitty_by_dna))
        .route("/get-tradable-kitty-by-dna", get(get_td_kitty_by_dna))
        .route("/get-all-kitty-list", get(get_all_kitty_list))
        .route("/get-all-tradable-kitty-list", get(get_all_td_kitty_list))
        .route("/get-owned-kitty-list", get(get_owned_kitty_list))
        .route("/get-owned-tradable-kitty-list", get(get_owned_td_kitty_list))
        .route("/get-all-coins", get(get_all_coins))
        .route("/get-owned-coins", get(get_owned_coins))


        .route("/debug-generate-key", post(debug_generate_key))
        .route("/debug-get-keys", get(debug_get_keys))

        //.route("/spend-coins", put(spend_coins))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("In the main");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn original_get_db() -> anyhow::Result<Db> {

    let client = HttpClientBuilder::default().build(DEFAULT_ENDPOINT)?;

    // Read node's genesis block.
    let node_genesis_hash = rpc::node_get_block_hash(0, &client)
        .await?
        .expect("node should be able to return some genesis hash");
    let node_genesis_block = rpc::node_get_block(node_genesis_hash, &client)
        .await?
        .expect("node should be able to return some genesis block");
    log::debug!("Node's Genesis block::{:?}", node_genesis_hash);

    // Open the local database
    let data_path = temp_dir();
    let db_path = data_path.join("wallet_database");
    let db = sync::open_db(db_path, node_genesis_hash, node_genesis_block.clone())?;

    let num_blocks =
        sync::height(&db)?.expect("db should be initialized automatically when opening.");
    log::info!("Number of blocks in the db: {num_blocks}");

    // No filter at-all 
    let keystore_filter = |_v: &OuterVerifier| -> bool {
        true
    };

    if !sled::Db::was_recovered(&db) {
        println!("!sled::Db::was_recovered(&db) called ");
        // This is a new instance, so we need to apply the genesis block to the database.
        async { 
            sync::apply_block(&db, node_genesis_block, node_genesis_hash, &keystore_filter)
            .await;
        };
    }

    sync::synchronize(&db, &client, &keystore_filter).await?;

    println!(
        "Wallet database synchronized with node to height {:?}",
        sync::height(&db)?.expect("We just synced, so there is a height available")
    );

    if let Err(err) = db.flush() {
        println!("Error flushing Sled database: {}", err);
    }
    
    Ok(db)
}


async fn get_db() -> anyhow::Result<Db> {
    let client = HttpClientBuilder::default().build(DEFAULT_ENDPOINT)?;
    let data_path = temp_dir();
    let db_path = data_path.join("wallet_database");
    let node_genesis_hash = rpc::node_get_block_hash(0, &client)
        .await?
        .expect("node should be able to return some genesis hash");
    let node_genesis_block = rpc::node_get_block(node_genesis_hash, &client)
        .await?
        .expect("node should be able to return some genesis block");
    println!("Node's Genesis block::{:?}", node_genesis_hash);

    let db = sync::open_db(db_path, node_genesis_hash, node_genesis_block.clone())?;
    Ok(db)
}


async fn get_local_keystore() -> anyhow::Result<LocalKeystore> {
    let data_path = temp_dir();
    let keystore_path = data_path.join("keystore");
    println!("keystore_path: {:?}", keystore_path);
    let keystore = sc_keystore::LocalKeystore::open(keystore_path.clone(), None)?;
    keystore::insert_development_key_for_this_session(&keystore)?;
    Ok(keystore)
}

async fn sync_db<F: Fn(&OuterVerifier) -> bool>(
    db: &Db, 
    client: &HttpClient, 
    filter: &F) -> anyhow::Result<()> {
    
    if !sled::Db::was_recovered(&db) {
        let node_genesis_hash = rpc::node_get_block_hash(0, &client)
            .await?
            .expect("node should be able to return some genesis hash");
        let node_genesis_block = rpc::node_get_block(node_genesis_hash, &client)
            .await?
            .expect("node should be able to return some genesis block");

            
        println!(" in sync_db !sled::Db::was_recovered(&db)");
        async { 
            sync::apply_block(&db, node_genesis_block, node_genesis_hash, &filter)
            .await;
        };
    }
    println!(" sync::synchronize will be called!!");
    sync::synchronize(&db, &client, &filter).await?;
    
    log::info!(
        "Wallet database synchronized with node to height {:?}",
        sync::height(&db)?.expect("We just synced, so there is a height available")
    );
    Ok(())
}

async fn sync_and_get_db() -> anyhow::Result<Db> {
    let db = get_db().await?;
    let keystore = get_local_keystore().await?;
    let client = HttpClientBuilder::default().build(DEFAULT_ENDPOINT)?;
    let keystore_filter = |v: &OuterVerifier| -> bool {
        matches![v,
            OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey })
                if crate::keystore::has_key(&keystore, owner_pubkey)
        ] || matches![v, OuterVerifier::UpForGrabs(UpForGrabs)] // used for timestamp
    };
    sync_db(&db, &client, &keystore_filter).await?;
    Ok(db)
}

/// Parse a string into an H256 that represents a public key
pub(crate) fn h256_from_string(s: &str) -> anyhow::Result<H256> {
    let s = strip_0x_prefix(s);

    let mut bytes: [u8; 32] = [0; 32];
    hex::decode_to_slice(s, &mut bytes as &mut [u8])
        .map_err(|_| clap::Error::new(clap::error::ErrorKind::ValueValidation))?;
    Ok(H256::from(bytes))
}

use std::error::Error;
/// Parse an output ref from a string
pub(crate) fn convert_output_ref_from_string(s: &str) -> Result<OutputRef, Box<dyn Error>> {
    let s = strip_0x_prefix(s);
    let bytes = hex::decode(s)?;

    OutputRef::decode(&mut &bytes[..])
        .map_err(|_| "Failed to decode OutputRef from string".into())
}

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

/// Generate a plaform-specific temporary directory for the wallet
fn temp_dir() -> PathBuf {
    // Since it is only used for testing purpose, we don't need a secure temp dir, just a unique one.
    /*
    std::env::temp_dir().join(format!(
        "tuxedo-wallet-{}",
        std::time::UNIX_EPOCH.elapsed().unwrap().as_millis(),
    ))
    */
    std::env::temp_dir().join(format!(
        "tuxedo-wallet"
    ))
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
        .data_dir()
        .into()
}

/// Utility to pretty print an outer verifier
pub fn pretty_print_verifier(v: &OuterVerifier) {
    match v {
        OuterVerifier::Sr25519Signature(sr25519_signature) => {
            println! {"owned by {}", sr25519_signature.owner_pubkey}
        }
        OuterVerifier::UpForGrabs(_) => println!("that can be spent by anyone"),
        OuterVerifier::ThresholdMultiSignature(multi_sig) => {
            let string_sigs: Vec<_> = multi_sig
                .signatories
                .iter()
                .map(|sig| format!("0x{}", hex::encode(sig)))
                .collect();
            println!(
                "Owned by {:?}, with a threshold of {} sigs necessary",
                string_sigs, multi_sig.threshold
            );
        }
    }
}
