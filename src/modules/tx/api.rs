// use async_trait::async_trait;
// use cosmrs::proto::cosmos::tx::v1beta1::TxRaw;
// use cosmrs::tx::Body;
// use cosmrs::tx::SignerInfo;
// use serde::Serialize;
// use tendermint_rpc::Client;

// use crate::chain::coin::{Coin, Denom};
// use crate::chain::error::ChainError;
// use crate::chain::msg::Msg;
// use crate::clients::client::ClientAbciQuery;
// use crate::config::cfg::ChainConfig;
// use crate::modules::auth::api::Auth;
// use crate::modules::auth::model::Account;
// use crate::{
//     chain::{fee::Fee, request::TxOptions, Any},
//     signing_key::key::SigningKey,
// };

// use super::error::TxError;
// use super::model::RawTx;

// // TODO: Query endpoints
// // * tx_query_get_tx()
// // * tx_query_get_txs_event()
// // * tx_query_get_block_with_txs()

// impl<T> Tx for T where T: Client {}

// #[async_trait]
// pub trait Tx: Client + Sized {

// }
