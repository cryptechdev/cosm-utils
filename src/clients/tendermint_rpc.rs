use async_trait::async_trait;
use cosmrs::{
    rpc::Client,
    tendermint::{
        abci::{response::DeliverTx, Event},
        Hash,
    },
};
use lazy_static::lazy_static;
use log::info;
use std::time::Duration;
use tendermint_rpc::{
    client::CompatMode,
    endpoint::{
        abci_query::AbciQuery,
        broadcast::{tx_async, tx_commit, tx_sync},
        tx,
    },
    query::{EventType, Query},
    HttpClient, Order,
};
use tokio::sync::RwLock;

use crate::chain::error::ChainError;
use crate::chain::tx::RawTx;

use super::client::{
    ClientAbciQuery, ClientTxAsync, ClientTxCommit, ClientTxSync, GetErr, GetEvents, GetValue,
    HashSearch,
};

impl GetEvents for tx_commit::Response {
    fn get_events(&self) -> &[Event] {
        self.deliver_tx.events.as_slice()
    }
}

impl GetEvents for DeliverTx {
    fn get_events(&self) -> &[Event] {
        self.events.as_slice()
    }
}

impl GetErr for tx_commit::Response {
    fn get_err(self) -> Result<Self, ChainError> {
        if self.deliver_tx.code.is_err() || self.check_tx.code.is_err() {
            return Err(ChainError::TxCommit {
                res: format!("{:?}", self),
            });
        }
        Ok(self)
    }
}

impl GetErr for tx_sync::Response {
    fn get_err(self) -> Result<Self, ChainError> {
        if self.code.is_err() {
            return Err(ChainError::TxSync {
                res: format!("{:?}", self),
            });
        }
        Ok(self)
    }
}

impl GetErr for tx_async::Response {
    fn get_err(self) -> Result<Self, ChainError> {
        if self.code.is_err() {
            return Err(ChainError::TxAsync {
                res: format!("{:?}", self),
            });
        }
        Ok(self)
    }
}

impl GetErr for AbciQuery {
    fn get_err(self) -> Result<Self, ChainError> {
        if self.code.is_err() {
            return Err(ChainError::AbciQuery { res: self });
        }
        Ok(self)
    }
}

impl GetValue for AbciQuery {
    fn get_value(&self) -> &[u8] {
        &self.value
    }
}

#[async_trait]
impl<T> ClientAbciQuery for T
where
    T: Client + Sync,
{
    type Response = AbciQuery;

    async fn abci_query<V>(
        &self,
        path: Option<String>,
        data: V,
        height: Option<u32>,
        prove: bool,
    ) -> Result<Self::Response, ChainError>
    where
        V: Into<Vec<u8>> + Send,
    {
        let res = self
            .abci_query(path, data, height.map(Into::into), prove)
            .await?;
        Ok(res.get_err()?)
    }
}

#[async_trait]
impl<T> HashSearch for T
where
    T: ClientAbciQuery + Client + Sync,
{
    async fn hash_search(&self, hash: &Hash) -> Result<tx::Response, ChainError> {
        let query = Query::from(EventType::Tx).and_eq("tx.hash", hash.to_string());
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let start_time = tokio::time::Instant::now();
        interval.tick().await;
        loop {
            interval.tick().await;

            let search_res = self
                .tx_search(query.clone(), false, 1, 255, Order::Ascending)
                .await?;
            if let Some(tx) = search_res.txs.first() {
                return Ok(tx.clone());
            }
            if tokio::time::Instant::now() - start_time > Duration::from_secs(30) {
                return Err(ChainError::TxSearchTimeout { tx_hash: *hash });
            }
        }
    }
}

#[async_trait]
impl<T> ClientTxCommit for T
where
    T: Client + Sync,
{
    type Response = tx_commit::Response;
    async fn broadcast_tx_commit(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError> {
        let res = self.broadcast_tx_commit(raw_tx.to_bytes()?).await?;
        Ok(res.get_err()?)
    }
}

#[async_trait]
impl<T> ClientTxSync for T
where
    T: Client + Sync,
{
    type Response = tx_sync::Response;
    async fn broadcast_tx_sync(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError> {
        let res = self.broadcast_tx_sync(raw_tx.to_bytes()?).await?;
        Ok(res.get_err()?)
    }
}

#[async_trait]
impl<T> ClientTxAsync for T
where
    T: Client + Sync,
{
    type Response = tx_async::Response;
    async fn broadcast_tx_async(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError> {
        let res = self.broadcast_tx_async(raw_tx.to_bytes()?).await?;
        Ok(res.get_err()?)
    }
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait ClientCompat: Client + Sized {
    async fn query_compat_mode(&self) -> Result<CompatMode, ChainError> {
        let version = self.status().await?.node_info.version;
        info!("got tendermint version: {}", version);
        Ok(CompatMode::from_version(version)?)
    }

    async fn get_compat(endpoint_url: &str) -> Result<Self, ChainError>;

    /// WARNING: This function creates a global static to remember the compat mode
    /// for convenience with repeated calls. Once the mode is set, it cannot not be changed
    /// no matter how many times you call it.
    async fn get_persistent_compat(endpoint_url: &str) -> Result<Self, ChainError>;
}

lazy_static! {
    static ref COMPAT_MODE: RwLock<Option<CompatMode>> = RwLock::new(None);
}

#[async_trait]
impl ClientCompat for HttpClient {
    async fn get_compat(endpoint_url: &str) -> Result<Self, ChainError> {
        let mut client = Self::new(endpoint_url)?;
        let compat_mode = client.query_compat_mode().await?;
        client.set_compat_mode(compat_mode);
        Ok(client)
    }

    async fn get_persistent_compat(endpoint_url: &str) -> Result<Self, ChainError> {
        let mut client = Self::new(endpoint_url)?;
        let maybe_compat_mode = *COMPAT_MODE.read().await;
        let compat_mode = match maybe_compat_mode {
            Some(compat_mode) => compat_mode,
            None => {
                let compat_mode = client.query_compat_mode().await?;
                *COMPAT_MODE.write().await = Some(compat_mode);
                compat_mode
            }
        };
        client.set_compat_mode(compat_mode);
        Ok(client)
    }
}
