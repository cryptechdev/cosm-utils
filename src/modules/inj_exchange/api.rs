use async_trait::async_trait;
use injective_std::types::injective::exchange::v1beta1::{MsgCreateSpotLimitOrder, MsgBatchCreateSpotLimitOrders, MsgBatchUpdateOrders, QuerySpotMarketsRequest, QuerySpotMarketsResponse, QuerySpotMarketResponse, QuerySpotMarketRequest};

use crate::{
    chain::request::TxOptions,
    clients::client::{ClientAbciQuery, ClientTxCommit, QueryResponse},
    config::cfg::ChainConfig,
    signing_key::key::SigningKey,
};

use super::error::ExchangeError;

impl<T> ExchangeQuery for T where T: ClientAbciQuery {}

#[async_trait]
pub trait ExchangeQuery: ClientAbciQuery + Sized {
    /// Query the amount of `denom` currently held by an `address`
    async fn exchange_query_spot_markets(
        &self,
        req: QuerySpotMarketsRequest,
        height: Option<u32>,
    ) -> Result<QueryResponse<<Self as ClientAbciQuery>::Response, QuerySpotMarketsResponse>, ExchangeError> {

        Ok(self.query::<_, QuerySpotMarketsResponse>(req, "/injective.exchange.v1beta1.Query/SpotMarkets", height).await?)
    }

    async fn exchange_query_spot_market(
        &self,
        req: QuerySpotMarketRequest,
        height: Option<u32>,
    ) -> Result<QueryResponse<<Self as ClientAbciQuery>::Response, QuerySpotMarketResponse>, ExchangeError> {

        Ok(self.query::<_, QuerySpotMarketResponse>(req, "/injective.exchange.v1beta1.Query/SpotMarket", height).await?)
    }
}

impl<T> ExchangeTxCommit for T where T: ClientTxCommit + ClientAbciQuery {}

#[async_trait]
pub trait ExchangeTxCommit: ClientTxCommit + ClientAbciQuery {
    async fn exchange_batch_update_orders_commit(
        &self,
        chain_cfg: &ChainConfig,
        req: MsgBatchUpdateOrders,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxCommit>::Response, ExchangeError>
    {
        let tx_raw = self.tx_sign(chain_cfg, vec![req], key, tx_options).await?;

        Ok(self.broadcast_tx_commit(&tx_raw).await?)
    }

    async fn exchange_create_spot_limit_order_commit(
        &self,
        chain_cfg: &ChainConfig,
        req: MsgCreateSpotLimitOrder,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxCommit>::Response, ExchangeError>
    {
        let tx_raw = self.tx_sign(chain_cfg, vec![req], key, tx_options).await?;

        Ok(self.broadcast_tx_commit(&tx_raw).await?)
    }

    async fn exchange_create_spot_limit_order_batch_commit(
        &self,
        chain_cfg: &ChainConfig,
        req: MsgBatchCreateSpotLimitOrders,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxCommit>::Response, ExchangeError>
    {
        let tx_raw = self.tx_sign(chain_cfg, vec![req], key, tx_options).await?;

        Ok(self.broadcast_tx_commit(&tx_raw).await?)
    }
}