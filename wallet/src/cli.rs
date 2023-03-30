//! Tuxedo Template Wallet's Command Line Interface.
//! 
//! Built with clap's derive macros.

use std::path::PathBuf;

use clap::{ArgAction::Append, Args, Parser, Subcommand};
use sp_core::H256;
use tuxedo_core::types::OutputRef;

use crate::{DEFAULT_ENDPOINT, keystore::SHAWN_PUB_KEY, h256_from_string, output_ref_from_string};

/// The wallet's main CLI struct
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to
    pub endpoint: String,

    #[arg(long, short)]
    /// Path where the wallet data is stored. Wallet data is just keystore at the moment,
    /// but will contain a local database of UTXOs in the future.
    ///
    /// Default value is platform specific
    pub data_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

/// The tasks supported by the wallet
#[derive(Debug, Subcommand)]
pub enum Command {
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
    pub input: Vec<OutputRef>,

    // /// All inputs to the transaction will be from this same sender.
    // /// When not specified, inputs from any owner are chosen indiscriminantly
    // #[arg(long, short, value_parser = h256_from_string)]
    // sender: Option<H256>,

    // https://docs.rs/clap/latest/clap/_derive/_cookbook/typed_derive/index.html
    // shows how to specify a custom parsing function
    /// Hex encoded address (sr25519 pubkey) of the recipient
    #[arg(long, short, value_parser = h256_from_string, default_value = SHAWN_PUB_KEY)]
    pub recipient: H256,

    // The `action = Append` allows us to accept the same value multiple times.
    /// An output amount. For the transaction to be valid, the outputs must add up to less than
    /// the sum of the inputs. The wallet will not enforce this and will gladly send an invalid transaction
    /// which will then e rejected by the node.
    #[arg(long, short, action = Append)]
    pub output_amount: Vec<u128>,
}