use async_trait::async_trait;
use injective_std::types::injective::exchange::v1beta1::{MsgCreateSpotLimitOrder, MsgBatchCreateSpotLimitOrders};

use crate::{
    chain::request::TxOptions,
    clients::client::{ClientAbciQuery, ClientTxCommit},
    config::cfg::ChainConfig,
    signing_key::key::SigningKey,
};

use super::error::ExchangeError;

// impl<T> ExchangeQuery for T where T: ClientAbciQuery {}

// #[async_trait]
// pub trait ExchangeQuery: ClientAbciQuery + Sized {
//     /// Query the amount of `denom` currently held by an `address`
//     async fn exchange_query_balance(
//         &self,
//         address: Address,
//         denom: Denom,
//         height: Option<u32>,
//     ) -> Result<QueryResponse<<Self as ClientAbciQuery>::Response, BalanceResponse>, ExchangeError> {
//         let req = QueryBalanceRequest {
//             address: address.into(),
//             denom: denom.into(),
//         };

//         let res = self
//             .query::<_, QueryBalanceResponse>(req, "/cosmos.exchange.v1beta1.Query/Balance", height)
//             .await?;

//         // NOTE: we are unwrapping here, because unknown denoms still have a 0 balance returned here
//         // let balance = res.value.balance.unwrap().try_into()?;
//         res.try_map(|x| {
//             let balance: Coin = x.balance.unwrap().try_into()?;
//             Ok(BalanceResponse { balance })
//         })
//         // let balance: Coin = res.value.balance.clone().unwrap().try_into()?;
//         // Ok(res.map(|_| BalanceResponse { balance }))
//     }
// }

impl<T> ExchangeTxCommit for T where T: ClientTxCommit + ClientAbciQuery {}

#[async_trait]
pub trait ExchangeTxCommit: ClientTxCommit + ClientAbciQuery {

    async fn exchange_create_spot_limit_order_commit<I>(
        &self,
        chain_cfg: &ChainConfig,
        req: MsgCreateSpotLimitOrder,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxCommit>::Response, ExchangeError>
    where
        I: IntoIterator<Item = MsgCreateSpotLimitOrder> + Send,
    {
        let tx_raw = self.tx_sign(chain_cfg, vec![req], key, tx_options).await?;

        Ok(self.broadcast_tx_commit(&tx_raw).await?)
    }

    async fn exchange_create_spot_limit_order_batch_commit<I>(
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