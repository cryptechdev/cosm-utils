use crate::chain::coin::{Coin, Denom};
use crate::chain::fee::{Fee, GasInfo};
use crate::chain::msg::Msg;
use crate::chain::request::TxOptions;
use crate::config::cfg::ChainConfig;
use crate::modules::auth::error::AccountError;
use crate::modules::auth::model::{Account, AccountResponse, Address};
use crate::modules::tx::error::TxError;
use crate::signing_key::key::SigningKey;
use crate::{chain::error::ChainError, modules::tx::model::RawTx};
use async_trait::async_trait;
use cosmrs::proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse,
};
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse, TxRaw};
use cosmrs::proto::traits::Message;
use cosmrs::tendermint::Hash;
use cosmrs::Any;

use cosmrs::tendermint::abci::{Event, EventAttribute};
use cosmrs::tx::{Body, SignerInfo};
#[cfg(feature = "mockall")]
use mockall::automock;

use serde::Serialize;
use tendermint_rpc::endpoint::tx;

fn encode_msg<T: Message>(msg: T) -> Result<Vec<u8>, ChainError> {
    let mut data = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut data)
        .map_err(ChainError::prost_proto_encoding)?;
    Ok(data)
}

pub trait GetErr: Sized {
    fn get_err(self) -> Result<Self, ChainError>;
}

pub trait GetValue {
    fn get_value(&self) -> &[u8];
}

pub trait GetEvents {
    fn get_events(&self) -> &[Event];

    fn find_event_tags(&self, event_type: String, key_name: String) -> Vec<&EventAttribute> {
        let mut events = vec![];
        for event in self.get_events() {
            if event.kind == event_type {
                for attr in &event.attributes {
                    if attr.key == key_name {
                        events.push(attr);
                    }
                }
            }
        }
        events
    }
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait HashSearch: ClientAbciQuery {
    async fn hash_search(&self, hash: &Hash) -> Result<tx::Response, ChainError>;
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait ClientTxCommit {
    type Response: GetErr + GetEvents;
    async fn broadcast_tx_commit(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError>;
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait ClientTxSync {
    type Response: GetErr;
    async fn broadcast_tx_sync(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError>;
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait ClientTxAsync {
    type Response: GetErr;
    async fn broadcast_tx_async(&self, raw_tx: &RawTx) -> Result<Self::Response, ChainError>;
}

#[cfg_attr(feature = "mockall", automock)]
#[async_trait]
pub trait ClientAbciQuery: Sized {
    type Response: GetErr + GetValue;
    async fn abci_query<V>(
        &self,
        path: Option<String>,
        data: V,
        height: Option<u32>,
        prove: bool,
    ) -> Result<Self::Response, ChainError>
    where
        V: Into<Vec<u8>> + Send;

    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        Self: Sized,
        I: Message + Default + 'static,
        O: Message + Default + 'static,
    {
        let bytes = encode_msg(msg)?;

        let res = self
            .abci_query(Some(path.to_string()), bytes, None, false)
            .await?;

        let proto_res =
            O::decode(res.get_err()?.get_value()).map_err(ChainError::prost_proto_decoding)?;

        Ok(proto_res)
    }

    async fn auth_query_account(&self, address: Address) -> Result<AccountResponse, AccountError> {
        let req = QueryAccountRequest {
            address: address.into(),
        };

        let res = self
            .query::<_, QueryAccountResponse>(req, "/cosmos.auth.v1beta1.Query/Account")
            .await?;

        let account = res.account.ok_or(AccountError::Address {
            message: "Invalid account address".to_string(),
        })?;

        let base_account = BaseAccount::decode(account.value.as_slice())
            .map_err(ChainError::prost_proto_decoding)?;

        Ok(AccountResponse {
            account: base_account.try_into()?,
        })
    }

    #[allow(deprecated)]
    async fn query_simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError> {
        let req = SimulateRequest {
            tx: None,
            tx_bytes: tx.to_bytes()?,
        };

        let bytes = encode_msg(req)?;

        let res = self
            .abci_query(
                Some("/cosmos.tx.v1beta1.Service/Simulate".to_string()),
                bytes,
                None,
                false,
            )
            .await?;

        let sim_res = SimulateResponse::decode(res.get_err()?.get_value())
            .map_err(ChainError::prost_proto_decoding)?;

        let gas_info = sim_res.gas_info.ok_or(ChainError::Simulation {
            result: sim_res.result.unwrap(),
        })?;

        Ok(gas_info.into())
    }

    // Sends tx with an empty public_key / signature, like they do in the cosmos-sdk:
    // https://github.com/cosmos/cosmos-sdk/blob/main/client/tx/tx.go#L133
    async fn tx_simulate<I>(
        &self,
        denom: &str,
        gas_price: f64,
        gas_adjustment: f64,
        msgs: I,
        account: &Account,
    ) -> Result<Fee, TxError>
    where
        I: IntoIterator<Item = Any> + Send,
    {
        let tx = Body::new(msgs, "cosm-client memo", 0u16);

        let denom: Denom = denom.parse()?;

        let fee = Fee::new(
            Coin {
                denom: denom.clone(),
                amount: 0u128,
            },
            0u64,
            None,
            None,
        );

        let auth_info =
            SignerInfo::single_direct(None, account.sequence).auth_info(fee.try_into()?);

        let tx_raw = TxRaw {
            body_bytes: tx.into_bytes().map_err(ChainError::proto_encoding)?,
            auth_info_bytes: auth_info.into_bytes().map_err(ChainError::proto_encoding)?,
            signatures: vec![vec![]],
        };

        let gas_info = self.query_simulate_tx(&tx_raw.into()).await?;

        // TODO: clean up this gas conversion code to be clearer
        let gas_limit = (gas_info.gas_used.value() as f64 * gas_adjustment).ceil();
        let amount = Coin {
            denom,
            amount: ((gas_limit * gas_price).ceil() as u64).into(),
        };

        let fee = Fee::new(amount, gas_limit as u64, None, None);

        Ok(fee)
    }

    async fn tx_sign<T>(
        &self,
        chain_cfg: &ChainConfig,
        msgs: Vec<T>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<RawTx, TxError>
    where
        T: Msg + Serialize + Send + Sync,
        <T as Msg>::Err: Send + Sync,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let timeout_height = tx_options.timeout_height.unwrap_or_default();

        let account = if let Some(ref account) = tx_options.account {
            account.clone()
        } else {
            self.auth_query_account(sender_addr).await?.account
        };

        let fee = if let Some(fee) = &tx_options.fee {
            fee.clone()
        } else {
            self.tx_simulate(
                &chain_cfg.denom,
                chain_cfg.gas_price,
                chain_cfg.gas_adjustment,
                msgs.iter()
                    .map(|m| m.to_any())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ChainError::ProtoEncoding {
                        message: e.to_string(),
                    })?,
                &account,
            )
            .await?
        };

        let raw = key
            .sign(
                msgs,
                timeout_height,
                &tx_options.memo,
                account,
                fee,
                &chain_cfg.chain_id,
                &chain_cfg.derivation_path,
            )
            .await?;
        Ok(raw)
    }
}
