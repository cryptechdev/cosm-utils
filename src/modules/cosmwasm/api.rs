use async_trait::async_trait;
use serde::Serialize;
use tendermint_rpc::endpoint::broadcast::tx_commit;
use tendermint_rpc::Client;

use crate::chain::request::TxOptions;
use crate::clients::response::FindEventTags;
use crate::config::cfg::ChainConfig;
use crate::modules::tx::api::Tx;
use crate::prelude::ClientUtils;
use cosmrs::proto::cosmwasm::wasm::v1::{
    QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};

use crate::modules::auth::model::Address;
use crate::signing_key::key::SigningKey;

use super::model::{
    ExecRequest, InstantiateBatchResponse, InstantiateRequest, MigrateRequest,
    StoreCodeBatchResponse, StoreCodeRequest,
};
use super::{
    error::CosmwasmError,
    model::{InstantiateResponse, StoreCodeResponse},
};

impl<T> Cosmwasm for T where T: Client {}

#[async_trait]
pub trait Cosmwasm: Client + Sized {
    async fn wasm_store_commit(
        &self,
        chain_cfg: &ChainConfig,
        req: StoreCodeRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<StoreCodeResponse<tx_commit::Response>, CosmwasmError> {
        let mut res = self
            .wasm_store_batch_commit(chain_cfg, vec![req], key, tx_options)
            .await?;

        Ok(StoreCodeResponse {
            code_id: res.code_ids.remove(0),
            res: res.res,
        })
    }

    async fn wasm_store_batch_commit<I>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<StoreCodeBatchResponse<tx_commit::Response>, CosmwasmError>
    where
        I: IntoIterator<Item = StoreCodeRequest> + Send,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let msgs = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = self.tx_sign(chain_cfg, msgs, key, tx_options).await?;

        let res = self.broadcast_tx_commit(tx_raw.to_bytes()?).await?;

        let code_ids = res
            .deliver_tx
            .find_event_tags("store_code".to_string(), "code_id".to_string())
            .into_iter()
            .map(|x| x.value.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

        Ok(StoreCodeBatchResponse { code_ids, res })
    }

    async fn wasm_instantiate_commit<S>(
        &self,
        chain_cfg: &ChainConfig,
        req: InstantiateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<InstantiateResponse<tx_commit::Response>, CosmwasmError>
    where
        S: Serialize + Send,
    {
        let mut res = self
            .wasm_instantiate_batch_commit(chain_cfg, vec![req], key, tx_options)
            .await?;

        Ok(InstantiateResponse {
            address: res.addresses.remove(0),
            res: res.res,
        })
    }

    async fn wasm_instantiate_batch_commit<S, I>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<InstantiateBatchResponse<tx_commit::Response>, CosmwasmError>
    where
        S: Serialize + Send,
        I: IntoIterator<Item = InstantiateRequest<S>> + Send,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let msgs = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = self.tx_sign(chain_cfg, msgs, key, tx_options).await?;

        let res = self.broadcast_tx_commit(tx_raw.to_bytes()?).await?;

        let events = res
            .deliver_tx
            .find_event_tags("instantiate".to_string(), "_contract_address".to_string());

        if events.is_empty() {
            return Err(CosmwasmError::MissingEvent);
        }

        let addrs = events
            .into_iter()
            .map(|e| e.value.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(InstantiateBatchResponse {
            addresses: addrs,
            res,
        })
    }

    async fn wasm_execute_commit<S>(
        &self,
        chain_cfg: &ChainConfig,
        req: ExecRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<tx_commit::Response, CosmwasmError>
    where
        S: Serialize + Send,
    {
        self.wasm_execute_batch_commit(chain_cfg, vec![req], key, tx_options)
            .await
    }

    async fn wasm_execute_batch_commit<S, I>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<tx_commit::Response, CosmwasmError>
    where
        S: Serialize + Send,
        I: IntoIterator<Item = ExecRequest<S>> + Send,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let msgs = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = self.tx_sign(chain_cfg, msgs, key, tx_options).await?;

        let res = self.broadcast_tx_commit(tx_raw.to_bytes()?).await?;

        Ok(res)
    }

    async fn wasm_query_commit<S: Serialize + Sync>(
        &self,
        address: Address,
        msg: &S,
    ) -> Result<QuerySmartContractStateResponse, CosmwasmError> {
        let payload = serde_json::to_vec(msg).map_err(CosmwasmError::json)?;

        let req = QuerySmartContractStateRequest {
            address: address.into(),
            query_data: payload,
        };

        let res = self
            .query::<_, QuerySmartContractStateResponse>(
                req,
                "/cosmwasm.wasm.v1.Query/SmartContractState",
            )
            .await?;

        Ok(res)
    }

    async fn wasm_migrate_commit<S>(
        &self,
        chain_cfg: &ChainConfig,
        req: MigrateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<tx_commit::Response, CosmwasmError>
    where
        S: Serialize + Send,
    {
        self.wasm_migrate_batch_commit(chain_cfg, vec![req], key, tx_options)
            .await
    }

    async fn wasm_migrate_batch_commit<S, I>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: I,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<tx_commit::Response, CosmwasmError>
    where
        S: Serialize + Send,
        I: IntoIterator<Item = MigrateRequest<S>> + Send,
    {
        let sender_addr = key
            .to_addr(&chain_cfg.prefix, &chain_cfg.derivation_path)
            .await?;

        let msgs = reqs
            .into_iter()
            .map(|r| r.to_proto(sender_addr.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_raw = self.tx_sign(chain_cfg, msgs, key, tx_options).await?;

        let res = self.broadcast_tx_commit(tx_raw.to_bytes()?).await?;

        Ok(res)
    }
}
