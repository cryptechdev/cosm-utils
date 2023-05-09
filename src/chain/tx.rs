use crate::chain::error::ChainError;
use cosmrs::proto::traits::MessageExt;
use cosmrs::proto::{cosmos::tx::v1beta1::TxRaw, traits::Message};
use cosmrs::tx::Raw;

#[derive(Clone, Debug, PartialEq)]
pub struct RawTx(TxRaw);

impl RawTx {
    /// Deserialize raw transaction from serialized protobuf.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ChainError> {
        Ok(RawTx(
            Message::decode(bytes).map_err(ChainError::prost_proto_decoding)?,
        ))
    }

    /// Serialize raw transaction as a byte vector.
    pub fn to_bytes(&self) -> Result<Vec<u8>, ChainError> {
        self.0.to_bytes().map_err(ChainError::prost_proto_encoding)
    }
}

impl From<RawTx> for TxRaw {
    fn from(tx: RawTx) -> Self {
        tx.0
    }
}

impl From<TxRaw> for RawTx {
    fn from(tx: TxRaw) -> Self {
        RawTx(tx)
    }
}

impl From<RawTx> for Raw {
    fn from(tx: RawTx) -> Self {
        tx.0.into()
    }
}

impl From<Raw> for RawTx {
    fn from(tx: Raw) -> Self {
        RawTx(tx.into())
    }
}
