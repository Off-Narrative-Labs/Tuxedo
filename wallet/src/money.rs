//! Wallet features related to spending money and checking balances.

use crate::{cli::SpendArgs, fetch_storage, sync};

use anyhow::anyhow;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::Encode;
use runtime::{
    money::{Coin, MoneyConstraintChecker},
    OuterConstraintChecker, OuterVerifier, Transaction,
};
use sc_keystore::LocalKeystore;
use sled::Db;
use sp_core::sr25519::Public;
use sp_runtime::traits::{BlakeTwo256, Hash};
use tuxedo_core::{
    types::{Input, Output, OutputRef},
    verifier::SigCheck,
};

/// Create and send a transaction that spends coins on the network
pub async fn spend_coins(
    db: &Db,
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

    // Construct each output and then push to the transactions
    let mut total_output_amount = 0;
    for amount in &args.output_amount {
        let output = Output {
            payload: Coin::new(*amount).into(),
            verifier: OuterVerifier::SigCheck(SigCheck {
                owner_pubkey: args.recipient,
            }),
        };
        total_output_amount += amount;
        transaction.outputs.push(output);
    }

    // The total input set will consist of any manually chosen inputs
    // plus any automatically chosen to make the input amount high enough
    let mut total_input_amount = 0;
    let mut all_input_refs = args.input;
    for output_ref in &all_input_refs {
        let (_owner_pubkey, amount) = sync::get_unspent(db, output_ref)?.ok_or(anyhow!(
            "user-specified output ref not found in local database"
        ))?;
        total_input_amount += amount;
    }
    //TODO filtering on a specific sender
    while total_input_amount < total_output_amount {
        let (output_ref, _owner_pubkey, amount) = sync::get_arbitrary_unspent(db)?;
        all_input_refs.push(output_ref);
        total_input_amount += amount;
    }

    // Make sure each input decodes and is still present in the node's storage,
    // and then push to transaction.
    for output_ref in &all_input_refs {
        get_coin_from_storage(output_ref, client).await?;
        transaction.inputs.push(Input {
            output_ref: output_ref.clone(),
            redeemer: vec![], // We will sign the total transaction so this should be empty
        });
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
                crate::keystore::sign_with(keystore, &public, &stripped_encoded_transaction)?
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

    // Print new output refs for user to check later
    let tx_hash = <BlakeTwo256 as Hash>::hash_of(&transaction.encode());
    for (i, output) in transaction.outputs.iter().enumerate() {
        let new_coin_ref = OutputRef {
            tx_hash,
            index: i as u32,
        };
        let amount = output.payload.extract::<Coin>()?.0;

        print!(
            "Created {:?} worth {amount}. ",
            hex::encode(new_coin_ref.encode())
        );
        crate::pretty_print_verifier(&output.verifier);
    }

    Ok(())
}

/// Given an output ref, fetch the details about this coin from the node's
/// storage.
pub async fn get_coin_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<(Coin, OuterVerifier)> {
    let utxo = fetch_storage::<OuterVerifier>(output_ref, client).await?;
    let coin_in_storage: Coin = utxo.payload.extract()?;

    Ok((coin_in_storage, utxo.verifier))
}
