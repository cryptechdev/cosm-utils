use cosmrs::Any;
use injective_std::types::injective::exchange::v1beta1::{MsgCreateSpotLimitOrder, MsgBatchCreateSpotLimitOrders};
use prost::Message;

use crate::chain::msg::IntoAny;

use super::error::ExchangeError;

impl IntoAny for MsgCreateSpotLimitOrder {
    type Err = ExchangeError;

    fn into_any(self) -> Result<cosmrs::Any, Self::Err> {
        Ok(Any{
            type_url: Self::TYPE_URL.to_string(),
            value: self.encode_to_vec(),
        })
    }
}

impl IntoAny for MsgBatchCreateSpotLimitOrders {
    type Err = ExchangeError;

    fn into_any(self) -> Result<cosmrs::Any, Self::Err> {
        Ok(Any{
            type_url: Self::TYPE_URL.to_string(),
            value: self.encode_to_vec(),
        })
    }
}