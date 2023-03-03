//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::{thread::sleep, time::Duration};

use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use parity_scale_codec::{Decode, Encode};
use runtime::{
    amoeba::{AmoebaCreation, AmoebaDetails, AmoebaMitosis},
    OuterRedeemer, Transaction,
};
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    redeemer::UpForGrabs,
    types::{Input, Output, OutputRef},
};

// This async, tokio, anyhow stuff arose because I needed to `await` for rpc responses.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup jsonrpsee and endpoint-related information. Kind of blindly following
    // https://github.com/paritytech/jsonrpsee/blob/master/examples/examples/http.rs
    let url = "http://localhost:9933";
    let client = HttpClientBuilder::default().build(url)?;

    // TODO Eventually we will have to work with key
    // Generate the well-known alice key
    // let pair = todo!();

    // Construct a simple amoeba spawning transaction (no signature required)
    let eve = AmoebaDetails {
        generation: 0,
        four_bytes: *b"eve_",
    };
    let spawn_tx = Transaction {
        inputs: Vec::new(),
        outputs: vec![Output {
            payload: eve.into(),
            redeemer: UpForGrabs.into(),
        }],
        verifier: AmoebaCreation.into(),
    };

    // Calculate the OutputRef which also serves as the storage location
    let eve_ref = OutputRef {
        tx_hash: <BlakeTwo256 as Hash>::hash_of(&spawn_tx.encode()),
        index: 0,
    };

    // Send the transaction
    let spawn_hex = hex::encode(&spawn_tx.encode());
    let params = rpc_params![spawn_hex];
    let spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;
    println!("Node's response to spawn transaction: {:?}", spawn_response);

    // Wait a few seconds to make sure a block has been authored.
    sleep(Duration::from_secs(3));

    // Check that the amoeba is in storage and print its details
    let eve_from_storage = get_amoeba_from_storage(&eve_ref, &client).await?;
    println!("Eve Amoeba retrieved from storage: {:?}", eve_from_storage);

    // Create a mitosis transaction on the Eve amoeba
    let cain = AmoebaDetails {
        generation: 1,
        four_bytes: *b"cain",
    };
    let able = AmoebaDetails {
        generation: 1,
        four_bytes: *b"able",
    };
    let mitosis_tx = Transaction {
        inputs: vec![Input {
            output_ref: eve_ref,
            witness: Vec::new(),
        }],
        outputs: vec![
            Output {
                payload: cain.into(),
                redeemer: UpForGrabs.into(),
            },
            Output {
                payload: able.into(),
                redeemer: UpForGrabs.into(),
            },
        ],
        verifier: AmoebaMitosis.into(),
    };

    // Calculate the two OutputRefs for the daughters
    let cain_ref = OutputRef {
        tx_hash: <BlakeTwo256 as Hash>::hash_of(&mitosis_tx.encode()),
        index: 0,
    };
    let able_ref = OutputRef {
        tx_hash: <BlakeTwo256 as Hash>::hash_of(&mitosis_tx.encode()),
        index: 1,
    };

    // Send the mitosis transaction
    let mitosis_hex = hex::encode(&mitosis_tx.encode());
    let params = rpc_params![mitosis_hex];
    let mitosis_response: Result<String, _> =
        client.request("author_submitExtrinsic", params).await;
    println!(
        "Node's response to mitosis transaction: {:?}",
        mitosis_response
    );

    // Wait a few seconds to make sure a block has been authored.
    sleep(Duration::from_secs(3));

    // Check that the daughters are in storage and print their details
    let cain_from_storage = get_amoeba_from_storage(&cain_ref, &client).await?;
    println!(
        "Cain Amoeba retrieved from storage: {:?}",
        cain_from_storage
    );
    let able_from_storage = get_amoeba_from_storage(&able_ref, &client).await?;
    println!(
        "Able Amoeba retrieved from storage: {:?}",
        able_from_storage
    );

    Ok(())
}

async fn get_amoeba_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<AmoebaDetails> {
    let ref_hex = hex::encode(&output_ref.encode());
    let params = rpc_params![ref_hex];
    let rpc_response: Result<Option<String>, _> = client.request("state_getStorage", params).await;

    // Open up result and strip off 0x prefix
    let response_hex = rpc_response?
        .expect("Amoeba was not found in storage")
        .chars()
        .skip(2)
        .collect::<String>();
    let response_bytes = hex::decode(response_hex)
        //TODO I would prefer to use `?` here instead of panicking
        .expect("Eve bytes from storage can decode correctly");
    let utxo: Output<OuterRedeemer> = Decode::decode(&mut &response_bytes[..])?;
    let amoeba_from_storage: AmoebaDetails = utxo.payload.extract().unwrap();

    Ok(amoeba_from_storage)
}
