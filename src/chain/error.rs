use cosmrs::proto::cosmos::base::abci::v1beta1::Result;
use cosmrs::proto::prost::{DecodeError, EncodeError};
use cosmrs::tendermint::Hash;
use cosmrs::ErrorReport;
use tendermint_rpc::endpoint::abci_query::AbciQuery;
use thiserror::Error;

#[cfg(feature = "keyring")]
pub use keyring::Error as KeyringError;

pub use cosmrs::rpc::Error as TendermintRPCError;
pub use cosmrs::tendermint::Error as TendermintError;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("invalid denomination: {name:?}")]
    Denom { name: String },

    #[error("invalid chainId: {chain_id:?}")]
    ChainId { chain_id: String },

    #[error("api endpoint is not configured {api_type:?}")]
    MissingApiEndpoint { api_type: String },

    #[error("invalid mnemonic")]
    Mnemonic,

    #[error("invalid derivation path")]
    DerviationPath,

    #[error("cryptographic error: {message:?}")]
    Crypto { message: String },

    #[error("invalid query path url: {url:?}")]
    QueryPath { url: String },

    #[error("proto encoding error: {message:?}")]
    ProtoEncoding { message: String },

    #[error("proto decoding error: {message:?}")]
    ProtoDecoding { message: String },

    #[error("error during simulation: {result:?}")]
    Simulation { result: Result },

    #[error("tx_search timed out looking for: {tx_hash:?}")]
    TxSearchTimeout { tx_hash: Hash },

    #[cfg(feature = "keyring")]
    #[error(transparent)]
    Keyring(#[from] KeyringError),

    #[error("CosmosSDK error: {res:?}")]
    CosmosSdk { res: AbciQuery },

    #[error("Tendermint error")]
    Tendermint(#[from] TendermintError),

    #[error(transparent)]
    RPC(#[from] TendermintRPCError),
}

impl ChainError {
    pub(crate) fn crypto(e: ErrorReport) -> ChainError {
        ChainError::Crypto {
            message: e.to_string(),
        }
    }

    pub(crate) fn proto_encoding(e: ErrorReport) -> ChainError {
        ChainError::ProtoEncoding {
            message: e.to_string(),
        }
    }

    pub(crate) fn prost_proto_encoding(e: EncodeError) -> ChainError {
        ChainError::ProtoEncoding {
            message: e.to_string(),
        }
    }

    pub(crate) fn prost_proto_decoding(e: DecodeError) -> ChainError {
        ChainError::ProtoDecoding {
            message: e.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("Raw chain response is empty")]
    EmptyResponse,

    #[error(transparent)]
    Serde(#[from] serde_json::error::Error),
}
