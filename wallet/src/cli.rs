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

/// The wallet's main CLI struct
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to.
    pub endpoint: String,

    #[arg(long, short('d'))]
    /// Path where the wallet data is stored. Default value is platform specific.
    pub base_path: Option<PathBuf>,

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
    /// Demonstrate creating an amoeba and performing mitosis on it.
    AmoebaDemo,

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
