//! Tuxedo Template Wallet's Command Line Interface.
//!
//! Built with clap's derive macros.

use std::path::PathBuf;

use clap::{ArgAction::Append, Args, Parser, Subcommand};
use sp_core::H256;
use tuxedo_core::types::OutputRef;

use crate::{h256_from_string, keystore::SHAWN_PUB_KEY, output_ref_from_string, DEFAULT_ENDPOINT};

/// The default number of coins to be minted.
pub const DEFAULT_MINT_VALUE: &str = "100";

/// Default recipient is  SHAWN_KEY and output amount is 0
pub const DEFAULT_RECIPIENT: &str = "d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67 0";

/// The default name of the kitty to be created. Show be 4 character long
pub const DEFAULT_KITTY_NAME: &str = "    ";

/// The default gender of the kitty to be created.
pub const DEFAULT_KITTY_GENDER: &str = "female";
use crate::keystore;


fn parse_recipient_coins(s: &str) -> Result<(H256, Vec<u128>), &'static str> {
    println!("In parse_recipient_coins");
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 2 {
        let mut recipient = h256_from_string(parts[0]);
        let coins = parts[1..].iter().filter_map(|&c| c.parse().ok()).collect();
        match recipient {
            Ok(r) => {
                println!("Recipient: {}", r);
                return Ok((r, coins));
            },
            _ => {},
        };
    }
    println!("Sending the error value ");
    Err("Invalid input format")
}




/// The wallet's main CLI struct
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to.
    pub endpoint: String,

    #[arg(long, short)]
    /// Path where the wallet data is stored. Default value is platform specific.
    pub path: Option<PathBuf>,

    #[arg(long, verbatim_doc_comment)]
    /// Skip the initial sync that the wallet typically performs with the node.
    /// The wallet will use the latest data it had previously synced.
    pub no_sync: bool,

    #[arg(long)]
    /// A temporary directory will be created to store the configuration and will be deleted at the end of the process.
    /// path will be ignored if this is set.
    pub tmp: bool,

    #[arg(long, verbatim_doc_comment)]
    /// Specify a development wallet instance, using a temporary directory (like --tmp).
    /// The keystore will contain the development key Shawn.
    pub dev: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The tasks supported by the wallet
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the block based on block height.
    /// get the block hash ad print the block.
    getBlock {
        /// Input the blockheight to be retrived.
        block_height: Option<u32>, // fixme
    },

    /*
    /// Demonstrate creating an amoeba and performing mitosis on it.
    AmoebaDemo,
    */
    /// Mint coins , optionally amount and publicKey of owner can be passed
    /// if amount is not passed , 100 coins are minted
    /// If publickKey of owner is not passed , then by default SHAWN_PUB_KEY is used.
    #[command(verbatim_doc_comment)]
    MintCoins(MintCoinArgs),

    /// Verify that a particular coin exists.
    /// Show its value and owner from both chain storage and the local database.
    #[command(verbatim_doc_comment)]
    VerifyCoin {
        /// A hex-encoded output reference
        #[arg(value_parser = output_ref_from_string)]
        output_ref: OutputRef,
    },

    //Some(Command::MintCoins { amount }) => money::mint_coins(&db, &client, &keystore,amount).await,
    /// Spend some coins.
    /// For now, all outputs in a single transaction go to the same recipient.
    // FixMe: #62
    #[command(verbatim_doc_comment)]
    SpendCoins(SpendArgs),

    /// Insert a private key into the keystore to later use when signing transactions.
    InsertKey {
        /// Seed phrase of the key to insert.
        seed: String,
        // /// Height from which the blockchain should be scanned to sync outputs
        // /// belonging to this address. If non is provided, no re-syncing will
        // /// happen and this key will be treated like a new key.
        // sync_height: Option<u32>,
    },

    /// Generate a private key using either some or no password and insert into the keystore.
    GenerateKey {
        /// Initialize a public/private key pair with a password
        password: Option<String>,
    },

    /// Show public information about all the keys in the keystore.
    ShowKeys,

    /// Remove a specific key from the keystore.
    /// WARNING! This will permanently delete the private key information.
    /// Make sure your keys are backed up somewhere safe.
    #[command(verbatim_doc_comment)]
    RemoveKey {
        /// The public key to remove
        #[arg(value_parser = h256_from_string)]
        pub_key: H256,
    },

    /// For each key tracked by the wallet, shows the sum of all UTXO values owned by that key.
    /// This sum is sometimes known as the "balance".
    #[command(verbatim_doc_comment)]
    ShowBalance,

    /// Show the complete list of UTXOs known to the wallet.
    ShowAllOutputs,

    /// Show the latest on-chain timestamp.
    ShowTimestamp,

    /// Create Kitty without mom and dad.
    CreateKitty(CreateKittyArgs),

    /// Verify that a particular kitty exists.
    /// Show its details and owner from both chain storage and the local database.

    #[command(verbatim_doc_comment)]
    VerifyKitty {
        /// A hex-encoded output reference
        #[arg(value_parser = output_ref_from_string)]
        output_ref: OutputRef,
    },

    /// Breed Kitties.
    #[command(verbatim_doc_comment)]
    BreedKitty(BreedKittyArgs),

    /// List Kitty for sale.
    ListKittyForSale(ListKittyForSaleArgs),

    /// Delist Kitty from sale.
    DelistKittyFromSale(DelistKittyFromSaleArgs),

    /// Update kitty name.
    UpdateKittyName(UpdateKittyNameArgs),

    /// Update kitty price. Applicable only to tradable kitties
    UpdateKittyPrice(UpdateKittyPriceArgs),

    /// Buy Kitty.
    #[command(verbatim_doc_comment)]
    BuyKitty(BuyKittyArgs),

    /// Show all kitties  key tracked by the wallet.
    #[command(verbatim_doc_comment)]
    ShowAllKitties,

    /// ShowOwnedKitties.
    /// Shows the kitties owned by owner.
    #[command(verbatim_doc_comment)]
    ShowOwnedKitties(ShowOwnedKittyArgs),
}

#[derive(Debug, Args)]
pub struct MintCoinArgs {
    /// Pass the amount to be minted.
    #[arg(long, short, verbatim_doc_comment, action = Append,default_value = DEFAULT_MINT_VALUE)]
    pub amount: u128,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

/*
// Old implementation 
#[derive(Debug, Args)]
pub struct SpendArgs {
    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins.
    #[arg(long, short, verbatim_doc_comment, value_parser = output_ref_from_string)]
    pub input: Vec<OutputRef>,

    // /// All inputs to the transaction will be from this same sender.
    // /// When not specified, inputs from any owner are chosen indiscriminantly
    // #[arg(long, short, value_parser = h256_from_string)]
    // sender: Option<H256>,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the recipient.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub recipient: H256,

    // The `action = Append` allows us to accept the same value multiple times.
    /// An output amount. For the transaction to be valid, the outputs must add up to less than the sum of the inputs.
    /// The wallet will not enforce this and will gladly send an invalid which will then be rejected by the node.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub output_amount: Vec<u128>,
}
*/

#[derive(Debug, Args,Clone)]
pub struct SpendArgs {
    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins.
    #[arg(long, short, verbatim_doc_comment, value_parser = output_ref_from_string)]
    pub input: Vec<OutputRef>,

    /// Variable number of recipients and their associated coins.
    /// For example, "--recipients 0x1234 1 2 --recipients 0x5678 3 4 6"
   // #[arg(long, short, verbatim_doc_comment, value_parser = parse_recipient_coins, action = Append)]
   // pub recipients: Vec<(H256, Vec<u128>)>,
   #[arg(long, short, verbatim_doc_comment, value_parser = parse_recipient_coins, action = Append, 
    default_value = DEFAULT_RECIPIENT)]
   pub recipients: Vec<(H256, Vec<u128>)>,
}

#[derive(Debug, Args)]
pub struct CreateKittyArgs {

    /// Pass the name of the kitty to be minted.
    /// If kitty name is not passed ,name is choosen randomly from predefine name vector.
    #[arg(long, short, action = Append, default_value = DEFAULT_KITTY_NAME)]
    pub kitty_name: String,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct ShowOwnedKittyArgs {
    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct BreedKittyArgs {
    /// Name of Mom to be used for breeding .
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub mom_name: String,

    /// Name of Dad to be used for breeding .
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub dad_name: String,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct UpdateKittyNameArgs {
    /// Current name of Kitty.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub current_name: String,

    /// New name of Kitty.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub new_name: String,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct UpdateKittyPriceArgs {
    /// Current name of Kitty.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub current_name: String,

    /// Price of Kitty.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub price: u128,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct BuyKittyArgs {
    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins.
    #[arg(long, short, verbatim_doc_comment, value_parser = output_ref_from_string)]
    pub input: Vec<OutputRef>,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the seller of tradable kitty.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub seller: H256,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner of tradable kitty.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,

    /// Name of kitty to be bought.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub kitty_name: String,

    // The `action = Append` allows us to accept the same value multiple times.
    /// An output amount. For the transaction to be valid, the outputs must add up to less than the sum of the inputs.
    /// The wallet will not enforce this and will gladly send an invalid which will then be rejected by the node.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub output_amount: Vec<u128>,
}

#[derive(Debug, Args)]
pub struct ListKittyForSaleArgs {
    /// Pass the name of the kitty to be listed for sale.
    #[arg(long, short, verbatim_doc_comment, action = Append, default_value = DEFAULT_KITTY_NAME)]
    pub name: String,

    /// Price of Kitty.
    #[arg(long, short, verbatim_doc_comment, action = Append)]
    pub price: u128,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}

#[derive(Debug, Args)]
pub struct DelistKittyFromSaleArgs {
    /// Pass the name of the kitty to be delisted or removed from the sale .
    #[arg(long, short, verbatim_doc_comment, action = Append, default_value = DEFAULT_KITTY_NAME)]
    pub name: String,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the owner.
    #[arg(long, short, verbatim_doc_comment, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub owner: H256,
}
