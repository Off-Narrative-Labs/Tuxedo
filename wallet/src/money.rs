//! Wallet features related to spending money and checking balances.

use crate::{fetch_storage, SpendArgs, KEY_TYPE};

use std::{thread::sleep, time::Duration};

use anyhow::anyhow;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::Encode;
use runtime::{
    money::{Coin, MoneyConstraintChecker},
    OuterConstraintChecker, OuterVerifier, Transaction,
};
use sc_keystore::LocalKeystore;
use sp_core::sr25519::Public;
use sp_keystore::CryptoStore;
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    types::{Input, Output, OutputRef},
    verifier::SigCheck,
};

/// Create and send a transaction that spends coins on the network
pub async fn spend_coins(
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: SpendArgs,
) -> anyhow::Result<()> {
    println!("The args are:: {:?}", args);

    // Construct a template Transaction to push coins into later
    let mut transaction = Transaction {
        inputs: Vec::new(),
        outputs: Vec::new(),
        checker: OuterConstraintChecker::Money(MoneyConstraintChecker::Spend),
    };

    // Make sure each input decodes and is present in storage, and then push to transaction.
    for output_ref in &args.input {
        print_coin_from_storage(output_ref, client).await?;
        transaction.inputs.push(Input {
            output_ref: output_ref.clone(),
            redeemer: vec![], // We will sign the total transaction so this should be empty
        });
    }

    // Construct each output and then push to the transactions
    for amount in &args.output_amount {
        let output = Output {
            payload: Coin::new(*amount).into(),
            verifier: OuterVerifier::SigCheck(SigCheck {
                owner_pubkey: args.recipient,
            }),
        };
        transaction.outputs.push(output);
    }

    // Keep a copy of the stripped encoded transaction for signing purposes
    let stripped_encoded_transaction = transaction.clone().encode();

    // Iterate back through the inputs, signing, and putting the signatures in place.
    for input in &mut transaction.inputs {
        // Fetch the output from storage
        let utxo = fetch_storage::<OuterVerifier>(&input.output_ref, client).await?;

        // Construct the proof that it can be consumed
        let redeemer = match utxo.verifier {
            OuterVerifier::SigCheck(SigCheck { owner_pubkey }) => {
                let public = Public::from_h256(owner_pubkey);
                keystore
                    .sign_with(KEY_TYPE, &public.into(), &stripped_encoded_transaction)
                    .await?
                    .ok_or(anyhow!("Key doesn't exist in keystore"))?
            }
            OuterVerifier::UpForGrabs(_) => Vec::new(),
            OuterVerifier::ThresholdMultiSignature(_) => todo!(),
        };

        // insert the proof
        input.redeemer = redeemer;
    }

    // Send the transaction
    let genesis_spend_hex = hex::encode(transaction.encode());
    let params = rpc_params![genesis_spend_hex];
    let genesis_spend_response: Result<String, _> =
        client.request("author_submitExtrinsic", params).await;
    println!(
        "Node's response to spend transaction: {:?}",
        genesis_spend_response
    );

    // Wait a few seconds to make sure a block has been authored.
    sleep(Duration::from_secs(3));

    // Retrieve new coins from storage
    for i in 0..transaction.outputs.len() {
        let new_coin_ref = OutputRef {
            tx_hash: <BlakeTwo256 as Hash>::hash_of(&transaction.encode()),
            index: i as u32,
        };

        print_coin_from_storage(&new_coin_ref, client).await?;
    }

    Ok(())
}

/// Pretty print the details of a coin in storage given the OutputRef
pub async fn print_coin_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<()> {
    let utxo = fetch_storage::<OuterVerifier>(output_ref, client).await?;
    let coin_in_storage: Coin = utxo.payload.extract()?;

    print!(
        "{}: Found coin worth {:?} units ",
        hex::encode(output_ref.encode()),
        coin_in_storage.0
    );

    match utxo.verifier {
        OuterVerifier::SigCheck(sig_check) => {
            println! {"owned by 0x{}", hex::encode(sig_check.owner_pubkey)}
        }
        OuterVerifier::UpForGrabs(_) => println!("that can be spent by anyone"),
        OuterVerifier::ThresholdMultiSignature(_) => println!("ThresholdMultiSign TODO!"),
    }

    Ok(())
}
