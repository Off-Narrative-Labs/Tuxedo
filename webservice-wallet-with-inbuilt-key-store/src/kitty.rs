use crate::money::get_coin_from_storage;
use crate::rpc::fetch_storage;
use crate::sync;

//use crate::cli::BreedArgs;
use tuxedo_core::{
    dynamic_typing::UtxoData,
    types::{Input, Output, OutputRef},
    verifier::Sr25519Signature,
};

use anyhow::anyhow;
use jsonrpsee::{core::client::ClientT, http_client::HttpClient, rpc_params};
use parity_scale_codec::Encode;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sc_keystore::LocalKeystore;
use sled::Db;
use sp_core::sr25519::Public;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use crate::get_local_keystore;

use runtime::{
    kitties::{
        DadKittyStatus, FreeKittyConstraintChecker, KittyDNA, KittyData, KittyHelpers,
        MomKittyStatus, Parent,
    },
    money::{Coin, MoneyConstraintChecker},
    tradable_kitties::{TradableKittyConstraintChecker, TradableKittyData},
    OuterVerifier, Transaction,
};

use crate::cli::DEFAULT_KITTY_NAME;

use crate::cli::{
    BreedKittyArgs, BuyKittyArgs, CreateKittyArgs, DelistKittyFromSaleArgs, ListKittyForSaleArgs,
    UpdateKittyNameArgs, UpdateKittyPriceArgs,
};
use parity_scale_codec::Decode;

static MALE_KITTY_NAMES: [&str; 50] = [
    "Whis", "Purr", "Feli", "Leo", "Maxi", "Osca", "Simb", "Char", "Milo", "Tige", "Jasp",
    "Smokey", "Oliv", "Loki", "Boot", "Gizm", "Rock", "Budd", "Shad", "Zeus", "Tedd", "Samm",
    "Rust", "Tom", "Casp", "Blue", "Coop", "Coco", "Mitt", "Bent", "Geor", "Luck", "Rome", "Moch",
    "Muff", "Ches", "Whis", "Appl", "Hunt", "Toby", "Finn", "Frod", "Sale", "Kobe", "Dext", "Jinx",
    "Mick", "Pump", "Thor", "Sunn",
];

// Female kitty names
static FEMALE_KITTY_NAMES: [&str; 50] = [
    "Luna", "Mist", "Cleo", "Bell", "Lucy", "Nala", "Zoe", "Dais", "Lily", "Mia", "Chlo", "Stel",
    "Coco", "Will", "Ruby", "Grac", "Sash", "Moll", "Lola", "Zara", "Mist", "Ange", "Rosi", "Soph",
    "Zeld", "Layl", "Ambe", "Prin", "Cind", "Moch", "Zara", "Dais", "Cinn", "Oliv", "Peac", "Pixi",
    "Harl", "Mimi", "Pipe", "Whis", "Cher", "Fion", "Kiki", "Suga", "Peac", "Ange", "Mapl", "Zigg",
    "Lily", "Nova",
];

pub fn generate_random_string(length: usize) -> String {
    let rng = rand::thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    random_string
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

fn create_tx_input_based_on_td_kittyName(
    db: &Db,
    name: String,
) -> anyhow::Result<(TradableKittyData, Input)> {
    let tradable_kitty =
        crate::sync::get_tradable_kitty_from_local_db_based_on_name(&db, name.clone());
    let Some((tradable_kitty_info, out_ref)) = tradable_kitty.unwrap() else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let input = Input {
        output_ref: out_ref,
        redeemer: vec![], // We will sign the total transaction so this should be empty
    };
    Ok((tradable_kitty_info, input))
}

fn create_tx_input_based_on_kitty_name<'a>(
    db: &Db,
    name: String,
) -> anyhow::Result<(KittyData, Input)> {
    //let kitty = crate::sync::get_kitty_from_local_db_based_on_name(&db, name.clone());
    //let name_extractor_kitty_data = move |kitty: &'a KittyData| -> &'a [u8; 4] { &kitty.name };

    let kitty = crate::sync::get_kitty_from_local_db_based_on_name(&db, name.clone());
    let Some((kitty_info, out_ref)) = kitty.unwrap() else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let input = Input {
        output_ref: out_ref,
        redeemer: vec![], // We will sign the total transaction so this should be empty
    };
    Ok((kitty_info, input))
}

fn create_tx_input_based_on_td_kitty_name<'a>(
    db: &Db,
    name: String,
) -> anyhow::Result<(TradableKittyData, Input)> {
    let kitty = crate::sync::get_tradable_kitty_from_local_db_based_on_name(&db, name.clone());
    let Some((kitty_info, out_ref)) = kitty.unwrap() else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let input = Input {
        output_ref: out_ref,
        redeemer: vec![], // We will sign the total transaction so this should be empty
    };
    Ok((kitty_info, input))
}

fn print_new_output(transaction: &Transaction) -> anyhow::Result<()> {
    let tx_hash = <BlakeTwo256 as Hash>::hash_of(&transaction.encode());
    for (i, output) in transaction.outputs.iter().enumerate() {
        let new_ref = OutputRef {
            tx_hash,
            index: i as u32,
        };
        match output.payload.type_id {
            KittyData::TYPE_ID => {
                let new_kitty = output.payload.extract::<KittyData>()?.dna.0;
                print!(
                    "Created {:?} basic Kitty {:?}. ",
                    hex::encode(new_ref.encode()),
                    new_kitty
                );
            }
            TradableKittyData::TYPE_ID => {
                let new_kitty = output
                    .payload
                    .extract::<TradableKittyData>()?
                    .kitty_basic_data
                    .dna
                    .0;
                print!(
                    "Created {:?} TradableKitty {:?}. ",
                    hex::encode(new_ref.encode()),
                    new_kitty
                );
            }
            Coin::<0>::TYPE_ID => {
                let amount = output.payload.extract::<Coin<0>>()?.0;
                print!(
                    "Created {:?} worth {amount}. ",
                    hex::encode(new_ref.encode())
                );
            }

            _ => continue,
        }
        crate::pretty_print_verifier(&output.verifier);
    }
    Ok(())
}

async fn send_signed_tx(
    transaction: &Transaction,
    client: &HttpClient,
) -> anyhow::Result<()> {

    // Encode the transaction
    let spawn_hex = hex::encode(transaction.encode());
    let params = rpc_params![spawn_hex];
    let spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;
    
    println!("Node's response to spawn transaction: {:?}", spawn_response);
    match spawn_response {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow::Error::msg(format!("Error sending transaction: {:?}", err))),
    }
}

async fn send_tx(
    transaction: &mut Transaction,
    client: &HttpClient,
    local_keystore: Option<&LocalKeystore>,
) -> anyhow::Result<()> {
    // Keep a copy of the stripped encoded transaction for signing purposes
    let stripped_encoded_transaction = transaction.clone().encode();

    let _ = match local_keystore {
        Some(ks) => {
            // Iterate back through the inputs, signing, and putting the signatures in place.
            for input in &mut transaction.inputs {
                // Fetch the output from storage
                let utxo = fetch_storage::<OuterVerifier>(&input.output_ref, client).await?;

                // Construct the proof that it can be consumed
                let redeemer = match utxo.verifier {
                    OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
                        let public = Public::from_h256(owner_pubkey);
                        crate::keystore::sign_with(ks, &public, &stripped_encoded_transaction)?
                    }
                    OuterVerifier::UpForGrabs(_) => Vec::new(),
                    OuterVerifier::ThresholdMultiSignature(_) => todo!(),
                };
                // insert the proof
                input.redeemer = redeemer;
            }
        }
        None => {}
    };

    // Encode the transaction
    let spawn_hex = hex::encode(transaction.encode());
    let params = rpc_params![spawn_hex];
    let spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;
    
    println!("Node's response to spawn transaction: {:?}", spawn_response);
    match spawn_response {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow::Error::msg(format!("Error sending transaction: {:?}", err))),
    }
}



fn gen_random_gender() -> Gender {
    // Create a local random number generator
    let mut rng = rand::thread_rng();

    // Generate a random number between 0 and 1
    let random_number = rng.gen_range(0..=1);

    // We Use the random number to determine the gender
    match random_number {
        0 => Gender::Male,
        _ => Gender::Female,
    }
}

fn get_random_kitty_name_from_pre_defined_vec(g: Gender, name_slice: &mut [u8; 4]) {
    // Create a local random number generator
    let mut rng = rand::thread_rng();

    // Generate a random number between 0 and 1
    let random_number = rng.gen_range(0..=50);

    // We Use the random number to determine the gender

    let name = match g {
        Gender::Male => MALE_KITTY_NAMES[random_number],
        Gender::Female => FEMALE_KITTY_NAMES[random_number],
    };
    //name.to_string()
    name_slice.copy_from_slice(name.clone().as_bytes());
}

fn validate_kitty_name_from_db(
    db: &Db,
    owner_pubkey: &H256,
    name: String,
    name_slice: &mut [u8; 4],
) -> anyhow::Result<()> {
    if name.len() != 4 {
        return Err(anyhow!(
            "Please input a name of length 4 characters. Current length: {}",
            name.len()
        ));
    }

    match crate::sync::is_kitty_name_duplicate(&db, name.clone(), &owner_pubkey) {
        Ok(Some(true)) => {
            println!("Kitty name is duplicate , select another name");
            return Err(anyhow!(
                "Please input a non-duplicate name of length 4 characters"
            ));
        }
        _ => {}
    };
    name_slice.copy_from_slice(name.clone().as_bytes());

    return Ok(());
}

fn create_new_family(
    new_mom: &mut KittyData,
    new_dad: &mut KittyData,
    new_child: &mut KittyData,
) -> anyhow::Result<()> {
    new_mom.parent = Parent::Mom(MomKittyStatus::RearinToGo);
    if new_mom.num_breedings >= 2 {
        new_mom.parent = Parent::Mom(MomKittyStatus::HadBirthRecently);
    }
    new_mom.num_breedings = new_mom.num_breedings.checked_add(1).expect("REASON");
    new_mom.free_breedings = new_mom.free_breedings.checked_sub(1).expect("REASON");

    new_dad.parent = Parent::Dad(DadKittyStatus::RearinToGo);
    if new_dad.num_breedings >= 2 {
        new_dad.parent = Parent::Dad(DadKittyStatus::Tired);
    }

    new_dad.num_breedings = new_dad.num_breedings.checked_add(1).expect("REASON");
    new_dad.free_breedings = new_dad.free_breedings.checked_sub(1).expect("REASON");

    let child_gender = match gen_random_gender() {
        Gender::Male => Parent::dad(),
        Gender::Female => Parent::mom(),
    };

    let child = KittyData {
        parent: child_gender,
        free_breedings: 2,
        name: *b"tomy", // Name of child kitty need to be generated in better way
        dna: KittyDNA(BlakeTwo256::hash_of(&(
            new_mom.dna.clone(),
            new_dad.dna.clone(),
            new_mom.num_breedings,
            new_dad.num_breedings,
        ))),
        num_breedings: 0,
        // price: None,
        //  is_available_for_sale: false,
    };
    *new_child = child;
    Ok(())
}

pub async fn create_kitty(
    db: &Db,
    client: &HttpClient,
    args: CreateKittyArgs,
) -> anyhow::Result<Option<KittyData>> {
    let mut kitty_name = [0; 4];

    let g = gen_random_gender();
    let gender = match g {
        Gender::Male => Parent::dad(),
        Gender::Female => Parent::mom(),
    };

    if args.kitty_name.clone() == DEFAULT_KITTY_NAME.to_string() {
        get_random_kitty_name_from_pre_defined_vec(g, &mut kitty_name);
    } else {
        validate_kitty_name_from_db(&db, &args.owner, args.kitty_name.clone(), &mut kitty_name)?;
    }
    // Generate a random string of length 5
    let random_string = generate_random_string(5) + args.kitty_name.as_str();
    let dna_preimage: &[u8] = random_string.as_bytes();

    let child_kitty = KittyData {
        parent: gender,
        dna: KittyDNA(BlakeTwo256::hash(dna_preimage)),
        name: kitty_name, // Default value for now, as the provided code does not specify a value
        ..Default::default()
    };

    // Create the Output
    let output = Output {
        payload: child_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };
    // Create the Transaction
    let mut transaction = Transaction {
        inputs: Vec::new(),
        peeks: Vec::new(),
        outputs: vec![output],
        checker: FreeKittyConstraintChecker::Create.into(),
    };

    send_tx(&mut transaction, &client, None).await?;
    print_new_output(&transaction)?;
    Ok(Some(child_kitty))
}

pub async fn list_kitty_for_sale(
    signed_transaction: &Transaction,
    db: &Db,
    client: &HttpClient,
) -> anyhow::Result<Option<TradableKittyData>> {
    let tradable_kitty: TradableKittyData = signed_transaction.outputs[0].payload
        .extract::<TradableKittyData>()
        .map_err(|_| anyhow!("Invalid output, Expected TradableKittyData"))?;
    log::info!("The list_kitty_for_sale args : {:?}", signed_transaction);
    send_signed_tx(&signed_transaction, &client).await?;
    print_new_output(&signed_transaction)?;
    Ok(Some(tradable_kitty))
}

pub async fn delist_kitty_from_sale(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: DelistKittyFromSaleArgs,
) -> anyhow::Result<Option<KittyData>> {
    log::info!("The list_kitty_for_sale args : {:?}", args);
    let Ok((td_kitty_info, input)) = create_tx_input_based_on_td_kittyName(db, args.name.clone())
    else {
        return Err(anyhow!("No kitty with name {} in localdb", args.name));
    };

    let inputs: Vec<Input> = vec![input];
    let basic_kitty = td_kitty_info.kitty_basic_data;

    // Create the Output
    let output = Output {
        payload: basic_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::DelistKittiesFromSale.into(),
    };

    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(Some(basic_kitty))
}

pub async fn breed_kitty(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: BreedKittyArgs,
) -> anyhow::Result<Option<Vec<KittyData>>> {
    log::info!("The Breed kittyArgs are:: {:?}", args);

    let Ok((mom_kitty_info, mom_ref)) =
        create_tx_input_based_on_kitty_name(db, args.mom_name.clone())
    else {
        return Err(anyhow!("No kitty with name {} in localdb", args.mom_name));
    };

    let Ok((dad_kitty_info, dad_ref)) =
        create_tx_input_based_on_kitty_name(db, args.dad_name.clone())
    else {
        return Err(anyhow!("No kitty with name {} in localdb", args.dad_name));
    };

    let inputs: Vec<Input> = vec![mom_ref, dad_ref];

    let mut new_mom: KittyData = mom_kitty_info;

    let mut new_dad = dad_kitty_info;

    let mut child: KittyData = Default::default();

    create_new_family(&mut new_mom, &mut new_dad, &mut child)?;
    // Create the Output mom
    println!("New mom Dna = {:?}", new_mom.dna);
    println!("New Dad Dna = {:?}", new_dad.dna);
    println!("Child Dna = {:?}", child.dna);

    let output_mom = Output {
        payload: new_mom.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Output dada
    let output_dad = Output {
        payload: new_dad.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    let output_child = Output {
        payload: child.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    let new_family = Box::new(vec![output_mom, output_dad, output_child]);

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: (&[
            new_family[0].clone(),
            new_family[1].clone(),
            new_family[2].clone(),
        ])
            .to_vec(),
        checker: FreeKittyConstraintChecker::Breed.into(),
    };

    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(vec![
        new_mom.clone(),
        new_dad.clone(),
        child.clone(),
    ].into())
}

pub async fn buy_kitty(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: BuyKittyArgs,
)  -> anyhow::Result<Option<TradableKittyData>> {
    log::info!("The Buy kittyArgs are:: {:?}", args);

    let Ok((kitty_info, kitty_ref)) =
        create_tx_input_based_on_td_kittyName(db, args.kitty_name.clone())
    else {
        return Err(anyhow!("No kitty with name {} in localdb", args.kitty_name));
    };

    let inputs: Vec<Input> = vec![kitty_ref];
    // Create the KittyData
    let mut output_kitty = kitty_info.clone();

    let output = Output {
        payload: output_kitty.into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::Buy.into(),
    };

    // Construct each output and then push to the transactions for Money
    let mut total_output_amount = 0;
    for amount in &args.output_amount {
        let output = Output {
            payload: Coin::<0>::new(*amount).into(),
            verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
                owner_pubkey: args.seller,
            }),
        };
        total_output_amount += amount;
        transaction.outputs.push(output);
        if total_output_amount >= kitty_info.price.into() {
            break;
        }
    }

    let mut total_input_amount = 0;
    let mut all_input_refs = args.input;
    for output_ref in &all_input_refs {
        let (_owner_pubkey, amount) = sync::get_unspent(db, output_ref)?.ok_or(anyhow!(
            "user-specified output ref not found in local database"
        ))?;
        total_input_amount += amount;
    }

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
    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;

    Ok(Some(kitty_info))
}

pub async fn update_td_kitty_name(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: UpdateKittyNameArgs,
) -> anyhow::Result<Option<TradableKittyData>>  {
    log::info!("The set_kitty_property are:: {:?}", args);
    
    let Ok((td_kitty_info, td_kitty_ref)) = create_tx_input_based_on_td_kitty_name(db, args.current_name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb",args.current_name));
    };
    
    let mut array = [0; 4];
    let kitty_name: &[u8; 4] = {
        array.copy_from_slice(args.new_name.as_bytes());
        &array
    };

    let inputs: Vec<Input> = vec![td_kitty_ref.clone()];
    let mut updated_kitty: TradableKittyData = td_kitty_info;
    updated_kitty.kitty_basic_data.name = *kitty_name;
    let output = Output {
        payload: updated_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::UpdateKittiesName.into(),
    };

    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(Some(updated_kitty))
}

pub async fn update_kitty_name(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: UpdateKittyNameArgs,
) -> anyhow::Result<Option<KittyData>> {
    log::info!("The set_kitty_property are:: {:?}", args);

    let Ok((input_kitty_info, input_kitty_ref)) = create_tx_input_based_on_kitty_name(db,args.current_name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb",args.current_name));
    };

    let mut array = [0; 4];
    let kitty_name: &[u8; 4] = {
        array.copy_from_slice(args.new_name.as_bytes());
        &array
    };

    let inputs: Vec<Input> = vec![input_kitty_ref];

    let mut updated_kitty: KittyData = input_kitty_info.clone();
    updated_kitty.name = *kitty_name;
    let output = Output {
        payload: updated_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: FreeKittyConstraintChecker::UpdateKittiesName.into(),
    };

    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(Some(updated_kitty))
}

pub async fn update_kitty_price(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: UpdateKittyPriceArgs,
) -> anyhow::Result<Option<TradableKittyData>> {
    log::info!("The set_kitty_property are:: {:?}", args);
    let Ok((input_kitty_info, input_kitty_ref)) =
        create_tx_input_based_on_td_kitty_name(db, args.current_name.clone())
    else {
        return Err(anyhow!(
            "No kitty with name {} in localdb",
            args.current_name
        ));
    };

    let inputs: Vec<Input> = vec![input_kitty_ref];
    let mut updated_kitty: TradableKittyData = input_kitty_info;
    updated_kitty.price = args.price;
    let output = Output {
        payload: updated_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::UpdateKittiesPrice.into(),
    };

    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(Some(updated_kitty))
}

/// Apply a transaction to the local database, storing the new coins.
pub(crate) fn apply_transaction(
    db: &Db,
    tx_hash: <BlakeTwo256 as Hash>::Output,
    index: u32,
    output: &Output<OuterVerifier>,
) -> anyhow::Result<()> {
    let kitty_detail: KittyData = output.payload.extract()?;

    let output_ref = OutputRef { tx_hash, index };
    println!("in kitty:apply_transaction output_ref = {:?}", output_ref);
    match output.verifier {
        OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
            // Add it to the global unspent_outputs table
            crate::sync::add_fresh_kitty_to_db(db, &output_ref, &owner_pubkey, &kitty_detail)
        }
        _ => Err(anyhow!("{:?}", ())),
    }
}

pub(crate) fn convert_kitty_name_string(kitty: &KittyData) -> Option<String> {
    if let Ok(kitty_name) = std::str::from_utf8(&kitty.name) {
        return Some(kitty_name.to_string());
    } else {
        println!("Invalid UTF-8 data in the Kittyname");
    }
    None
}

/// Given an output ref, fetch the details about this coin from the node's
/// storage.
// Need to revisit this for tradableKitty
pub async fn get_kitty_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<(KittyData, OuterVerifier)> {
    let utxo = fetch_storage::<OuterVerifier>(output_ref, client).await?;

    let kitty_in_storage: KittyData = utxo.payload.extract()?;

    Ok((kitty_in_storage, utxo.verifier))
}

/// Apply a transaction to the local database, storing the new coins.
pub(crate) fn apply_td_transaction(
    db: &Db,
    tx_hash: <BlakeTwo256 as Hash>::Output,
    index: u32,
    output: &Output<OuterVerifier>,
) -> anyhow::Result<()> {
    let tradable_kitty_detail: TradableKittyData = output.payload.extract()?;

    let output_ref = OutputRef { tx_hash, index };
    println!(
        "in Tradable kitty:apply_transaction output_ref = {:?}",
        output_ref
    );
    match output.verifier {
        OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
            // Add it to the global unspent_outputs table
            crate::sync::add_fresh_tradable_kitty_to_db(
                db,
                &output_ref,
                &owner_pubkey,
                &tradable_kitty_detail,
            )
        }
        _ => Err(anyhow!("{:?}", ())),
    }
}

pub(crate) fn convert_td_kitty_name_string(tradable_kitty: &TradableKittyData) -> Option<String> {
    if let Ok(kitty_name) = std::str::from_utf8(&tradable_kitty.kitty_basic_data.name) {
        return Some(kitty_name.to_string());
    } else {
        println!("Invalid UTF-8 data in the Kittyname");
    }
    None
}

/// Given an output ref, fetch the details about this coin from the node's
/// storage.
pub async fn get_td_kitty_from_storage(
    output_ref: &OutputRef,
    client: &HttpClient,
) -> anyhow::Result<(TradableKittyData, OuterVerifier)> {
    let utxo = fetch_storage::<OuterVerifier>(output_ref, client).await?;
    let kitty_in_storage: TradableKittyData = utxo.payload.extract()?;

    Ok((kitty_in_storage, utxo.verifier))
}

//////////////////////////////////////////////////////////////////
// Below is for web server handling 
//////////////////////////////////////////////////////////////////


async fn send_tx_with_signed_inputs(
    transaction: &mut Transaction,
    client: &HttpClient,
    local_keystore: Option<&LocalKeystore>,
    signed_inputs: Vec<Input>,
) -> anyhow::Result<()> {
    // Keep a copy of the stripped encoded transaction for signing purposes
    let stripped_encoded_transaction = transaction.clone().encode();
    transaction.inputs = signed_inputs;

    let _ = match local_keystore {
        Some(ks) => {
            // Iterate back through the inputs, signing, and putting the signatures in place.
            for input in &mut transaction.inputs {
                // Fetch the output from storage
                let utxo = fetch_storage::<OuterVerifier>(&input.output_ref, client).await?;

                // Construct the proof that it can be consumed
                let redeemer = match utxo.verifier {
                    OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
                        let public = Public::from_h256(owner_pubkey);
                        crate::keystore::sign_with(ks, &public, &stripped_encoded_transaction)?
                    }
                    OuterVerifier::UpForGrabs(_) => Vec::new(),
                    OuterVerifier::ThresholdMultiSignature(_) => todo!(),
                };
                // insert the proof
                input.redeemer = redeemer;
            }
        }
        None => {}
    };

    // Encode the transaction
    let spawn_hex = hex::encode(transaction.encode());
    let params = rpc_params![spawn_hex];
    let spawn_response: Result<String, _> = client.request("author_submitExtrinsic", params).await;
    
    println!("Node's response to spawn transaction: {:?}", spawn_response);
    match spawn_response {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow::Error::msg(format!("Error sending transaction: {:?}", err))),
    }
}

pub async fn create_txn_for_list_kitty(
    db: &Db,
    name: String,
    price:u128,
    publick_key:H256,
) -> anyhow::Result<Option<Transaction>> {
    // Need to filter based on name and publick key.
    // Ideally need to filter based on DNA.
    let Ok((kitty_info, input)) = create_tx_input_based_on_kitty_name(db, name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let inputs: Vec<Input> = vec![input];

    let tradable_kitty = TradableKittyData {
        kitty_basic_data: kitty_info,
        price: price,
    };

    // Create the Output
    let output = Output {
        payload: tradable_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: publick_key,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::ListKittiesForSale.into(),
    };

    Ok(Some(transaction))
}

pub async fn create_inpututxo_list(
    transaction: &mut Transaction,
    client: &HttpClient,
) -> anyhow::Result<Option<Vec<Output<OuterVerifier>>>> {
    let mut utxo_list: Vec<Output<OuterVerifier>> = Vec::new();
    for input in &mut transaction.inputs {
        // Fetch the output from storage
        let utxo = fetch_storage::<OuterVerifier>(&input.output_ref, client).await?;
        utxo_list.push(utxo);
    }
    Ok(Some(utxo_list))
}

// Below function will not be used in real usage , it is just for test purpose

pub async fn create_signed_txn_for_list_kitty(
    db: &Db,
    name: String,
    price:u128,
    publick_key:H256,
    client: &HttpClient,
) -> anyhow::Result<Option<Transaction>> {
    // Need to filter based on name and publick key.
    // Ideally need to filter based on DNA.
    let Ok((kitty_info, input)) = create_tx_input_based_on_kitty_name(db, name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let inputs: Vec<Input> = vec![input];

    let tradable_kitty = TradableKittyData {
        kitty_basic_data: kitty_info,
        price: price,
    };

    // Create the Output
    let output = Output {
        payload: tradable_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: publick_key,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::ListKittiesForSale.into(),
    };

    let stripped_encoded_transaction = transaction.clone().encode();
    let local_keystore = get_local_keystore().await.expect("Key store error");

    for input in &mut transaction.inputs {
        // Fetch the output from storage
        let utxo = fetch_storage::<OuterVerifier>(&input.output_ref, client).await?;

        // Construct the proof that it can be consumed
        let redeemer = match utxo.verifier {
            OuterVerifier::Sr25519Signature(Sr25519Signature { owner_pubkey }) => {
                let public = Public::from_h256(owner_pubkey);
                crate::keystore::sign_with(&local_keystore, &public, &stripped_encoded_transaction)?
            }
            OuterVerifier::UpForGrabs(_) => Vec::new(),
            OuterVerifier::ThresholdMultiSignature(_) => todo!(),
        };
        // insert the proof
        input.redeemer = redeemer;
    }

    Ok(Some(transaction))
}


/*
pub async fn create_inputs_for_list_kitty(
    db: &Db,
    name: String,
    publick_key:H256,
) -> anyhow::Result<Option<Vec<Input>>> {
    // Need to filter based on name and publick key.
    // Ideally need to filter based on DNA.
    let Ok((kitty_info, input)) = create_tx_input_based_on_kitty_name(db, name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let inputs: Vec<Input> = vec![input];

    Ok(Some(inputs))
}
pub async fn list_kitty_for_sale_based_on_inputs(
    db: &Db,
    name: String,
    price: u128,
    inputs: Vec<Input>,
    public_key:H256,
) -> anyhow::Result<Option<Vec<Input>>> {
    // Need to filter based on name and publick key.
    // Ideally need to filter based on DNA.
    let Ok((kitty_info, input)) = create_tx_input_based_on_kitty_name(db, name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb", name));
    };

    let inputs: Vec<Input> = vec![input];


    Ok(Some(inputs))
}
pub async fn list_kitty_for_sale(
    db: &Db,
    client: &HttpClient,
    keystore: &LocalKeystore,
    args: ListKittyForSaleArgs,
) -> anyhow::Result<Option<TradableKittyData>> {
    log::info!("The list_kitty_for_sale args : {:?}", args);

    let Ok((kitty_info, input)) = create_tx_input_based_on_kitty_name(db, args.name.clone()) else {
        return Err(anyhow!("No kitty with name {} in localdb", args.name));
    };

    let inputs: Vec<Input> = vec![input];

    let tradable_kitty = TradableKittyData {
        kitty_basic_data: kitty_info,
        price: args.price,
    };

    // Create the Output
    let output = Output {
        payload: tradable_kitty.clone().into(),
        verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: args.owner,
        }),
    };

    // Create the Transaction
    let mut transaction = Transaction {
        inputs: inputs,
        peeks: Vec::new(),
        outputs: vec![output],
        checker: TradableKittyConstraintChecker::ListKittiesForSale.into(),
    };
    send_tx(&mut transaction, &client, Some(&keystore)).await?;
    print_new_output(&transaction)?;
    Ok(Some(tradable_kitty))
}
*/