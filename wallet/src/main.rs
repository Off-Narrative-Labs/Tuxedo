//! A simple CLI wallet. For now it is a toy just to start testing things out.

use jsonrpsee::{http_client::{HttpClientBuilder, HttpClient}, rpc_params, core::client::ClientT};
use clap::{Parser, Subcommand};
use parity_scale_codec::{Encode, Decode};
use tuxedo_core::{Redeemer, types::{Output, OutputRef}};
use anyhow::anyhow;

mod amoeba;
mod money;

const DEFAULT_ENDPOINT: &str = "http://localhost:9933";


#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
    #[arg(long, short, default_value_t = DEFAULT_ENDPOINT.to_string())]
    /// RPC endpoint of the node that this wallet will connect to
    endpoint: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Demonstrate creating an amoeba and performing mitosis on it.
    AmoebaDemo,
    /// Verify that a particular coin exists in storage. Show its value and owner.
    VerifyCoin,
    /// Spend some coins
    SpendCoins,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup jsonrpsee and endpoint-related information.
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let client = HttpClientBuilder::default().build(cli.endpoint)?;

    match cli.command {
        Command::AmoebaDemo => amoeba::amoeba_demo(&client).await,
        Command::VerifyCoin => todo!(),
        Command::SpendCoins => money::spend_coins(&client).await,
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