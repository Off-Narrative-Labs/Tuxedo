//! Wallet features related to spending money and checking balances.

use crate::{fetch_storage, SpendArgs};

use std::{thread::sleep, time::Duration};

use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::{Decode, Encode};
use runtime::{
    money::{Coin, MoneyConstraintChecker},
    OuterConstraintChecker, OuterVerifier, Transaction,
};
use sp_core::{crypto::Pair as PairT, sr25519::Pair};
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    types::{Input, Output, OutputRef},
    verifier::SigCheck,
};

/// Create and send a transaction that spends coins on the network
pub async fn spend_coins(client: &HttpClient, args: SpendArgs) -> anyhow::Result<()> {
    println!("The args are:: {:?}", args);
    let (provided_pair, _) = Pair::from_phrase(&args.seed, None)?;

    // Construct a template Transaction to push coins into later
    let mut transaction = Transaction {
        inputs: Vec::new(),
        peeks: Vec::new(),
        evictions: Vec::new(),
        outputs: Vec::new(),
        checker: OuterConstraintChecker::Money(MoneyConstraintChecker::Spend),
    };

    // Make sure each input decodes and is present in storage, and then push to transaction.
    for input in &args.input {
        let output_ref = OutputRef::decode(&mut &hex::decode(input)?[..])?;
        print_coin_from_storage(&output_ref, client).await?;
        transaction.inputs.push(Input {
            output_ref,
            redeemer: vec![], // We will sign the total transaction so this should be empty
        });
    }

    // Construct each output and then push to the transactions
    for amount in &args.output_amount {
        let output = Output {
            payload: Coin::new(*amount).into(),
            verifier: OuterVerifier::SigCheck(SigCheck {
                owner_pubkey: provided_pair.public().into(),
            }),
        };
        transaction.outputs.push(output);
    }

    // Create a signature over the entire transaction
    // TODO this will need to generalize. We will need to loop through the inputs
    // producing the signature for whichever owner it is, or even more generally,
    // producing the verifier for whichever verifier it is.
    let signature = provided_pair.sign(&transaction.encode());

    // Iterate back through the inputs putting the signature in place.
    for input in &mut transaction.inputs {
        input.redeemer = signature.encode();
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
    }

    Ok(())
}
