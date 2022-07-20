use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Storage, Addr};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use cw_storage_plus::{Item, Map};

static STORE_KEY: &[u8] = b"store";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Flower {
    pub token_id: String,
    pub owner: String,
    pub title: String,
    pub description: String,
    pub media: String,
    pub total_nfts: i32,
    pub price: i32,
}

pub const CONFIG: Item<Flower> = Item::new("config");
pub const CW721_ADDRESS: Item<Addr> = Item::new("cw721_address");

pub fn store(storage: &mut dyn Storage) -> Bucket<Flower> {
    bucket(storage, STORE_KEY)
}

pub fn store_query(storage: &dyn Storage) -> ReadonlyBucket<Flower> {
    bucket_read(storage, STORE_KEY)
}
