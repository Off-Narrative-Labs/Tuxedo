use serde::{Deserialize, Serialize};

use jsonrpsee::http_client::HttpClientBuilder;
use crate::money;
use sp_core::H256;

use crate::cli::MintCoinArgs;
use crate::original_get_db;

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9944";


use axum::{Json,http::HeaderMap};



#[derive(Debug, Deserialize)]
pub struct MintCoinsRequest {
    pub amount: u128,
    pub owner_public_key:String,
}

#[derive(Debug, Serialize)]
pub struct MintCoinsResponse {
    pub message: String,

    // Add any additional fields as needed
}

pub async fn mint_coins(body: Json<MintCoinsRequest>) -> Json<MintCoinsResponse> {
    let client_result = HttpClientBuilder::default().build(DEFAULT_ENDPOINT);
    let client = match client_result {
        Ok(client) => client,
        Err(err) => {
            return Json(MintCoinsResponse {
                message: format!("Error creating HTTP client: {:?}", err),
            });
        }
    };

     // Convert the hexadecimal string to bytes
    //let public_key_bytes = hex::decode(SHAWN_PUB_KEY).expect("Invalid hexadecimal string");
    let public_key_bytes = hex::decode(body.owner_public_key.as_str()).expect("Invalid hexadecimal string");
    
     // Convert the bytes to H256
    let public_key_h256 = H256::from_slice(&public_key_bytes);
    // Call the mint_coins function from your CLI wallet module
    match money::mint_coins(&client, MintCoinArgs {
        amount: body.amount,
        owner: public_key_h256,
    }).await {
        Ok(()) => Json(MintCoinsResponse {
            message: format!("Coins minted successfully"),
        }),
        Err(err) => Json(MintCoinsResponse {
            message: format!("Error minting coins: {:?}", err),
        }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCoinsResponse {
    pub message: String,
    pub coins:Option<Vec<(String, H256, u128)>>,
}

pub async fn get_all_coins() -> Json<GetCoinsResponse> {
    let db = original_get_db().await.expect("Error");

    match crate::sync::get_all_coins(&db) {
        Ok(all_coins) => {
            
            if !all_coins.is_empty() {
                return Json(GetCoinsResponse {
                    message: format!("Success: Found Coins"),
                    coins: Some(all_coins),
                });
            }
        },
        Err(_) => {
            return Json(GetCoinsResponse {
                message: format!("Error: Can't find coins"),
                coins: None,
            });
        }
    }

    Json(GetCoinsResponse {
        message: format!("Error: Can't find coins"),
        coins: None,
    })
}

use std::str::FromStr;
pub async fn get_owned_coins(headers: HeaderMap) -> Json<GetCoinsResponse> {
    let public_key_header = headers.get("owner_public_key").expect("public_key_header is missing");
    let public_key_h256 = H256::from_str(public_key_header.to_str().expect("Failed to convert to H256"));

    let db = original_get_db().await.expect("Error");

    match crate::sync::get_owned_coins(&db,&public_key_h256.unwrap()) {
        Ok(all_coins) => {
            
            if !all_coins.is_empty() {
                return Json(GetCoinsResponse {
                    message: format!("Success: Found Coins"),
                    coins: Some(all_coins),
                });
            }
        },
        Err(_) => {
            return Json(GetCoinsResponse {
                message: format!("Error: Can't find coins"),
                coins: None,
            });
        }
    }

    Json(GetCoinsResponse {
        message: format!("Error: Can't find coins"),
        coins: None,
    })
}
