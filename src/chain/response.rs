use cosmrs::proto::cosmos::base::abci::v1beta1::TxResponse as CosmosResponse;

use cosmrs::rpc::endpoint::{
    abci_query::AbciQuery, broadcast::tx_async::Response as AsyncTendermintResponse,
    broadcast::tx_sync::Response as SyncTendermintResponse,
};
use cosmrs::tendermint::abci::Code as TendermintCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::error::DeserializeError;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Default)]
pub struct ChainResponse {
    pub code: Code,
    pub data: Option<Vec<u8>>,
    pub log: String,
}

impl ChainResponse {
    pub fn data<'a, T: Deserialize<'a>>(&'a self) -> Result<T, DeserializeError> {
        let r: T = serde_json::from_slice(
            self.data
                .as_ref()
                .ok_or(DeserializeError::EmptyResponse)?
                .as_slice(),
        )?;
        Ok(r)
    }
}

impl From<AbciQuery> for ChainResponse {
    fn from(res: AbciQuery) -> ChainResponse {
        ChainResponse {
            code: res.code.into(),
            data: Some(res.value),
            log: res.log,
        }
    }
}

// impl From<TxResult> for ChainResponse {
//     fn from(res: TxResult) -> ChainResponse {
//         ChainResponse {
//             code: res.code.into(),
//             data: res.data.map(|d| d.into()),
//             log: res.log.to_string(),
//         }
//     }
// }

/// AsyncChainTxResponse is returned from the async `tx_broadcast()` api.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Default)]
pub struct AsyncChainTxResponse {
    pub res: ChainResponse,
    pub tx_hash: String,
}

impl AsRef<ChainResponse> for AsyncChainTxResponse {
    fn as_ref(&self) -> &ChainResponse {
        &self.res
    }
}

impl From<CosmosResponse> for AsyncChainTxResponse {
    fn from(res: CosmosResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()), // TODO
                log: res.raw_log,
            },
            tx_hash: res.txhash,
        }
    }
}

impl From<AsyncTendermintResponse> for AsyncChainTxResponse {
    fn from(res: AsyncTendermintResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()),
                log: res.log.to_string(),
            },
            tx_hash: res.hash.to_string(),
        }
    }
}

impl From<SyncTendermintResponse> for AsyncChainTxResponse {
    fn from(res: SyncTendermintResponse) -> Self {
        Self {
            res: ChainResponse {
                code: res.code.into(),
                data: Some(res.data.into()),
                log: res.log.to_string(),
            },
            tx_hash: res.hash.to_string(),
        }
    }
}

// /// ChainTxResponse is returned from the blocking `tx_broadcast_block()` api.
// /// Since we wait for the tx to be commited in the next block, we get the full tx data.
// #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Default)]
// pub struct ChainTxResponse {
//     pub res: ChainResponse,
//     pub events: Vec<Event>,
//     pub gas_wanted: u64,
//     pub gas_used: u64,
//     pub tx_hash: String,
//     pub height: u64,
// }

// impl AsRef<ChainResponse> for ChainTxResponse {
//     fn as_ref(&self) -> &ChainResponse {
//         &self.res
//     }
// }

// impl From<BlockingTendermintResponse> for ChainTxResponse {
//     fn from(res: BlockingTendermintResponse) -> Self {
//         ChainTxResponse {
//             res: ChainResponse {
//                 code: res.deliver_tx.code.into(),
//                 data: Some(res.deliver_tx.data.to_vec()),
//                 log: res.deliver_tx.log.to_string(),
//             },
//             events: res.deliver_tx.events.into_iter().map(Into::into).collect(),
//             gas_used: res.deliver_tx.gas_used.into(),
//             gas_wanted: res.deliver_tx.gas_wanted.into(),
//             tx_hash: res.hash.to_string(),
//             height: res.height.into(),
//         }
//     }
// }

// impl TryFrom<CosmosResponse> for ChainTxResponse {
//     type Error = ChainError;

//     fn try_from(res: CosmosResponse) -> Result<Self, Self::Error> {
//         Ok(ChainTxResponse {
//             res: ChainResponse {
//                 code: res.code.into(),
//                 data: Some(res.data.into()), // TODO
//                 log: res.raw_log,
//             },
//             events: res
//                 .events
//                 .into_iter()
//                 .map(TryInto::try_into)
//                 .collect::<Result<Vec<_>, _>>()?,
//             gas_wanted: res.gas_wanted as u64,
//             gas_used: res.gas_used as u64,
//             tx_hash: res.txhash,
//             height: res.height as u64,
//         })
//     }
// }

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub enum Code {
    #[default]
    Ok,
    Err(u32),
}

impl Code {
    pub fn is_ok(self) -> bool {
        match self {
            Code::Ok => true,
            Code::Err(_) => false,
        }
    }

    pub fn is_err(self) -> bool {
        !self.is_ok()
    }

    pub fn value(self) -> u32 {
        u32::from(self)
    }
}

impl From<u32> for Code {
    fn from(value: u32) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err),
        }
    }
}

impl From<Code> for u32 {
    fn from(code: Code) -> u32 {
        match code {
            Code::Ok => 0,
            Code::Err(err) => err,
        }
    }
}

impl From<u16> for Code {
    fn from(value: u16) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err.into()),
        }
    }
}

impl From<u8> for Code {
    fn from(value: u8) -> Code {
        match value {
            0 => Code::Ok,
            err => Code::Err(err.into()),
        }
    }
}

impl From<TendermintCode> for Code {
    fn from(value: TendermintCode) -> Code {
        match value {
            TendermintCode::Ok => Code::Ok,
            TendermintCode::Err(err) => Code::Err(err.into()),
        }
    }
}

// #[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
// pub struct Event {
//     pub kind: String,
//     pub attributes: Vec<EventAttribute>,
// }

// impl From<TendermintEvent> for Event {
//     fn from(e: TendermintEvent) -> Self {
//         Self {
//             kind: e.type_str,
//             attributes: e.attributes.into_iter().map(Into::into).collect(),
//         }
//     }
// }

// impl TryFrom<Event> for TendermintEvent {
//     type Error = ChainError;

//     fn try_from(e: Event) -> Result<Self, Self::Error> {
//         Ok(Self {
//             kind: e.kind,
//             attributes: e
//                 .attributes
//                 .into_iter()
//                 .map(TryInto::try_into)
//                 .collect::<Result<Vec<_>, _>>()?,
//         })
//     }
// }

// impl TryFrom<ProtoEvent> for Event {
//     type Error = ChainError;

//     fn try_from(e: ProtoEvent) -> Result<Self, Self::Error> {
//         Ok(Self {
//             kind: e.r#type,
//             attributes: e
//                 .attributes
//                 .into_iter()
//                 .map(TryInto::try_into)
//                 .collect::<Result<Vec<_>, _>>()?,
//         })
//     }
// }

// impl From<Event> for ProtoEvent {
//     fn from(e: Event) -> Self {
//         Self {
//             r#type: e.kind,
//             attributes: e.attributes.into_iter().map(Into::into).collect(),
//         }
//     }
// }

// #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Eq, PartialEq, Hash)]
// pub struct Tag {
//     pub key: String,
//     pub value: String,
// }

// impl From<TendermintProtoTag> for Tag {
//     fn from(tag: TendermintProtoTag) -> Self {
//         Self {
//             key: tag.key.to_string(),
//             value: tag.value.to_string(),
//         }
//     }
// }

// impl TryFrom<Tag> for TendermintProtoTag {
//     type Error = ChainError;

//     fn try_from(tag: Tag) -> Result<Self, Self::Error> {
//         Ok(Self {
//             key: Key::from_str(&tag.key)?,
//             value: Value::from_str(&tag.value)?,
//         })
//     }
// }

// impl From<Tag> for EventAttribute {
//     fn from(tag: Tag) -> Self {
//         Self {
//             key: tag.key.into_bytes().into(),
//             value: tag.value.into_bytes().into(),
//             index: true,
//         }
//     }
// }

// impl TryFrom<EventAttribute> for Tag {
//     type Error = ChainError;

//     fn try_from(attr: EventAttribute) -> Result<Self, Self::Error> {
//         Ok(Self {
//             key: String::from_utf8(attr.key.into()).map_err(|e| ChainError::ProtoDecoding {
//                 message: e.to_string(),
//             })?,
//             value: String::from_utf8(attr.value.into()).map_err(|e| ChainError::ProtoDecoding {
//                 message: e.to_string(),
//             })?,
//         })
//     }
// }
