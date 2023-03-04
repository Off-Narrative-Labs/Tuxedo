//! A simple CLI wallet. For now it is a toy just to start testing things out.

use anyhow::anyhow;
use clap::{ArgAction::Append, Args, Parser, Subcommand};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use tuxedo_core::{
    types::{Output, OutputRef},
    Redeemer,
};

mod amoeba;
mod money;

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9933";

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

    /// Spend some coins.
    ///
    /// For now, all inputs must be owned by the same signer, and all outputs go to
    /// the same recipient.
    SpendCoins(SpendArgs),
}

#[derive(Debug, Args)]
pub struct SpendArgs {
    /// The seed phrase of the coin owner who will sign the inputs. For now, all inputs
    /// must be owned by the same signer.
    #[arg(long, short, default_value_t = SHAWN_PHRASE.to_string())]
    seed: String,

    /// An input to be consumed by this transaction. This argument may be specified multiple times.
    /// They must all be coins and, for now, must all be owned by the same signer.
    #[arg(long, short)]
    input: Vec<String>,

    /// Hex encoded address (sr25519 pubkey) of the recipient (non 0x prefixed for now)
    /// Generate with subkey, or use Shawn's: d2bf4b844dfefd6772a8843e669f943408966a977e3ae2af1dd78e0f55f4df67
    #[arg(long, short, default_value_t = SHAWN_PUB_KEY.to_string())]
    recipient: String,

    // The `action = Append` allows us to accept the same value multiple times.
    // This works on the wallet side, but is incorrectly rejected by the node. See #37.
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
        Command::SpendCoins(args) => money::spend_coins(&client, args).await,
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
