use serde::{Deserialize, Serialize};


use jsonrpsee::http_client::HttpClientBuilder;



use crate::kitty;
use sp_core::H256;

use crate::cli::{CreateKittyArgs,
    DelistKittyFromSaleArgs, UpdateKittyNameArgs, UpdateKittyPriceArgs, 
    BuyKittyArgs, BreedKittyArgs};

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9944";
use crate::get_local_keystore;
use crate::sync_and_get_db;
use crate::original_get_db;
use crate::convert_output_ref_from_string;

use axum::{Json,http::HeaderMap};

use std::convert::Infallible;



use runtime::{
    kitties::{
        KittyData,
    },
    money::{Coin},
    tradable_kitties::{TradableKittyData},
    OuterVerifier, Transaction,
};
use tuxedo_core::types::OutputRef;
use tuxedo_core::types::Output;

#[derive(Debug, Deserialize)]
pub struct CreateKittyRequest {
    pub name: String,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct CreateKittyResponse {
    pub message: String,
    pub kitty:Option<KittyData>
    // Add any additional fields as needed
}

pub async fn create_kitty(body: Json<CreateKittyRequest>) -> Result<Json<CreateKittyResponse>, Infallible> {
    println!("create_kitties called ");
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(CreateKittyResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);

    match kitty::create_kitty(&db, &client, CreateKittyArgs {
        kitty_name: body.name.to_string(),
        owner: public_key_h256,
    }).await {
        Ok(Some(created_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = CreateKittyResponse {
                message: format!("Kitty created successfully"),
                kitty: Some(created_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(CreateKittyResponse {
            message: format!("Kitty creation failed: No data returned"),
            kitty:None,
        })),
        Err(err) => Ok(Json(CreateKittyResponse {
            message: format!("Error creating kitty: {:?}", err),
            kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Get kitty by DNA
////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct GetKittyByDnaResponse {
    pub message: String,
    pub kitty:Option<KittyData>,
}

pub async fn get_kitty_by_dna(headers: HeaderMap) -> Json<GetKittyByDnaResponse> {
    println!("Headers map = {:?}",headers);
    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");
    let db = original_get_db().await.expect("Error");
    let mut found_kitty: Option<(KittyData, OutputRef)> = None;

    if let Ok(Some((kitty_info, out_ref))) =
        crate::sync::get_kitty_from_local_db_based_on_dna(&db,dna_header)
    {
        found_kitty = Some((kitty_info, out_ref));
    }

    let response = match found_kitty {
        Some((kitty_info, _)) => GetKittyByDnaResponse {
            message: format!("Success: Found Kitty with DNA {:?}", dna_header),
            kitty: Some(kitty_info),
        },
        None => GetKittyByDnaResponse {
            message: format!("Error: Can't find Kitty with DNA {:?}", dna_header),
            kitty: None,
        },
    };

    Json(response)
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetTdKittyByDnaResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>,
}

pub async fn get_td_kitty_by_dna(headers: HeaderMap) -> Json<GetTdKittyByDnaResponse> {
    println!("Headers map = in td kitty {:?}",headers);
    let dna_header = headers
        .get("td-kitty-dna")
        .expect("Td-Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Td-Kitty DNA header");
    let db = original_get_db().await.expect("Error");
    let mut found_td_kitty: Option<(TradableKittyData, OutputRef)> = None;

    if let Ok(Some((td_kitty_info, out_ref))) =
        crate::sync::get_tradable_kitty_from_local_db_based_on_dna(&db,dna_header)
    {
        found_td_kitty = Some((td_kitty_info, out_ref));
    }

    let response = match found_td_kitty {
        Some((kitty_info, _)) => GetTdKittyByDnaResponse {
            message: format!("Success: Found Tradable Kitty with DNA {:?}", dna_header),
            td_kitty: Some(kitty_info),
        },
        None => GetTdKittyByDnaResponse {
            message: format!("Error: Can't find Tradable Kitty with DNA {:?}", dna_header),
            td_kitty: None,
        },
    };

    Json(response)
}

////////////////////////////////////////////////////////////////////
// Get all kitty List 
////////////////////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllKittiesResponse {
    pub message: String,
    pub kitty_list:Option<Vec<KittyData>>,
}

pub async fn get_all_kitty_list() -> Json<GetAllKittiesResponse> {
    let db = original_get_db().await.expect("Error");

    match crate::sync::get_all_kitties_from_local_db(&db) {
        Ok(all_kitties) => {
            let kitty_list: Vec<KittyData> = all_kitties.map(|(_, kitty)| kitty).collect();
            
            if !kitty_list.is_empty() {
                return Json(GetAllKittiesResponse {
                    message: format!("Success: Found Kitties"),
                    kitty_list: Some(kitty_list),
                });
            }
        },
        Err(_) => {
            return Json(GetAllKittiesResponse {
                message: format!("Error: Can't find Kitties"),
                kitty_list: None,
            });
        }
    }

    Json(GetAllKittiesResponse {
        message: format!("Error: Can't find Kitties"),
        kitty_list: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAllTdKittiesResponse {
    pub message: String,
    pub td_kitty_list:Option<Vec<TradableKittyData>>,
}

pub async fn get_all_td_kitty_list() -> Json<GetAllTdKittiesResponse> {
    let db = original_get_db().await.expect("Error");

    match crate::sync::get_all_tradable_kitties_from_local_db(&db) {
        Ok(owned_kitties) => {
            let tradable_kitty_list: Vec<TradableKittyData> = owned_kitties.map(|(_, kitty)| kitty).collect();
            
            if !tradable_kitty_list.is_empty() {
                return Json(GetAllTdKittiesResponse {
                    message: format!("Success: Found TradableKitties"),
                    td_kitty_list: Some(tradable_kitty_list),
                });
            }
        },
        Err(_) => {
            return Json(GetAllTdKittiesResponse {
                message: format!("Error: Can't find TradableKitties"),
                td_kitty_list: None,
            });
        }
    }

    Json(GetAllTdKittiesResponse {
        message: format!("Error: Can't find Kitties"),
        td_kitty_list: None,
    })
}
////////////////////////////////////////////////////////////////////
// Get owned kitties  
////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOwnedKittiesResponse {
    pub message: String,
    pub kitty_list:Option<Vec<KittyData>>,
}
use std::str::FromStr;
pub async fn get_owned_kitty_list(headers: HeaderMap) -> Json<GetOwnedKittiesResponse> {
    let public_key_header = headers.get("owner_public_key").expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header.to_str().expect("Failed to convert to H256"));

    let db = original_get_db().await.expect("Error");

    match crate::sync::get_owned_kitties_from_local_db(&db,&public_key_h256.unwrap()) {
        Ok(owned_kitties) => {
            let kitty_list: Vec<KittyData> = owned_kitties.map(|(_, kitty,_)| kitty).collect();
            
            if !kitty_list.is_empty() {
                return Json(GetOwnedKittiesResponse {
                    message: format!("Success: Found Kitties"),
                    kitty_list: Some(kitty_list),
                });
            }
        },
        Err(_) => {
            return Json(GetOwnedKittiesResponse {
                message: format!("Error: Can't find Kitties"),
                kitty_list: None,
            });
        }
    }

    Json(GetOwnedKittiesResponse {
        message: format!("Error: Can't find Kitties"),                    
        kitty_list: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOwnedTdKittiesResponse {
    pub message: String,
    pub td_kitty_list:Option<Vec<TradableKittyData>>,
}

pub async fn get_owned_td_kitty_list(headers: HeaderMap) -> Json<GetOwnedTdKittiesResponse> {
    let public_key_header = headers.get("owner_public_key").expect("public_key_header is missing");
    let public_key_h256 = H256::from_str(public_key_header.to_str().expect("Failed to convert to H256"));
    let db = original_get_db().await.expect("Error");

    match crate::sync::get_owned_tradable_kitties_from_local_db(&db,&public_key_h256.unwrap()) {
        Ok(owned_kitties) => {
            let tradable_kitty_list: Vec<TradableKittyData> = owned_kitties.map(|(_, kitty, _)| kitty).collect();
            
            if !tradable_kitty_list.is_empty() {
                return Json(GetOwnedTdKittiesResponse {
                    message: format!("Success: Found TradableKitties"),
                    td_kitty_list: Some(tradable_kitty_list),
                });
            }
        },
        Err(_) => {
            return Json(GetOwnedTdKittiesResponse {
                message: format!("Error: Can't find TradableKitties"),
                td_kitty_list: None,
            });
        }
    }

    Json(GetOwnedTdKittiesResponse {
        message: format!("Error: Can't find td Kitties"),
        td_kitty_list: None,
    })
}

////////////////////////////////////////////////////////////////////
// Common structures and functions 
////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnAndUtxoListForList {
    pub message: String,
    pub transaction: Option<Transaction>,
    pub input_utxo_list:Option<Vec<Output<OuterVerifier>>>,
}

#[derive(Debug, Deserialize)]
pub struct SignedTxnRequest {
    pub signed_transaction: Transaction,
}

async fn create_response(
    txn: Option<Transaction>,
    message: String,
) -> Json<GetTxnAndUtxoListForList> {
    match txn {
        Some(txn) => {
            let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
            let client = match client_result {
                Ok(client) => client,
                Err(err) => {
                    return Json(GetTxnAndUtxoListForList {
                        message: format!("Error creating HTTP client: {:?}", err),
                        transaction: None,
                        input_utxo_list: None,
                    });
                }
            };
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;
            Json(GetTxnAndUtxoListForList {
                message,
                transaction: Some(txn),
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            })
        },
        None => Json(GetTxnAndUtxoListForList {
            message,
            transaction: None,
            input_utxo_list: None,
        }),
    }
}


////////////////////////////////////////////////////////////////////
// List kitty for Sale 
////////////////////////////////////////////////////////////////////



pub async fn get_txn_and_inpututxolist_for_list_kitty_for_sale(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);

    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");

    let price_header = headers
        .get("kitty-price")
        .expect("Kitty price is missing");

    let price_number: u128 = price_header
        .to_str()
        .expect("Failed to parse priceheader")
        .parse()
        .expect("ailed to parse price number as u128");


    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
            .to_str()
            .expect("Failed to convert to H256"));

    let db = original_get_db().await.expect("Error");

    match kitty::create_txn_for_list_kitty(&db, 
        dna_header,
        price_number,
        public_key_h256.unwrap(),
    ).await {
        Ok(txn) => create_response(
            txn,
            "List kitty for Sale txn created successfully".to_string(),
        ).await,
        Err(err) => create_response(
            None,
            format!("Error!! List kitty for sale txn creation: {:?}", err),
        ).await,
    }
}

#[derive(Debug, Serialize)]
pub struct ListKittyForSaleResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
    // Add any additional fields as needed
}

pub async fn list_kitty_for_sale (body: Json<SignedTxnRequest>) -> Result<Json<ListKittyForSaleResponse>, Infallible> {
    println!("List kitties for sale is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(ListKittyForSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    match kitty::list_kitty_for_sale(&body.signed_transaction,
        &client).await {
        Ok(Some(listed_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = ListKittyForSaleResponse {
                message: format!("Kitty listed for sale successfully"),
                td_kitty: Some(listed_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(ListKittyForSaleResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(ListKittyForSaleResponse {
            message: format!("Error listing forsale: {:?}", err),
            td_kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// De-list kitty from Sale 
////////////////////////////////////////////////////////////////////


pub async fn get_txn_and_inpututxolist_for_delist_kitty_from_sale(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    // create_tx_for_list_kitty
    println!("Headers map = {:?}",headers);
    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");

    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
            .to_str()
            .expect("Failed to convert to H256"));
    
    let db = original_get_db().await.expect("Error");

    match kitty::create_txn_for_delist_kitty(&db,
        dna_header,
        public_key_h256.unwrap(),
    ).await {
        Ok(txn) => create_response(
            txn,
            "List kitty for Sale txn created successfully".to_string(),
        ).await,
        Err(err) => create_response(
            None,
            format!("Error!! List kitty for sale txn creation: {:?}", err),
        ).await,
    }
}

#[derive(Debug, Serialize)]
pub struct DelistKittyFromSaleResponse {
    pub message: String,
    pub kitty:Option<KittyData>
}
pub async fn delist_kitty_from_sale(body: Json<SignedTxnRequest>) -> Result<Json<DelistKittyFromSaleResponse>, Infallible> {
    println!("List kitties for sale is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(DelistKittyFromSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                kitty:None,
            }));
        }
    };

    match kitty::delist_kitty_from_sale(&body.signed_transaction,
        &client).await {
        Ok(Some(delisted_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = DelistKittyFromSaleResponse {
                message: format!("Kitty delisted from sale successfully"),
                kitty: Some(delisted_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(DelistKittyFromSaleResponse {
            message: format!("Kitty delisting from sale  failed: No data returned"),
            kitty:None,
        })),
        Err(err) => Ok(Json(DelistKittyFromSaleResponse {
            message: format!("Error delisting from sale: {:?}", err),
            kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Update kitty name 
////////////////////////////////////////////////////////////////////

pub async fn get_txn_and_inpututxolist_for_kitty_name_update(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);
    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");

    let new_name_header = headers
        .get("kitty-new-name")
        .expect("Kitty name is missing");

    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
        .to_str()
        .expect("Failed to convert to H256"));

    let db = original_get_db().await.expect("Error");

    match kitty::create_txn_for_kitty_name_update(&db, 
        dna_header,
        new_name_header.to_str().expect("Failed to parse name header").to_string(),
        public_key_h256.unwrap(),
    ).await {
        Ok(txn) => create_response(
            txn,
            "Kitty name update txn created successfully".to_string(),
        ).await,
        Err(err) => create_response(
            None,
            format!("Error!! Kitty name update txn creation: {:?}", err),
        ).await,
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateKittyNameResponse {
    pub message: String,
    pub kitty:Option<KittyData>
}
pub async fn update_kitty_name(body: Json<SignedTxnRequest>) -> Result<Json<UpdateKittyNameResponse>, Infallible> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateKittyNameResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                kitty:None,
            }));
        }
    };

    match kitty::update_kitty_name(&body.signed_transaction,
        &client).await {
        Ok(Some(updated_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateKittyNameResponse {
                message: format!("Kitty name updated successfully"),
                kitty: Some(updated_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateKittyNameResponse {
            message: format!("Kitty name update failed: No data returned"),
            kitty:None,
        })),
        Err(err) => Ok(Json(UpdateKittyNameResponse {
            message: format!("Error!! Kitty name update: {:?}", err),
            kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Update tradable kitty name 
////////////////////////////////////////////////////////////////////

pub async fn get_txn_and_inpututxolist_for_td_kitty_name_update(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);
    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");
    let db = original_get_db().await.expect("Error");

    let new_name_header = headers
        .get("kitty-new-name")
        .expect("Kitty name is missing");

    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
            .to_str()
            .expect("Failed to convert to H256"));

    match kitty::create_txn_for_td_kitty_name_update(&db, 
        dna_header,
        new_name_header.to_str().expect("Failed to parse name header").to_string(),
        public_key_h256.unwrap(),
    ).await {
        Ok(txn) => create_response(
            txn,
            "Td Kitty name update txn created successfully".to_string(),
        ).await,
        Err(err) => create_response(
            None,
            format!("Error!! Td-Kitty name update txn creation: {:?}", err),
        ).await,
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateTddKittyNameResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}
pub async fn update_td_kitty_name(body: Json<SignedTxnRequest>) -> Result<Json<UpdateTddKittyNameResponse>, Infallible> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateTddKittyNameResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    match kitty::update_td_kitty_name(&body.signed_transaction,
        &client).await {
        Ok(Some(updated_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateTddKittyNameResponse {
                message: format!("Td-Kitty name updated successfully"),
                td_kitty: Some(updated_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateTddKittyNameResponse {
            message: format!("Td-Kitty name update failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(UpdateTddKittyNameResponse {
            message: format!("Error!! Td-Kitty name update: {:?}", err),
            td_kitty:None,
        })),
    }
}


////////////////////////////////////////////////////////////////////
// Update td-kitty price
////////////////////////////////////////////////////////////////////

pub async fn get_txn_and_inpututxolist_for_td_kitty_price_update(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);
    let dna_header = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");

    let price_header = headers
        .get("kitty-price")
        .expect("Kitty price is missing");

    // Convert the block number to the appropriate type if needed
    let price_number: u128 = price_header
        .to_str()
        .expect("Failed to parse priceheader to str")
        .parse().expect("Failed to parse priceheader to u128");

    let db = original_get_db().await.expect("Error");

    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
            .to_str()
            .expect("Failed to convert to H256"));

    match kitty::create_txn_for_td_kitty_price_update(
        &db, 
        dna_header,
        price_number,
        public_key_h256.unwrap(),
    ).await {
        Ok(Some(txn)) => {
            // Convert created_kitty to JSON and include it in the response
            let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
            let client = match client_result {
                Ok(client) => client,
                Err(err) => {
                    return Json(GetTxnAndUtxoListForList {
                        message: format!("Error creating HTTP client: {:?}", err),
                        transaction:None,
                        input_utxo_list:None
                    });
                }
            };
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;

            let response = GetTxnAndUtxoListForList {
                message: format!("Kitty name update txn created successfully"),
                transaction: Some(txn), 
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnAndUtxoListForList {
            message: format!("Kitty name update txn creation failed: No input returned"),
            transaction:None,
            input_utxo_list:None
        }),
        Err(err) => Json(GetTxnAndUtxoListForList {
            message: format!("Error!! Kitty name update txn creation: {:?}", err),
            transaction:None,
            input_utxo_list:None
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateTdKittyPriceResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}

pub async fn update_td_kitty_price(body: Json<SignedTxnRequest>) -> Result<Json<UpdateTdKittyPriceResponse>, Infallible> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateTdKittyPriceResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    match kitty::update_td_kitty_price(&body.signed_transaction,
        &client).await {
        Ok(Some(updated_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateTdKittyPriceResponse {
                message: format!("Kitty price updated successfully"),
                td_kitty: Some(updated_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateTdKittyPriceResponse {
            message: format!("Kitty price update failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(UpdateTdKittyPriceResponse {
            message: format!("Error in kitty price update: {:?}", err),
            td_kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Breed kitty
////////////////////////////////////////////////////////////////////

pub async fn get_txn_and_inpututxolist_for_breed_kitty(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);
    let mom_dna = headers
        .get("mom-dna")
        .expect("MOM DNA header is missing")
        .to_str()
        .expect("Failed to parse MOM DNA header");

    let dad_dna = headers
        .get("dad-dna")
        .expect("Dad DNA header is missing")
        .to_str()
        .expect("Failed to parse Dad DNA header");

    let child_kitty_name = headers
        .get("child-kitty-name")
        .expect("Child Kitty name is missing");

    let db = original_get_db().await.expect("Error");

    let public_key_header = headers
        .get("owner_public_key")
        .expect("public_key_header is missing");

    let public_key_h256 = H256::from_str(public_key_header
            .to_str()
            .expect("Failed to convert to H256"));

    match kitty::create_txn_for_breed_kitty(
        &db, 
        mom_dna,
        dad_dna,
        child_kitty_name.to_str().expect("Failed to parse name header").to_string(),
        public_key_h256.unwrap(),
    ).await {
        Ok(Some(txn)) => {
            // Convert created_kitty to JSON and include it in the response
            let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
            let client = match client_result {
                Ok(client) => client,
                Err(err) => {
                    return Json(GetTxnAndUtxoListForList {
                        message: format!("Error creating HTTP client: {:?}", err),
                        transaction:None,
                        input_utxo_list:None
                    });
                }
            };
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;

            let response = GetTxnAndUtxoListForList {
                message: format!("Kitty name update txn created successfully"),
                transaction: Some(txn), 
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnAndUtxoListForList {
            message: format!("Kitty name update txn creation failed: No input returned"),
            transaction:None,
            input_utxo_list:None
        }),
        Err(err) => Json(GetTxnAndUtxoListForList {
            message: format!("Error!! Kitty name update txn creation: {:?}", err),
            transaction:None,
            input_utxo_list:None
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct BreedKittyResponse {
    pub message: String,
    pub mom_kitty:Option<KittyData>,
    pub dad_kitty:Option<KittyData>,
    pub child_kitty:Option<KittyData>,
}

pub async fn breed_kitty(body: Json<SignedTxnRequest>) -> Result<Json<BreedKittyResponse>, Infallible> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(BreedKittyResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                mom_kitty: None,
                dad_kitty: None, 
                child_kitty: None, 
            }));
        }
    };

    match kitty::breed_kitty(&body.signed_transaction,
        &client).await {
        Ok(Some(kitty_family)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = BreedKittyResponse {
                message: format!("Kitty breeding done successfully"),
                mom_kitty: Some(kitty_family[0].clone()),
                dad_kitty: Some(kitty_family[1].clone()), 
                child_kitty: Some(kitty_family[2].clone()), 

            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(BreedKittyResponse {
            message: format!("Kitty breeding failed: No data returned"),
            mom_kitty: None,
            dad_kitty: None, 
            child_kitty: None, 
        })),
        Err(err) => Ok(Json(BreedKittyResponse {
            message: format!("Error in kitty breed: {:?}", err),
            mom_kitty: None,
            dad_kitty: None, 
            child_kitty: None, 
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Buy kitty
////////////////////////////////////////////////////////////////////

pub async fn get_txn_and_inpututxolist_for_buy_kitty(headers: HeaderMap) -> Json<GetTxnAndUtxoListForList> {
    println!("Headers map = {:?}",headers);

    let input_coins: Vec<OutputRef> = headers
        .get_all("input-coins")
        .iter()
        // Convert each coin string to an OutputRef, filter out errors
        .filter_map(|header| {
            let coin_str = header.to_str().unwrap_or_default();
            match convert_output_ref_from_string(coin_str) {
                Ok(output_ref) => Some(output_ref),
                Err(err) => {
                    // Print error message and skip this coin
                    eprintln!("Error converting input coin: {}", err);
                    None
                }
            }
        })
        .collect();
        println!("Input coins: {:?}", input_coins);
    let output_amount: Vec<u128> = headers
        .get("output_amount")
        .map_or_else(|| Vec::new(), |header| {
            header
                .to_str()
                .unwrap_or_default()
                .split(',')
                .filter_map(|amount_str| amount_str.parse().ok())
                .collect()
        });
    // Use the converted coins Vec<OutputRef> as needed
        println!("output_amount: {:?}", output_amount);

    let kitty_dna = headers
        .get("kitty-dna")
        .expect("Kitty DNA header is missing")
        .to_str()
        .expect("Failed to parse Kitty DNA header");


    let db = original_get_db().await.expect("Error");

    let buyer_public_key = headers
        .get("buyer_public_key")
        .expect("buyer_public_key is missing");

    let buyer_public_key_h256 = H256::from_str(buyer_public_key
            .to_str()
            .expect("Failed to convert buyer_public_keyto H256"));

    let seller_public_key = headers
        .get("seller_public_key")
        .expect("seller_public_key is missing");

    let seller_public_key_h256 = H256::from_str(seller_public_key
            .to_str()
            .expect("Failed to convert seller_public_key to H256"));

    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Json(GetTxnAndUtxoListForList {
                message: format!("Error creating HTTP client: {:?}", err),
                transaction:None,
                input_utxo_list:None
            });
        }
    };

    match kitty::create_txn_for_buy_kitty(
        &db, 
        input_coins,
        &kitty_dna,
        buyer_public_key_h256.unwrap(),
        seller_public_key_h256.unwrap(),
        &output_amount,
        &client,
    ).await {
        Ok(Some(txn)) => {
            // Convert created_kitty to JSON and include it in the response
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;

            let response = GetTxnAndUtxoListForList {
                message: format!("Kitty name update txn created successfully"),
                transaction: Some(txn), 
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnAndUtxoListForList {
            message: format!("Kitty name update txn creation failed: No input returned"),
            transaction:None,
            input_utxo_list:None
        }),
        Err(err) => Json(GetTxnAndUtxoListForList {
            message: format!("Error!! Kitty name update txn creation: {:?}", err),
            transaction:None,
            input_utxo_list:None
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct BuyTdKittyResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}


pub async fn buy_kitty(body: Json<SignedTxnRequest>) -> Result<Json<BuyTdKittyResponse>, Infallible> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(BuyTdKittyResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    match kitty::buy_kitty(&body.signed_transaction,
        &client).await {
        Ok(Some(traded_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = BuyTdKittyResponse {
                message: format!("Kitty traded successfully"),
                td_kitty: Some(traded_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(BuyTdKittyResponse {
            message: format!("Kitty trade failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(BuyTdKittyResponse {
            message: format!("Error in trading: {:?}", err),
            td_kitty:None,
        })),
    }
}



/*
pub async fn breed_kitty(body: Json<BreedKittyRequest>) -> Result<Json<BreedKittyResponse>, Infallible> {
    println!("update_td_kitty_price is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let db = sync_and_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(BreedKittyResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                mom_kitty:None,
                dad_kitty:None,
                child_kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256_of_owner = H256::from_slice(&public_key_bytes);

    let ks = get_local_keystore().await.expect("Error");

    match kitty::breed_kitty(&db, &client, &ks,BreedKittyArgs {
        mom_name: body.mom_name.clone(),
        dad_name: body.dad_name.clone(),
        owner: public_key_h256_of_owner,
    }).await {
        Ok(Some(new_family)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = BreedKittyResponse {
                message: format!("breeding successfully"),
                mom_kitty:Some(new_family[0].clone()),
                dad_kitty:Some(new_family[1].clone()),
                child_kitty:Some(new_family[2].clone()),
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(BreedKittyResponse {
            message: format!("Error in breeding  failed: No data returned"),
            mom_kitty:None,
            dad_kitty:None,
            child_kitty:None,
        })),
        Err(err) => Ok(Json(BreedKittyResponse {
            message: format!("Error in breeding : {:?}", err),
            mom_kitty:None,
            dad_kitty:None,
            child_kitty:None,
        })),
    }
}
*/