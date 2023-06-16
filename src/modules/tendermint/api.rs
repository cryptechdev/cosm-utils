use async_trait::async_trait;
use crate::proto::cosmos::base::tendermint::v1beta1::{
    GetLatestBlockRequest, GetLatestBlockResponse, GetLatestValidatorSetRequest,
    GetLatestValidatorSetResponse, GetValidatorSetByHeightRequest, GetValidatorSetByHeightResponse,
};
use tendermint_rpc::Client;

use crate::{chain::request::PaginationRequest, prelude::ClientUtils};

use super::{error::TendermintError, model::ValidatorSetResponse};

impl<T> Tendermint for T where T: Client {}

#[async_trait]
pub trait Tendermint: Client + Sized {
    async fn tendermint_query_latest_block(&self) -> Result<BlockResponse, TendermintError> {
        let req = GetLatestBlockRequest {};

        let res = self
            .query::<_, GetLatestBlockResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestBlock",
            )
            .await?;

        res.try_into()
    }

    async fn tendermint_query_latest_validator_set(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<ValidatorSetResponse, TendermintError> {
        let req = GetLatestValidatorSetRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .query::<_, GetLatestValidatorSetResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetLatestValidatorSet",
            )
            .await?;

        res.try_into()
    }

    async fn tendermint_query_validator_set_at_height(
        &self,
        block_height: u64,
        pagination: Option<PaginationRequest>,
    ) -> Result<ValidatorSetResponse, TendermintError> {
        let req = GetValidatorSetByHeightRequest {
            height: block_height as i64,
            pagination: pagination.map(Into::into),
        };

        let res = self
            .query::<_, GetValidatorSetByHeightResponse>(
                req,
                "/cosmos.base.tendermint.v1beta1.Service/GetValidatorSetByHeight",
            )
            .await?;

        res.try_into()
    }
}
