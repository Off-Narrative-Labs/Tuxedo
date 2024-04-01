use serde::{Deserialize, Serialize};

use sp_core::H256;
use crate::keystore;

use crate::{ keystore::SHAWN_PUB_KEY};

use axum::{http::StatusCode, response::IntoResponse, routing::{get, post},Json, Router};
use axum::{response::Html,};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use runtime::{opaque::Block as OpaqueBlock, Block};
use anyhow::bail;


#[derive(Debug, Deserialize)]
pub struct GenerateKeyRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GenerateKeyResponse {
    pub message: String,
    pub public_key: Option<String>,
    pub phrase: Option<String>,
}

pub async fn generate_key(body: Json<GenerateKeyRequest>) -> Json<GenerateKeyResponse> {
    match keystore::generate_key(body.password.clone()).await {
        Ok((public_key, phrase)) => Json(GenerateKeyResponse {
            message: format!("Keys generated successfully"),
            public_key: Some(public_key),
            phrase: Some(phrase),
        }),
        Err(err) => Json(GenerateKeyResponse {
            message: format!("Error generating keys: {:?}", err),
            public_key: None,
            phrase: None,
        }),
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// get keys 
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize)]
pub struct GetKeyResponse {
    pub message: String,
    pub keys: Option<Vec<String>>,
}

pub async fn get_keys() -> Json<GetKeyResponse> {
    match keystore::get_keys().await {
        Ok(keys_iter) => {
            // Lets collect keys into a vector of strings
            let keys: Vec<String> = keys_iter.map(|key| hex::encode(key)).collect();

            Json(GetKeyResponse {
                message: format!("Keys retrieved successfully"),
                keys: Some(keys),
            })
        }
        Err(err) => Json(GetKeyResponse {
            message: format!("Error retrieving keys: {:?}", err),
            keys: None,
        }),
    }
}
