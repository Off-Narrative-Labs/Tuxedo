//! Toy off-chain process to create an amoeba and perform mitosis on it

use crate::fetch_storage;

use std::{thread::sleep, time::Duration};

use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::Encode;
use runtime::{
    amoeba::{AmoebaCreation, AmoebaDetails, AmoebaMitosis},
    OuterRedeemer, Transaction,
};
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    redeemer::UpForGrabs,
    types::{Input, Output, OutputRef},
};

pub async fn amoeba_demo(client: &HttpClient) -> anyhow::Result<()> {
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
    let eve_from_storage: AmoebaDetails = fetch_storage::<OuterRedeemer>(&eve_ref, client)
        .await?
        .payload
        .extract()?;
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
    let cain_from_storage: AmoebaDetails = fetch_storage::<OuterRedeemer>(&cain_ref, client)
        .await?
        .payload
        .extract()?;
    println!(
        "Cain Amoeba retrieved from storage: {:?}",
        cain_from_storage
    );
    let able_from_storage: AmoebaDetails = fetch_storage::<OuterRedeemer>(&able_ref, client)
        .await?
        .payload
        .extract()?;
    println!(
        "Able Amoeba retrieved from storage: {:?}",
        able_from_storage
    );

    Ok(())
}
