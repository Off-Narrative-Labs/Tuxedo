use serde::{Deserialize, Serialize};

use jsonrpsee::http_client::HttpClientBuilder;
use parity_scale_codec::{Decode, Encode};
use runtime::OuterVerifier;
use std::path::PathBuf;
use sled::Db;
use crate::money;
use sp_core::H256;

use crate::cli::MintCoinArgs;

/// The default RPC endpoint for the wallet to connect to
const DEFAULT_ENDPOINT: &str = "http://localhost:9944";
use crate::{ keystore::SHAWN_PUB_KEY};


use axum::{http::StatusCode, response::IntoResponse, routing::{get, post},Json, Router};
use axum::{response::Html,};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use runtime::{opaque::Block as OpaqueBlock, Block};
use anyhow::bail;


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