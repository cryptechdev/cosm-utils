use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmrs::proto::traits::Message;
use cosmrs::rpc::Client;

use crate::chain::error::ChainError;
use crate::chain::fee::GasInfo;
use crate::modules::tx::model::RawTx;

use super::client::ClientUtils;

fn encode_msg<T: Message>(msg: T) -> Result<Vec<u8>, ChainError> {
    let mut data = Vec::with_capacity(msg.encoded_len());
    msg.encode(&mut data)
        .map_err(ChainError::prost_proto_encoding)?;
    Ok(data)
}

#[async_trait]
impl<T> ClientUtils for T
where
    T: Client + Sync,
{
    async fn query<I, O>(&self, msg: I, path: &str) -> Result<O, ChainError>
    where
        I: Message + Default + 'static,
        O: Message + Default + 'static,
    {
        let bytes = encode_msg(msg)?;

        let res = self
            .abci_query(Some(path.to_string()), bytes, None, false)
            .await?;

        let proto_res =
            O::decode(res.value.as_slice()).map_err(ChainError::prost_proto_decoding)?;

        Ok(proto_res)
    }

    #[allow(deprecated)]
    async fn simulate_tx(&self, tx: &RawTx) -> Result<GasInfo, ChainError> {
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

        let gas_info = SimulateResponse::decode(res.value.as_slice())
            .map_err(ChainError::prost_proto_decoding)?
            .gas_info
            .ok_or(ChainError::Simulation)?;

        Ok(gas_info.into())
    }
}
