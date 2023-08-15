#![allow(clippy::result_large_err)]

#[cfg(all(feature = "generic", feature = "injective"))]
compile_error!("Feature `generic` and `injective` are mutually exclusive and cannot be enabled together");

#[cfg(all(not(feature = "generic"), not(feature = "injective")))]
compile_error!("one of features `generic` or `injective` must be enabled");

pub mod proto {
    #[cfg(feature = "generic")]
    pub use cosmrs::proto::*;

    #[cfg(feature = "injective")]
    pub use injective_std::types::*;
}

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
        // bank::api::{BankTxAsync, BankTxCommit, BankTxSync},
        // cosmwasm::api::{CosmwasmQuery, CosmwasmTxCommit, CosmwasmTxAsync},
    };
    #[cfg(feature = "injective")]
    pub use crate::modules::inj_oracle::api::InjOracleQuery;
}
