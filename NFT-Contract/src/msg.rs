use crate::state::Flower;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub title: String,
    pub description: String,
    pub media: String,
    pub total_nfts: i32,
    pub price: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddNew {
        token_id: String,
        owner: String,
        title: String,
        description: String,
        media: String,
        total_nfts: i32,
        price: i32,
    },
    Sell {
        token_id: String,
        total_nfts: i32,
    },
    SwapNft {
        owner: String,
        recipient: String,
        owner_token_id: String,
        recipient_token_id: String,
    },
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SetPrice { 
        token_id: String, 
        price: i32 
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetFlower returns the flower's information
    GetFlower { token_id: String },
    GetAllFlowers { token_id: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FlowerInfoResponse {
    pub flower: Option<Flower>,
}
