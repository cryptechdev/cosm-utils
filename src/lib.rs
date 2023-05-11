#![allow(clippy::result_large_err)]
pub mod modules;

pub mod clients;

pub mod config;

pub mod signing_key;

pub mod chain;

pub use tendermint_rpc;

pub mod prelude {
    pub use crate::clients::client::{
        ClientAbciQuery, ClientTxAsync, ClientTxCommit, ClientTxSync,
    };
    pub use crate::clients::tendermint_rpc::ClientCompat;
    pub use crate::modules::{
        auth::api::Auth,
        bank::api::{BankTxAsync, BankTxCommit, BankTxSync},
        cosmwasm::api::{CosmwasmQuery, CosmwasmTxCommit},
    };
}
