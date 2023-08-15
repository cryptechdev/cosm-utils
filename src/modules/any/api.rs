use super::error::AnyError;
use crate::{prelude::{ClientAbciQuery, ClientTxAsync, ClientTxCommit}, config::cfg::ChainConfig, signing_key::key::SigningKey, chain::{request::{TxOptions, ExecRequest, TryFromEvents}, error::ChainError}, clients::client::GetEvents};
use async_trait::async_trait;
use tendermint_rpc::Client;

impl<T> TxCommit for T where T: ClientTxCommit + ClientAbciQuery {}

pub struct Response<E, C> {
    exec_res: E,
    client_res: C
}

pub struct Responses<E, C> {
    exec_res: Vec<E>,
    client_res: C
}

impl<E: Clone, C> From<Responses<E, C>> for Response<E, C> {
    fn from(value: Responses<E, C>) -> Self {
        Response { exec_res: value.exec_res[0].clone(), client_res: value.client_res }
    }
}

#[async_trait]
pub trait TxCommit: ClientTxCommit + ClientAbciQuery  {
    /// Send `amount` of funds from source (`from`) Address to destination (`to`) Address
    async fn send_commit<T>(
        &self,
        chain_cfg: &ChainConfig,
        req: T,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<Response<<T as ExecRequest>::Response,<Self as ClientTxCommit>::Response>, AnyError>
    where
        T: cosmrs::tx::Msg + Send + Sync + ExecRequest,
        <T as ExecRequest>::Response: TryFromEvents,
    {
        Ok(self.send_commit_batch(chain_cfg, vec![req], key, tx_options)
            .await?.into())
    }

    async fn send_commit_batch<T>(
        &self,
        chain_cfg: &ChainConfig,
        reqs: Vec<T>,
        key: &SigningKey,
        tx_options: &TxOptions,
    ) -> Result<Responses<<T as ExecRequest>::Response,<Self as ClientTxCommit>::Response>, AnyError>
    where
        T: cosmrs::tx::Msg + Send + Sync + ExecRequest,
        <T as ExecRequest>::Response: TryFromEvents,
    {
        let tx_raw = self.tx_sign(chain_cfg, reqs.clone(), key, tx_options).await?;

        let client_res = self.broadcast_tx_commit(&tx_raw).await?;
        let mut events = client_res.get_events().iter();
        let exec_res = reqs.into_iter().map(|_| {
            <T as ExecRequest>::Response::try_from_events(&mut events) 
        }).collect::<Result<Vec<<T as ExecRequest>::Response>, ChainError>>()?;
        Ok(Responses { exec_res, client_res })
    }
}

#[cfg(test)]
mod tests {
    use cosmrs::cosmwasm::MsgExecuteContract;
    use tendermint_rpc::{HttpClient, HttpClientUrl};

    #[tokio::test]
    async fn test() {
        let client = HttpClient::new("").unwrap();
        // client.
        // let x = client.send_commit(
        //     todo!(),
        //     MsgExecuteContract{ sender: todo!(), contract: todo!(), msg: todo!(), funds: todo!() },
        //     todo!(),
        //     todo!()
        // ).await;
    }
}