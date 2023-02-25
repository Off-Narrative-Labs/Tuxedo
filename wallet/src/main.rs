//! A simple CLI wallet. For now it is a toy just to start testing things out.

use std::{thread::sleep, time::Duration};

use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};
use parity_scale_codec::{Decode, Encode};
use runtime::{
    amoeba::{AmoebaCreation, AmoebaDetails, AmoebaMitosis},
    OuterRedeemer, Transaction,
};
use sp_core::{blake2_256, hexdisplay::HexDisplay, H256};
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
        tx_hash: H256::from(<BlakeTwo256 as Hash>::hash_of(&spawn_tx.encode())),
        index: 0,
    };
    let eve_ref_hex = format!("{:?}", HexDisplay::from(&eve_ref.encode()));
    println!("eve ref hex: {:?}", eve_ref_hex);

    // Send the transaction
    let spawn_hex = format!("{:?}", HexDisplay::from(&spawn_tx.encode()));
    let params = rpc_params![spawn_hex];
    let spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;
    println!("Node's response to spawn transaction: {:?}", spawn_response);

    // Wait a few seconds to make sure a block has been authored.
    sleep(Duration::from_secs(4));

    // Check that the amoeba is in storage and print its details
    let params = rpc_params![eve_ref_hex];
    let raw_eve_response: Result<Option<String>, _> =
        client.request("state_getStorage", params).await;
    let eve_from_storage_hex = raw_eve_response?
        .expect("Eve is found in storage")
        .chars()
        .skip(2)
        .collect::<String>();
    let eve_storage_bytes = hex::decode(eve_from_storage_hex)
        .expect("Eve bytes from storage can decode correctly");
    let eve_output_from_storage: Output<OuterRedeemer> = Decode::decode(&mut &eve_storage_bytes[..])?;
    let eve_from_storage: AmoebaDetails = eve_output_from_storage.payload.extract().unwrap();
    println!("Eve Amoeba retreived from storage: {:?}", eve_from_storage);

    // Perform mitosis on the amoeba
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
        tx_hash: H256::from(blake2_256(&mitosis_tx.encode())),
        index: 0,
    };
    let able_ref = OutputRef {
        tx_hash: H256::from(blake2_256(&mitosis_tx.encode())),
        index: 1,
    };

    // Check that the daughters are in storage and print their details

    Ok(())
}

//TODO Ask question (either github issue or stack exchange)
// Why do the blake2_256 methods give different results
#[test]
fn blake_2_256_inconsistency() {
    let message = b"hello world".to_vec();

    let hash_1 = H256::from(sp_core::blake2_256(&message));
    let hash_2 = <sp_runtime::traits::BlakeTwo256 as sp_runtime::traits::Hash>::hash_of(&message);

    println!("Hash 1: {:?}", hash_1);
    println!("Hash 2: {:?}", hash_2);

    assert_eq!(hash_1, hash_2);
}
