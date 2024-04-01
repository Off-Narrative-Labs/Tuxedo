use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize)]
pub struct MintCoinsRequest {
    pub amount: u32,
}

#[derive(Debug, Serialize)]
pub struct MintCoinsResponse {
    pub message: String,
    // Add any additional fields as needed
}

#[derive(Debug, Deserialize)]
pub struct CreateKittyRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateKittyResponse {
    pub message: String,
    // Add any additional fields as needed
}