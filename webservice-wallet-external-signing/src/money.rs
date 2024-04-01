//! Wallet features related to spending money and checking balances.

use crate::{cli::MintCoinArgs, cli::SpendArgs,rpc::fetch_storage, sync};

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
    verifier::Sr25519Signature,
};
use crate::original_get_db;

/// Create and send a transaction that mints the coins on the network
pub async fn mint_coins(client: &HttpClient, args: MintCoinArgs) -> anyhow::Result<()> {
    log::debug!("The args are:: {:?}", args);

    let transaction = Transaction {
        inputs: Vec::new(),
        peeks: Vec::new(),
        outputs: vec![(
            Coin::<0>::new(args.amount),
            OuterVerifier::Sr25519Signature(Sr25519Signature {
                owner_pubkey: args.owner,
            }),
        )
            .into()],
        checker: OuterConstraintChecker::Money(MoneyConstraintChecker::Mint),
    };

    let spawn_hex = hex::encode(transaction.encode());
    let params = rpc_params![spawn_hex];
    let _spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;

    log::info!(
        "Node's response to mint-coin transaction: {:?}",
        _spawn_response
    );

    let minted_coin_ref = OutputRef {
        tx_hash: <BlakeTwo256 as Hash>::hash_of(&transaction.encode()),
        index: 0,
    };
    let output = &transaction.outputs[0];
    let amount = output.payload.extract::<Coin<0>>()?.0;
    print!(
        "Minted {:?} worth {amount}. ",
        hex::encode(minted_coin_ref.encode())
    );
    crate::pretty_print_verifier(&output.verifier);

    Ok(())
}
use sp_core::H256;
struct RecipientOutput {
    pub recipient:H256,
    pub output_amount:Vec<u128>
}
fn extract_recipient_list_from_args(args: SpendArgs,) -> Vec<RecipientOutput> {
    let mut recipient_list:Vec<RecipientOutput> = Vec::new();
    for i in args.recipients {
        let rec_pient = RecipientOutput {
            recipient:i.0,
            output_amount:i.1,
        };
        recipient_list.push(rec_pient);
    }
    recipient_list
}
/// Create and send a transaction that spends coins on the network
pub async fn spend_coins(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: SpendArgs,
) -> anyhow::Result<()> {
    
    log::info!("In the spend_coins_to_multiple_recipient The args are:: {:?}", args);
    let mut transaction = Transaction {
        inputs: Vec::new(),
        peeks: Vec::new(),
        outputs: Vec::new(),
        checker: OuterConstraintChecker::Money(MoneyConstraintChecker::Spend),
    };

    let recipient_list:Vec<RecipientOutput> = extract_recipient_list_from_args(args.clone());

    let mut total_output_amount = 0;
    for recipient in &recipient_list {
        for amount in &recipient.output_amount {
            let output = Output {
                payload: Coin::<0>::new(*amount).into(),
                verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
                    owner_pubkey: recipient.recipient,
                }),
            };
            total_output_amount += amount;
            transaction.outputs.push(output);
        }
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

    // If the supplied inputs are not valuable enough to cover the output amount
    // we select the rest arbitrarily from the local db. (In many cases, this will be all the inputs.)
    if total_input_amount < total_output_amount {
        match sync::get_arbitrary_unspent_set(db, total_output_amount - total_input_amount)? {
            Some(more_inputs) => {
                all_input_refs.extend(more_inputs);
            }
            None => Err(anyhow!(
                "Not enough value in database to construct transaction"
            ))?,
        }
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
            OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
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
    log::info!(
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
        let amount = output.payload.extract::<Coin<0>>()?.0;

        print!(
            "Created {:?} worth {amount}. ",
            hex::encode(new_coin_ref.encode())
        );
        crate::pretty_print_verifier(&output.verifier);
    }

    Ok(())
}
/*
/// Create and send a transaction that spends coins on the network
pub async fn spend_coins1(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: SpendArgs,
) -> anyhow::Result<()> {
    log::info!("The args are:: {:?}", args);

    // Construct a template Transaction to push coins into later
    let mut transaction = Transaction {
        inputs: Vec::new(),
        peeks: Vec::new(),
        outputs: Vec::new(),
        checker: OuterConstraintChecker::Money(MoneyConstraintChecker::Spend),
    };

    // Construct each output and then push to the transactions
    let mut total_output_amount = 0;
    for amount in &args.output_amount {
        let output = Output {
            payload: Coin::<0>::new(*amount).into(),
            verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
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

    // If the supplied inputs are not valuable enough to cover the output amount
    // we select the rest arbitrarily from the local db. (In many cases, this will be all the inputs.)
    if total_input_amount < total_output_amount {
        match sync::get_arbitrary_unspent_set(db, total_output_amount - total_input_amount)? {
            Some(more_inputs) => {
                all_input_refs.extend(more_inputs);
            }
            None => Err(anyhow!(
                "Not enough value in database to construct transaction"
            ))?,
        }
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
            OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
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
    log::info!(
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
        let amount = output.payload.extract::<Coin<0>>()?.0;

        print!(
            "Created {:?} worth {amount}. ",
            hex::encode(new_coin_ref.encode())
        );
        crate::pretty_print_verifier(&output.verifier);
    }

    Ok(())
}
*/

/// Given an output ref, fetch the details about this coin from the node's
/// storage.
pub async fn get_coin_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<(Coin<0>, OuterVerifier)> {
    let utxo = fetch_storage::<OuterVerifier>(output_ref, client).await?;
    let coin_in_storage: Coin<0> = utxo.payload.extract()?;

    Ok((coin_in_storage, utxo.verifier))
}

/// Apply a transaction to the local database, storing the new coins.
pub(crate) fn apply_transaction(
    db: &Db,
    tx_hash: <BlakeTwo256 as Hash>::Output,
    index: u32,
    output: &Output<OuterVerifier>,
) -> anyhow::Result<()> {
    let amount = output.payload.extract::<Coin<0>>()?.0;
    let output_ref = OutputRef { tx_hash, index };
    match output.verifier {
        OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
            // Add it to the global unspent_outputs table
            crate::sync::add_unspent_output(db, &output_ref, &owner_pubkey, &amount)
        }
        _ => Err(anyhow!("{:?}", ())),
    }
}
