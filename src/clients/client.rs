use crate::chain::fee::GasInfo;
use crate::{chain::error::ChainError, modules::tx::model::RawTx};
use async_trait::async_trait;
use cosmrs::proto::traits::Message;
use cosmrs::tendermint::Hash;

#[cfg(feature = "mocks")]
use mockall::automock;

use tendermint_rpc::endpoint::tx;
use tendermint_rpc::Client;

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait ClientUtils {
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        Self: Sized,
        I: Message + Default + 'static,
        O: Message + Default + 'static;

    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError>;
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait HashSearch: Client {
    async fn hash_search(&self, hash: &Hash) -> Result<tx::Response, ChainError>;
}
