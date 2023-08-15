use super::error::InjOracleError;
use crate::{prelude::{ClientAbciQuery, ClientTxAsync}, clients::client::QueryResponse, config::cfg::ChainConfig, modules::bank::model::SendRequest, signing_key::key::SigningKey, chain::request::TxOptions};
use async_trait::async_trait;
use injective_std::types::injective::oracle::v1beta1::{
    QueryPythPriceRequest, QueryPythPriceResponse, QueryPythPriceStatesRequest,
    QueryPythPriceStatesResponse,
};
use serde::Serialize;

impl<T> BankTxAsync for T where T: ClientTxAsync + ClientAbciQuery {}

#[async_trait]
pub trait BankTxAsync: ClientTxAsync + ClientAbciQuery {
    /// Send `amount` of funds from source (`from`) Address to destination (`to`) Address
    async fn bank_send_async(
        &self,
        chain_cfg: &ChainConfig,
        req: SendRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxAsync>::Response, OrderBookError> {
        self.bank_send_batch_async(chain_cfg, vec![req], key, tx_options)
            .await
    }

    async fn bank_send_batch_async<I>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxAsync>::Response, OrderBookError>
    where
        I: IntoIterator<Item = SendRequest> + Send,
    {
        let msgs = reqs
            .into_iter()
            .map(Into::into)
            .collect::<Vec<SendRequestProto>>();

        let tx_raw = self.tx_sign(chain_cfg, msgs, key, tx_options).await?;

        Ok(self.broadcast_tx_async(&tx_raw).await?)
    }
}