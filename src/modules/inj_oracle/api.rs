use super::error::InjOracleError;
use crate::prelude::ClientAbciQuery;
use async_trait::async_trait;
use injective_std::types::injective::oracle::v1beta1::{
    QueryPythPriceRequest, QueryPythPriceResponse, QueryPythPriceStatesRequest,
    QueryPythPriceStatesResponse,
};
use serde::Serialize;

impl<T> InjOracleQuery for T where T: ClientAbciQuery {}

#[async_trait]
pub trait InjOracleQuery: ClientAbciQuery {
    async fn query_pyth_price<S: Serialize + Sync>(
        &self,
        price_id: String,
    ) -> Result<QueryPythPriceResponse, InjOracleError> {
        let req = QueryPythPriceRequest { price_id };

        let res = self
            .query::<_, QueryPythPriceResponse>(req, "/injective.oracle.v1beta1.Query/PythPrice")
            .await?;

        Ok(res)
    }

    async fn query_pyth_price_states(
        &self,
    ) -> Result<QueryPythPriceStatesResponse, InjOracleError> {
        let req = QueryPythPriceStatesRequest {};

        let res = self
            .query::<_, QueryPythPriceStatesResponse>(
                req,
                "/injective.oracle.v1beta1.Query/PythPriceStates",
            )
            .await?;

        Ok(res)
    }
}
