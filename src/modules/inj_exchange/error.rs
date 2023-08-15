use thiserror::Error;

use crate::{chain::error::ChainError, modules::auth::error::AccountError};

pub use serde_json::Error as SerdeJsonError;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error(transparent)]
    ChainError(#[from] ChainError),

    #[error(transparent)]
    AccountError(#[from] AccountError),
}
