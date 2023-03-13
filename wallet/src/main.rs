//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::path::PathBuf;

use anyhow::anyhow;
use clap::{ArgAction::Append, Args, Parser, Subcommand};
use hex::ToHex;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use sp_keystore::SyncCryptoStore;
use sp_runtime::KeyTypeId;
use tuxedo_core::{
    types::{Output, OutputRef},
    Redeemer,
};

use sp_core::{crypto::Pair as PairT, sr25519::Pair};

mod amoeba;
mod money;

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9933";

/// The default path for the keystore that stores the keys for signing transactions
const DEFAULT_DATA_PATH: &str = "/local/share/tuxedo-template-wallet";

/// A KeyTypeId to use in the keystore for Tuxedo transactions. We'll use this everywhere
/// until it becomes clear that there si a reason to use multiple of them
const KEY_TYPE: KeyTypeId = KeyTypeId(*b"_tux");

/// A default seed phrase for signing inputs when none is provided
/// Corresponds to the default pubkey.
const SHAWN_PHRASE: &str =
    "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

/// A default pubkey for receiving outputs when none is provided
/// Corresponds to the default seed phrase
const SHAWN_PUB_KEY: &str = "d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67";

/// The wallet's main CLI struct
#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to
    endpoint: String,

    #[arg(long, short, default_value_t = DEFAULT_DATA_PATH.to_string())]
    /// Path where to the wallet data is stored. Wallet data is just keystore at the moment,
    /// but will contain a local database of UTXOs in the future.
    data_path: String,

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
        /// A hex-encoded output reference (non 0x prefixed for now)
        ref_string: String,
    },

    /// Spend some coins. For now, all outputs go to the same recipient.
    SpendCoins(SpendArgs),

    /// Insert a private key into the keystore to later use when signing transactions.
    InsertKey {
        /// Seed phrase of the key to insert.
        seed: String,
    },
}

#[derive(Debug, Args)]
pub struct SpendArgs {
    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins and, for now, must all be owned by the same signer.
    #[arg(long, short)]
    input: Vec<String>,

    /// Hex encoded address (sr25519 pubkey) of the recipient (non 0x prefixed for now)
    /// Generate with subkey, or use Shawn's: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67
    #[arg(long, short, default_value_t = SHAWN_PUB_KEY.to_string())]
    recipient: String,

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

    // Setup the keystore.
    let data_path = PathBuf::from(cli.data_path);
    let keystore_path = data_path.join("keystore");
    let keystore = sc_keystore::LocalKeystore::open(keystore_path, None)?;

    // If the keystore is empty, insert the example Shawn key so example transactions can be signed.
    if keystore.keys(KEY_TYPE)?.is_empty() {
        // This only inserts it into memory. That should be fine for the example key since it can always be
        // re-inserted on each new run. But for user-provided keys, we want them to be persisted.
        // Hopefully insert_unknown will make that happen.
        keystore.sr25519_generate_new(KEY_TYPE, Some(SHAWN_PHRASE));
    }

    // Setup jsonrpsee and endpoint-related information.
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let client = HttpClientBuilder::default().build(cli.endpoint)?;

    // Dispatch to proper subcommand
    match cli.command {
        Command::AmoebaDemo => amoeba::amoeba_demo(&client).await,
        Command::VerifyCoin { ref_string } => {
            let output_ref = OutputRef::decode(&mut &hex::decode(ref_string)?[..])?;
            money::print_coin_from_storage(&output_ref, &client).await
        }
        Command::SpendCoins(args) => money::spend_coins(&client, &keystore, args).await,
        Command::InsertKey { seed } => {
            // We need to provide a public key to the keystore manually, so let's calculate it.
            let public_key = Pair::from_phrase(&seed, None)?.0.public();
            keystore.insert_unknown(KEY_TYPE, &seed, public_key.as_ref());
            Ok(())
        }
    }
}

/// Fetch an output from chain storage given an OutputRef
pub async fn fetch_storage<R: Redeemer>(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<Output<R>> {
    let ref_hex = hex::encode(output_ref.encode());
    let params = rpc_params![ref_hex];
    let rpc_response: Result<Option<String>, _> = client.request("state_getStorage", params).await;

    let response_hex = rpc_response?
        .ok_or(anyhow!("New coin can be retrieved from storage"))?
        .chars()
        .skip(2) // skipping 0x
        .collect::<String>();
    let response_bytes = hex::decode(response_hex)?;
    let utxo = Output::decode(&mut &response_bytes[..])?;

    Ok(utxo)
}
