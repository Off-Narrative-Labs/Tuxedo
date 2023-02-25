//! A simple CLI wallet. For now it is a toy just to start testing things out.

use parity_scale_codec::{Decode, Encode};
use runtime::{
    amoeba::{AmoebaCreation, AmoebaDetails, AmoebaMitosis},
    Transaction,
};
use sp_core::{blake2_256, H256};
use tuxedo_core::{
    redeemer::UpForGrabs,
    types::{Input, Output, OutputRef},
};

fn main() {
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

    // Send the transaction
    //todo!();

    // Calculate the OutputRef which also serves as the storage location
    let eve_ref = OutputRef {
        tx_hash: H256::from(blake2_256(&spawn_tx.encode())),
        index: 0,
    };

    // Check that the amoeba is in storage and print its details

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
}
