use thiserror::Error;

use crate::chain::error::ChainError;

pub use serde_json::Error as SerdeJsonError;

#[derive(Error, Debug)]
pub enum InjOracleError {
    #[error(transparent)]
    ChainError(#[from] ChainError),
}
