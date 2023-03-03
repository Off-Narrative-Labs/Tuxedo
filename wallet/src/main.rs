//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::{thread::sleep, time::Duration};

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use runtime::{
    money::{Coin, MoneyVerifier},
    OuterRedeemer, Transaction, OuterVerifier,
};
use sp_core::{
    sr25519::{Pair},
    crypto::{Pair as PairT},
    H256,
};
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    redeemer::{SigCheck},
    types::{Input, Output, OutputRef},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup jsonrpsee and endpoint-related information. Kind of blindly following
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let args: Vec<String> = std::env::args().collect();

    // How much of a coin to create the rest gets burned
    let amount: u128 = args[1].parse().expect("Can parse string into u128");
    // Seed from user
    let seed = &args[2];

    let url = "http://localhost:9933";
    let client = HttpClientBuilder::default().build(url)?;

    const SHAWN_PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
    let (shawn_pair, _) = Pair::from_phrase(SHAWN_PHRASE, None)?;

    println!("Seed is:: {}", seed.as_str());
    let (provided_pair, _) = Pair::from_phrase(seed.as_str(), None)?;

    // Genesis Coin Reference
    let shawn_coin_ref = OutputRef {
        tx_hash: H256::zero(),
        index: 0u32,
    };

    // Construct a simple Transaction to spend Shawn genesis coin
    let mut transaction = Transaction {
        inputs: vec![
            Input {
                output_ref: shawn_coin_ref,
                witness: vec![], // We will sign the total transaction so this should be empty
            }
        ],
        outputs: vec![
            Output {
                payload: Coin::new(amount).into(),
                redeemer: OuterRedeemer::SigCheck(SigCheck { owner_pubkey: provided_pair.public().into() }),
            }
        ],
        verifier: OuterVerifier::Money(MoneyVerifier::Spend),
    };

    let signature = shawn_pair.sign(&transaction.encode());
    transaction.inputs[0].witness = signature.encode();

    // Calculate the OutputRef which also serves as the storage location
    // In order to retrieve later
    let new_coin_ref = OutputRef {
        tx_hash: <BlakeTwo256 as Hash>::hash_of(&transaction.encode()),
        index: 0,
    };

    // Send the transaction
    let genesis_spend_hex = hex::encode(transaction.encode());
    let params = rpc_params![genesis_spend_hex];
    let genesis_spend_response: Result<String, _> =
        client.request("author_submitExtrinsic", params).await;
    println!(
        "Node's response to genesis_spend transaction: {:?}",
        genesis_spend_response
    );

    // Wait a few seconds to make sure a block has been authored.
    sleep(Duration::from_secs(3));

    // Retrieve new coins from storage
    let (pubkey, new_coin_from_storage) = get_coin_from_storage(&new_coin_ref, &client).await?;
    println!("Retrieved the new coin from storage {:?} with owner: {:?}", new_coin_from_storage, sp_core::hexdisplay::HexDisplay::from(&pubkey.encode()));
    Ok(())
}

async fn get_coin_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<(H256, Coin)> {
    let ref_hex = hex::encode(output_ref.encode());
    let params = rpc_params![ref_hex];
    let rpc_response: Result<Option<String>, _> = client.request("state_getStorage", params).await;

    let response_hex = rpc_response?
        .expect("New coin can be retrieved from storage")
        .chars()
        .skip(2) // skipping 0x
        .collect::<String>();
    let response_bytes = hex::decode(response_hex)?;
    let utxo = Output::<OuterRedeemer>::decode(&mut &response_bytes[..])?;
    let coin_in_storage: Coin = utxo.payload.extract().unwrap();
    let mut returned_pubkey = H256::zero();
    if let OuterRedeemer::SigCheck(sig_check) = utxo.redeemer {
        returned_pubkey = sig_check.owner_pubkey;
    }
    Ok((returned_pubkey, coin_in_storage))
}