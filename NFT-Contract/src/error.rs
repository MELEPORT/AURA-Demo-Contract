use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The amount of flowers left is not enough!")]
    NotEnoughAmount {},

    #[error("ID does not exist (id {id})")]
    IdNotExists { id: String },

    #[error("ID has been taken (id {id})")]
    IdTaken { id: String },
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
