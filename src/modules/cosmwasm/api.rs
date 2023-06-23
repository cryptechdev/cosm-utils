use async_trait::async_trait;
use serde::Serialize;

use crate::chain::request::TxOptions;
use crate::clients::client::{ClientTxCommit, GetEvents};
use crate::config::cfg::ChainConfig;
use crate::prelude::ClientAbciQuery;
use cosmrs::proto::cosmwasm::wasm::v1::{
    QuerySmartContractStateRequest, QuerySmartContractStateResponse, QueryRawContractStateRequest, QueryRawContractStateResponse,
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

impl<T> CosmwasmTxCommit for T where T: ClientTxCommit + ClientAbciQuery {}

#[async_trait]
pub trait CosmwasmTxCommit: ClientTxCommit + ClientAbciQuery {
    async fn wasm_store_commit(
        &self,
        chain_cfg: &ChainConfig,
        req: StoreCodeRequest,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<StoreCodeResponse<<Self as ClientTxCommit>::Response>, CosmwasmError> {
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
    ) -> Result<StoreCodeBatchResponse<<Self as ClientTxCommit>::Response>, CosmwasmError>
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

        let res = self.broadcast_tx_commit(&tx_raw).await?;

        #[cfg(feature = "generic")]
        let code_ids = res
            .find_event_tags("store_code".to_string(), "code_id".to_string())
            .into_iter()
            .map(|x| x.value.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

        #[cfg(feature = "injective")]
        let code_ids = res
            .find_event_tags("cosmwasm.wasm.v1.EventCodeStored".to_string(), "code_id".to_string())
            .into_iter()
            .map(|x| {x.value.replace('\"',"").parse::<u64>()})
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

        println!("code_ids: {:?}", code_ids);

        Ok(StoreCodeBatchResponse { code_ids, res })
    }

    async fn wasm_instantiate_commit<S>(
        &self,
        chain_cfg: &ChainConfig,
        req: InstantiateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<InstantiateResponse<<Self as ClientTxCommit>::Response>, CosmwasmError>
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
    ) -> Result<InstantiateBatchResponse<<Self as ClientTxCommit>::Response>, CosmwasmError>
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

        let res = self.broadcast_tx_commit(&tx_raw).await?;

        #[cfg(feature = "generic")]
        let addrs = res
            .find_event_tags("instantiate".to_string(), "_contract_address".to_string())
            .into_iter()
            .map(|x| x.value.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

        #[cfg(feature = "injective")]
        let addrs = res
            .find_event_tags("cosmwasm.wasm.v1.EventContractInstantiated".to_string(), "contract_address".to_string())
            .into_iter()
            .map(|x| {x.value.replace('\"',"").parse()})
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

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
    ) -> Result<<Self as ClientTxCommit>::Response, CosmwasmError>
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
    ) -> Result<<Self as ClientTxCommit>::Response, CosmwasmError>
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

        let res = self.broadcast_tx_commit(&tx_raw).await?;

        Ok(res)
    }

    async fn wasm_migrate_commit<S>(
        &self,
        chain_cfg: &ChainConfig,
        req: MigrateRequest<S>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<<Self as ClientTxCommit>::Response, CosmwasmError>
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
    ) -> Result<<Self as ClientTxCommit>::Response, CosmwasmError>
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

        let res = self.broadcast_tx_commit(&tx_raw).await?;

        Ok(res)
    }
}

impl<T> CosmwasmQuery for T where T: ClientAbciQuery {}

#[async_trait]
pub trait CosmwasmQuery: ClientAbciQuery {
    async fn wasm_query<S: Serialize + Sync>(
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

    async fn wasm_query_raw<S: Serialize + Sync>(
        &self,
        address: Address,
        payload: Vec<u8>,
    ) -> Result<QueryRawContractStateResponse, CosmwasmError> {

        let req = QueryRawContractStateRequest {
            address: address.into(),
            query_data: payload,
        };
        
        let res = self
            .query::<_, QueryRawContractStateResponse>(
                req,
                "/cosmwasm.wasm.v1.Query/RawContractState",
            )
            .await?;

        Ok(res)
    }
}
