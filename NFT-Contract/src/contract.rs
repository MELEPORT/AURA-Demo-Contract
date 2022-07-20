use std::f32::consts::E;
use erased_serde::{Serialize, Serializer};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, CosmosMsg, Empty, WasmMsg
};

use cosmwasm_std::Addr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, FlowerInfoResponse, InstantiateMsg, QueryMsg};
use cw721_base::{ExecuteMsg as Cw721ExecuteMsg, InstantiateMsg as Cw721InstantiateMsg, MintMsg};
use crate::state::{store, store_query, Flower, CONFIG, CW721_ADDRESS};
use cosmwasm_std::{Order, Record};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let flower = Flower {
        token_id: "0".to_string(),
        owner: _info.sender.to_string(),
        title: msg.title,
        description: msg.description,
        media: msg.media,
        total_nfts: msg.total_nfts,
        price: msg.price,
    };
    let key = flower.token_id.as_bytes();
    store(deps.storage).save(key, &flower)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddNew {
            token_id,
            owner,
            title,
            description,
            media,
            total_nfts,
            price,
        } => add_new(deps, token_id, owner, title, description, media, total_nfts, price),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, recipient, token_id),
        ExecuteMsg::SwapNft {
            owner,
            recipient,
            owner_token_id,
            recipient_token_id,
        } => execute_swap_nft(deps, owner, recipient, owner_token_id, recipient_token_id),
        ExecuteMsg::Sell { token_id, total_nfts } => sell(deps, token_id, total_nfts),
        ExecuteMsg::SetPrice { token_id, price } => set_price(deps, token_id, price),
    }
}

pub fn execute_swap_nft(deps: DepsMut, onwer:String, recipient: String, owner_token_id: String, recipient_token_id: String) -> Result<Response, ContractError> {
    let key_owner = owner_token_id.as_bytes();
    store(deps.storage).update(key_owner, |record_owner| {
        if let Some(mut record_owner) = record_owner {
            record_owner.owner = recipient;
            Ok(record_owner)
        } else {
            Err(ContractError::IdNotExists { id: owner_token_id.clone() })
        }
    })?;

    let key_recipient = recipient_token_id.as_bytes();
    store(deps.storage).update(key_recipient, |record_recipient| {
        if let Some(mut record_recipient) = record_recipient {
            record_recipient.owner = onwer;
            Ok(record_recipient)
        } else {
            Err(ContractError::IdNotExists { id: owner_token_id.clone() })
        }
    })?;

    Ok(Response::new().add_attribute("method", "swap_nft"))
}

pub fn execute_transfer_nft(deps: DepsMut, recipient: String, token_id: String) -> Result<Response, ContractError> {
    let key = token_id.as_bytes();
    store(deps.storage).update(key, |record| {
        if let Some(mut record) = record {
            record.owner = recipient;
            record.price = 0;
            Ok(record)
        } else {
            Err(ContractError::IdNotExists { id: token_id.clone() })
        }
    })?;

    Ok(Response::new().add_attribute("method", "transfer_nft"))
}

pub fn set_price(deps: DepsMut, token_id: String, price: i32) -> Result<Response, ContractError> {
    let key = token_id.as_bytes();
    store(deps.storage).update(key, |record| {
        if let Some(mut record) = record {
            record.price = price;
            Ok(record)
        } else {
            Err(ContractError::IdNotExists { id: token_id.clone() })
        }
    })?;

    Ok(Response::new().add_attribute("method", "set_price"))
}

pub fn add_new(
    deps: DepsMut,
    token_id: String,
    owner: String,
    title: String,
    description: String,
    media: String,
    total_nfts: i32,
    price: i32,
) -> Result<Response, ContractError> {
    let flower = Flower {
        token_id,
        owner,
        title,
        description,
        media,
        total_nfts,
        price,
    };
    let key = flower.token_id.as_bytes();
    if (store(deps.storage).may_load(key)?).is_some() {
        // token_id is already taken
        return Err(ContractError::IdTaken { id: flower.token_id });
    }
    store(deps.storage).save(key, &flower)?;
    Ok(Response::new()
        .add_attribute("method", "add_new")
        .add_attribute("id", flower.token_id))
}

pub fn sell(deps: DepsMut, id: String, total_nfts: i32) -> Result<Response, ContractError> {
    let key = id.as_bytes();
    store(deps.storage).update(key, |record| {
        if let Some(mut record) = record {
            if total_nfts > record.total_nfts {
                //The total_nfts of flowers left is not enough
                return Err(ContractError::NotEnoughAmount {});
            }
            record.total_nfts -= total_nfts;
            Ok(record)
        } else {
            Err(ContractError::IdNotExists { id: id.clone() })
        }
    })?;

    Ok(Response::new().add_attribute("method", "sell"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetFlower { token_id } => query_flower(deps, token_id),
        QueryMsg::GetAllFlowers { token_id } => query_all_flowers(deps, token_id),
    }
}

fn query_flower(deps: Deps, id: String) -> StdResult<Binary> {
    let key = id.as_bytes();
    let flower = match store_query(deps.storage).may_load(key)? {
        Some(flower) => Some(flower),
        None => None,
    };

    let resp = FlowerInfoResponse { flower };
    to_binary(&resp)
}

fn query_all_flowers(deps: Deps, id: String) -> StdResult<Binary> {
    // let key = id.as_bytes();
    // let start = Some(&10);
    // let flower = store_query(deps.storage).range(Some("0".as_bytes()), Some("10".as_bytes()), Order::Ascending);
    // to_binary(&flower)
    let key = id.as_bytes();
    let flower = match store_query(deps.storage).may_load(key)? {
        Some(flower) => Some(flower),
        None => return Err(StdError::generic_err("Flower does not exist")),
    };

    let resp = FlowerInfoResponse { flower };
    to_binary(&resp)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary, StdError};

    // #[test]
    // fn initialization() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     let msg = InstantiateMsg {
    //         title: "rose".to_string(),
    //         amount: 10,
    //         price: 10,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());
    //     // it worked, let's query the flower
    //     let res = query(
    //         deps.as_ref(),
    //         mock_env(),
    //         QueryMsg::GetFlower {
    //             token_id: "0".to_string(),
    //         },
    //     )
    //     .unwrap();
    //     let flower = Flower {
    //         token_id: "0".to_string(),
    //         owner: "lily_id".to_string(),
    //         title: "rose".to_string(),
    //         amount: 10,
    //         price: 10,
    //     };
    //     let expected = FlowerInfoResponse {
    //         flower: Some(flower),
    //     };
    //     let value: FlowerInfoResponse = from_binary(&res).unwrap();
    //     assert_eq!(expected, value);
    // }

    // #[test]
    // fn not_works_with_add_new_id_existed() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    //     let lily_id = "lily_id";
    //     let msg_asiatic = ExecuteMsg::AddNew {
    //         token_id: lily_id.to_string(),
    //         owner: lily_id.to_string(),
    //         title: "Asiatic lilies".to_string(),
    //         amount: 100,
    //         price: 100,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     // we can just call .unwrap() to assert this was a success
    //     let res = execute(deps.as_mut(), mock_env(), info, msg_asiatic).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let msg_oriental = ExecuteMsg::AddNew {
    //         token_id: lily_id.to_string(),
    //         owner: lily_id.to_string(),
    //         title: "Oriental lilies".to_string(),
    //         amount: 100,
    //         price: 100,
    //     };
    //     let err = execute(deps.as_mut(), mock_env(), info, msg_oriental).unwrap_err();
    //     match err {
    //         ContractError::IdTaken { id } => {
    //             assert_eq!(lily_id.to_string(), id);
    //         }
    //         e => panic!("unexpected error: {}", e),
    //     }
    // }

    // #[test]
    // fn not_works_with_sell() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    //     let lily_id = "lily_id";
    //     let msg_add_new = ExecuteMsg::AddNew {
    //         token_id: lily_id.to_string(),
    //         owner: lily_id.to_string(),
    //         title: "Asiatic lilies".to_string(),
    //         amount: 100,
    //         price: 100,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     // we can just call .unwrap() to assert this was a success
    //     let res = execute(deps.as_mut(), mock_env(), info, msg_add_new).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     let msg_sell = ExecuteMsg::Sell {
    //         token_id: "lily_id".to_string(),
    //         amount: 101,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let err = execute(deps.as_mut(), mock_env(), info, msg_sell).unwrap_err();
    //     match err {
    //         ContractError::NotEnoughAmount {} => {}
    //         e => panic!("unexpected error: {}", e),
    //     }
    // }

    // #[test]
    // fn not_works_with_query() {
    //     let ref deps = mock_dependencies_with_balance(&coins(2, "token"));
    //     let err = query(
    //         deps.as_ref(),
    //         mock_env(),
    //         QueryMsg::GetFlower {
    //             token_id: "not_existed_id".to_string(),
    //         },
    //     );
    //     match err {
    //         Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "Flower does not exist"),
    //         Err(e) => panic!("Unexpected error: {:?}", e),
    //         _ => panic!("Must return error"),
    //     }
    // }

    // #[test]
    // fn works_with_add_new_and_sell() {
    //     let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    //     let msg = ExecuteMsg::AddNew {
    //         token_id: "lily_id".to_string(),
    //         owner: "lily_id".to_string(),
    //         title: "lily".to_string(),
    //         amount: 100,
    //         price: 100,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     // we can just call .unwrap() to assert this was a success
    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());
    //     // it worked, let's query the flower
    //     let res = query(
    //         deps.as_ref(),
    //         mock_env(),
    //         QueryMsg::GetFlower {
    //             token_id: "lily_id".to_string(),
    //         },
    //     )
    //     .unwrap();
    //     let flower = Flower {
    //         token_id: "lily_id".to_string(),
    //         owner: "lily_id".to_string(),
    //         title: "lily".to_string(),
    //         amount: 100,
    //         price: 100,
    //     };
    //     let expected = FlowerInfoResponse {
    //         flower: Some(flower),
    //     };
    //     let value: FlowerInfoResponse = from_binary(&res).unwrap();
    //     assert_eq!(expected, value);

    //     let msg = ExecuteMsg::Sell {
    //         token_id: "lily_id".to_string(),
    //         amount: 45,
    //     };
    //     let info = mock_info("creator", &coins(1000, "earth"));
    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());
    //     // it worked, let's query the flower
    //     let res = query(
    //         deps.as_ref(),
    //         mock_env(),
    //         QueryMsg::GetFlower {
    //             token_id: "lily_id".to_string(),
    //         },
    //     )
    //     .unwrap();
    //     let flower = Flower {
    //         token_id: "lily_id".to_string(),
    //         owner: "lily_id".to_string(),
    //         title: "lily".to_string(),
    //         amount: 55,
    //         price: 100,
    //     };
    //     let expected = FlowerInfoResponse {
    //         flower: Some(flower),
    //     };
    //     let value: FlowerInfoResponse = from_binary(&res).unwrap();
    //     assert_eq!(expected, value);
    // }
// }
