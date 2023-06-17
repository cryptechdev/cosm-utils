use async_trait::async_trait;
use injective_std::types::injective::oracle::v1beta1::{QueryPythPriceRequest, QueryPythPriceResponse};
use serde::Serialize;
use crate::prelude::ClientAbciQuery;
use super::error::InjOracleError;

impl<T> InjOracleQuery for T where T: ClientAbciQuery {}

#[async_trait]
pub trait InjOracleQuery: ClientAbciQuery {
    async fn query_pyth_price<S: Serialize + Sync>(
        &self,
        price_id: String,
    ) -> Result<QueryPythPriceResponse, InjOracleError> {

        let req = QueryPythPriceRequest {
            price_id
        };

        let res = self
            .query::<_, QueryPythPriceResponse>(
                req,
                "/injective.oracle.v1beta1.Query/PythPrice",
            )
            .await?;

        Ok(res)
    }
}
