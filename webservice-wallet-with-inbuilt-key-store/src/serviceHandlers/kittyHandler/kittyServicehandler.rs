use serde::{Deserialize, Serialize};


use jsonrpsee::http_client::HttpClientBuilder;
use parity_scale_codec::{Decode, Encode};
use std::path::PathBuf;
use sled::Db;
use crate::kitty;
use sp_core::H256;

use crate::cli::{CreateKittyArgs, ListKittyForSaleArgs, 
    DelistKittyFromSaleArgs, UpdateKittyNameArgs, UpdateKittyPriceArgs, 
    BuyKittyArgs, BreedKittyArgs};

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9944";
use crate::{ keystore::SHAWN_PUB_KEY};

use crate::get_db;
use crate::get_local_keystore;
use crate::sync_and_get_db;
use crate::original_get_db;



use axum::{http::StatusCode, response::IntoResponse, routing::{get, post, put, patch},Json, Router,http::HeaderMap};

use std::convert::Infallible;
use axum::{response::Html,};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use runtime::{opaque::Block as OpaqueBlock, Block};
use anyhow::bail;
//use parity_scale_codec::Input;
use tuxedo_core::types::Input;


use runtime::{
    kitties::{
        DadKittyStatus, FreeKittyConstraintChecker, KittyDNA, KittyData, KittyHelpers,
        MomKittyStatus, Parent,
    },
    money::{Coin, MoneyConstraintChecker},
    tradable_kitties::{TradableKittyConstraintChecker, TradableKittyData},
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
// List kitty for Sale 
////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnAndUtxoListForListKittyForSaleResponse {
    pub message: String,
    pub transaction: Option<Transaction>,
    pub input_utxo_list:Option<Vec<Output<OuterVerifier>>>,
}

pub async fn get_txn_and_inpututxolist_for_list_kitty_for_sale(headers: HeaderMap) -> Json<GetTxnAndUtxoListForListKittyForSaleResponse> {
    // create_tx_for_list_kitty
    println!("Headers map = {:?}",headers);
    let name_header = headers.get("kitty-name").unwrap_or_else(|| {
        panic!("Kitty name is missing");
    });

    let name_str = name_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse name header");
    });

    // -------------------------------
    let price_header = headers.get("kitty-price").unwrap_or_else(|| {
        panic!("Kitty price is missing");
    });
    let price_str = price_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse priceheader");
    });

    // Convert the block number to the appropriate type if needed
    let price_number: u128 = price_str.parse().unwrap_or_else(|_| {
        panic!("Failed to parse price number as u128");
    });

    // -------------------------------

    let publick_key_header = headers.get("owner_public_key").unwrap_or_else(|| {
        panic!("publick_key_header is missing");
    });

    let publick_key_str = publick_key_header.to_str().unwrap_or_else(|_| {
        panic!("publick_key_header to parse");
    });

    let public_key_bytes = hex::decode(publick_key_str.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);

    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");
    
    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Json(GetTxnAndUtxoListForListKittyForSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                transaction:None,
                input_utxo_list:None
            });
        }
    };

    match kitty::create_txn_for_list_kitty(&db, 
        name_str.to_string(),
        price_number,
        public_key_h256,
    ).await {
        Ok(Some(txn)) => {
            // Convert created_kitty to JSON and include it in the response
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;

            let response = GetTxnAndUtxoListForListKittyForSaleResponse {
                message: format!("Kitty listed for sale successfully"),
                transaction: Some(txn), 
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnAndUtxoListForListKittyForSaleResponse {
            message: format!("Kitty listing forsale  failed: No input returned"),
            transaction:None,
            input_utxo_list:None
        }),
        Err(err) => Json(GetTxnAndUtxoListForListKittyForSaleResponse {
            message: format!("Error!! listing forsale: {:?}", err),
            transaction:None,
            input_utxo_list:None
        }),
    }
}

pub async fn debug_get_signed_txn_for_list_kitty_for_sale(headers: HeaderMap) -> Json<GetTxnAndUtxoListForListKittyForSaleResponse> {
    // create_tx_for_list_kitty
    println!("Headers map = {:?}",headers);
    let name_header = headers.get("kitty-name").unwrap_or_else(|| {
        panic!("Kitty name is missing");
    });

    let name_str = name_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse name header");
    });

    // -------------------------------
    let price_header = headers.get("kitty-price").unwrap_or_else(|| {
        panic!("Kitty price is missing");
    });
    let price_str = price_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse priceheader");
    });

    // Convert the block number to the appropriate type if needed
    let price_number: u128 = price_str.parse().unwrap_or_else(|_| {
        panic!("Failed to parse price number as u128");
    });

    // -------------------------------

    let publick_key_header = headers.get("owner_public_key").unwrap_or_else(|| {
        panic!("publick_key_header is missing");
    });

    let publick_key_str = publick_key_header.to_str().unwrap_or_else(|_| {
        panic!("publick_key_header to parse");
    });

    let public_key_bytes = hex::decode(publick_key_str.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);

    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");
    
    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Json(GetTxnAndUtxoListForListKittyForSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                transaction:None,
                input_utxo_list:None
            });
        }
    };
    match kitty::create_signed_txn_for_list_kitty(&db, 
        name_str.to_string(),
        price_number,
        public_key_h256,
        &client,
    ).await {
        Ok(Some(txn)) => {
            // Convert created_kitty to JSON and include it in the response
            let utxo_list = kitty::create_inpututxo_list(&mut txn.clone(),&client).await;

            let response = GetTxnAndUtxoListForListKittyForSaleResponse {
                message: format!("Kitty listed for sale successfully"),
                transaction: Some(txn), 
                input_utxo_list:utxo_list.expect("Cant crate the Utxo List"),
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnAndUtxoListForListKittyForSaleResponse {
            message: format!("Kitty listing forsale  failed: No input returned"),
            transaction:None,
            input_utxo_list:None
        }),
        Err(err) => Json(GetTxnAndUtxoListForListKittyForSaleResponse {
            message: format!("Error!! listing forsale: {:?}", err),
            transaction:None,
            input_utxo_list:None
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListKittyForSaleRequest {
    pub signed_transaction: Transaction,
}

#[derive(Debug, Serialize)]
pub struct ListKittyForSaleResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
    // Add any additional fields as needed
}

pub async fn list_kitty_for_sale (body: Json<ListKittyForSaleRequest>) -> Result<Json<ListKittyForSaleResponse>, Infallible> {
    println!("List kitties for sale is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");
    
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
        &db, &client).await {
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

#[derive(Debug, Deserialize)]
pub struct DelistKittyFromSaleRequest {
    pub name: String,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct DelistKittyFromSaleResponse {
    pub message: String,
    pub kitty:Option<KittyData>
}
pub async fn delist_kitty_from_sale (body: Json<DelistKittyFromSaleRequest>) -> Result<Json<DelistKittyFromSaleResponse>, Infallible> {
    println!("delist_kitty_from_sale is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(DelistKittyFromSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    let ks = get_local_keystore().await.expect("Error");

    match kitty::delist_kitty_from_sale(&db, &client, &ks,DelistKittyFromSaleArgs {
        name: body.name.to_string(),
        owner: public_key_h256,
    }).await {
        Ok(Some(delisted_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = DelistKittyFromSaleResponse {
                message: format!("Kitty listed for sale successfully"),
                kitty: Some(delisted_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(DelistKittyFromSaleResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            kitty:None,
        })),
        Err(err) => Ok(Json(DelistKittyFromSaleResponse {
            message: format!("Error listing forsale: {:?}", err),
            kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Update kitty name 
////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct UpdateKittyNameRequest {
    pub current_name: String,
    pub new_name:String,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct UpdateKittyNameResponse {
    pub message: String,
    pub kitty:Option<KittyData>
}
pub async fn update_kitty_name(body: Json<UpdateKittyNameRequest>) -> Result<Json<UpdateKittyNameResponse>, Infallible> {
    println!("update_kitty_name is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let db = sync_and_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateKittyNameResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    let ks = get_local_keystore().await.expect("Error");

    match kitty::update_kitty_name(&db, &client, &ks,UpdateKittyNameArgs {
        current_name: body.current_name.to_string(),
        new_name: body.new_name.to_string(),
        owner: public_key_h256,
    }).await {
        Ok(Some(updated_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateKittyNameResponse {
                message: format!("Kitty listed for sale successfully"),
                kitty: Some(updated_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateKittyNameResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            kitty:None,
        })),
        Err(err) => Ok(Json(UpdateKittyNameResponse {
            message: format!("Error listing forsale: {:?}", err),
            kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Update tradable kitty name 
////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct UpdateTdKittyNameRequest {
    pub current_name: String,
    pub new_name:String,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct UpdateTdKittyNameResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}
pub async fn update_td_kitty_name(body: Json<UpdateTdKittyNameRequest>) -> Result<Json<UpdateTdKittyNameResponse>, Infallible> {
    println!("update_td_kitty_name is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let db = sync_and_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateTdKittyNameResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    let ks = get_local_keystore().await.expect("Error");

    match kitty::update_td_kitty_name(&db, &client, &ks,UpdateKittyNameArgs {
        current_name: body.current_name.to_string(),
        new_name: body.new_name.to_string(),
        owner: public_key_h256,
    }).await {
        Ok(Some(updated_td_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateTdKittyNameResponse {
                message: format!("Kitty listed for sale successfully"),
                td_kitty: Some(updated_td_kitty), // Include the created kitty in the response
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateTdKittyNameResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(UpdateTdKittyNameResponse {
            message: format!("Error listing forsale: {:?}", err),
            td_kitty:None,
        })),
    }
}

////////////////////////////////////////////////////////////////////
// Update tradable kitty price 
////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct UpdateTdKittyPriceRequest {
    pub current_name: String,
    pub price:u128,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct UpdateTdKittyPriceResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}

pub async fn update_td_kitty_price(body: Json<UpdateTdKittyPriceRequest>) -> Result<Json<UpdateTdKittyPriceResponse>, Infallible> {
    println!("update_td_kitty_price is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let db = sync_and_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(UpdateTdKittyPriceResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    let ks = get_local_keystore().await.expect("Error");

    match kitty::update_kitty_price(&db, &client, &ks,UpdateKittyPriceArgs {
        current_name: body.current_name.to_string(),
        price: body.price,
        owner: public_key_h256,
    }).await {
        Ok(Some(updated_td_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = UpdateTdKittyPriceResponse {
                message: format!("Kitty listed for sale successfully"),
                td_kitty: Some(updated_td_kitty),
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(UpdateTdKittyPriceResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(UpdateTdKittyPriceResponse {
            message: format!("Error listing forsale: {:?}", err),
            td_kitty:None,
        })),
    }
}


////////////////////////////////////////////////////////////////////
// Buy kitty
////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct BuyTdKittyRequest {
    pub input_coins: Vec<OutputRef>,
    pub kitty_name: String,
    pub owner_public_key:String,
    pub seller_public_key:String,
    pub output_amount: Vec<u128>,
}

#[derive(Debug, Serialize)]
pub struct BuyTdKittyResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
}

pub async fn buy_kitty(body: Json<BuyTdKittyRequest>) -> Result<Json<BuyTdKittyResponse>, Infallible> {
    println!("update_td_kitty_price is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let db = sync_and_get_db().await.expect("Error");

    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(BuyTdKittyResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256_of_owner = H256::from_slice(&public_key_bytes);

    let public_key_bytes = hex::decode(body.seller_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256_of_seller = H256::from_slice(&public_key_bytes);

    let ks = get_local_keystore().await.expect("Error");

    match kitty::buy_kitty(&db, &client, &ks,BuyKittyArgs {
        input: body.input_coins.clone(),
        kitty_name: body.kitty_name.clone(),
        seller: public_key_h256_of_seller,
        owner: public_key_h256_of_owner,
        output_amount: body.output_amount.clone(),
    }).await {
        Ok(Some(updated_td_kitty)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = BuyTdKittyResponse {
                message: format!("Kitty listed for sale successfully"),
                td_kitty: Some(updated_td_kitty),
            };
            Ok(Json(response))
        },
        Ok(None) => Ok(Json(BuyTdKittyResponse {
            message: format!("Kitty listing forsale  failed: No data returned"),
            td_kitty:None,
        })),
        Err(err) => Ok(Json(BuyTdKittyResponse {
            message: format!("Error listing forsale: {:?}", err),
            td_kitty:None,
        })),
    }
}


////////////////////////////////////////////////////////////////////
// Breed kitty
////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
pub struct BreedKittyRequest {
    pub mom_name: String,
    pub dad_name: String,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct BreedKittyResponse {
    pub message: String,
    pub mom_kitty:Option<KittyData>,
    pub dad_kitty:Option<KittyData>,
    pub child_kitty:Option<KittyData>,
}

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

/*

#[derive(Debug, Deserialize)]
pub struct ListKittyForSaleRequest {
    pub name: String,
    pub price: u128,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct ListKittyForSaleResponse {
    pub message: String,
    pub td_kitty:Option<TradableKittyData>
    // Add any additional fields as needed
}

pub async fn list_kitty_for_sale (body: Json<ListKittyForSaleRequest>) -> Result<Json<ListKittyForSaleResponse>, Infallible> {
    println!("List kitties for sale is called {:?}",body);
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    //let db = sync_and_get_db().await.expect("Error");
    let db = original_get_db().await.expect("Error");
    
    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Ok(Json(ListKittyForSaleResponse {
                message: format!("Error creating HTTP client: {:?}", err),
                td_kitty:None,
            }));
        }
    };

    // Convert the hexadecimal string to bytes
    let public_key_bytes = hex::decode(body.owner_public_key.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    let ks = get_local_keystore().await.expect("Error");

    match kitty::list_kitty_for_sale(&db, &client, &ks,ListKittyForSaleArgs {
        name: body.name.to_string(),
        price: body.price,
        owner: public_key_h256,
    }).await {
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

///////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct GetTxnForListKittyForSaleResponse {
    pub message: String,
    pub list_kitty_inputs:Option<Vec<Input>>
}

pub async fn get_inputs_for_list_kitty_for_sale(headers: HeaderMap) -> Json<GetTxnForListKittyForSaleResponse> {
    // create_tx_for_list_kitty
    let name_header = headers.get("kitty-name").unwrap_or_else(|| {
        panic!("Kitty name is missing");
    });

    let name_str = name_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse name header");
    });

    // -------------------------------
    let price_header = headers.get("kitty-price").unwrap_or_else(|| {
        panic!("Kitty price is missing");
    });
    let price_str = price_header.to_str().unwrap_or_else(|_| {
        panic!("Failed to parse priceheader");
    });

    // Convert the block number to the appropriate type if needed
    let price_number: u128 = price_str.parse().unwrap_or_else(|_| {
        panic!("Failed to parse price number as u128");
    });

    // -------------------------------

    let publick_key_header = headers.get("kitty-name").unwrap_or_else(|| {
        panic!("publick_key_header is missing");
    });

    let publick_key_str = publick_key_header.to_str().unwrap_or_else(|_| {
        panic!("publick_key_header to parse");
    });

    let public_key_bytes = hex::decode(publick_key_str.clone()).expect("Invalid hexadecimal string");
    let public_key_h256 = H256::from_slice(&public_key_bytes);

    let db = original_get_db().await.expect("Error");

    match kitty::create_inputs_for_list_kitty(&db, 
        name_str.to_string(),
       // price_number,
        public_key_h256,
    ).await {
        Ok(Some(inputs)) => {
            // Convert created_kitty to JSON and include it in the response
            let response = GetTxnForListKittyForSaleResponse {
                message: format!("Kitty listed for sale successfully"),
                list_kitty_inputs: Some(inputs), // Include the created kitty in the response
            };
            Json(response)
        },
        Ok(None) => Json(GetTxnForListKittyForSaleResponse {
            message: format!("Kitty listing forsale  failed: No input returned"),
            list_kitty_inputs:None,
        }),
        Err(err) => Json(GetTxnForListKittyForSaleResponse {
            message: format!("Error listing forsale: {:?}", err),
            list_kitty_inputs:None,
        }),
    }
}

*/