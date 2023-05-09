use cosmrs::{
    crypto::PublicKey,
    proto::{
        cosmos::base::tendermint::v1beta1::{
            Block, GetLatestBlockResponse, GetLatestValidatorSetResponse,
            GetValidatorSetByHeightResponse, Validator as ProtoValidator,
        },
        tendermint::types::{Block, BlockId, Commit, Data, EvidenceList, Header},
    },
};

use serde::{Deserialize, Serialize};

use crate::{
    chain::{error::ChainError, request::PaginationResponse},
    modules::auth::model::Address,
};

use super::error::TendermintError;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SdkBlock {
    pub header: Option<Header>,
    pub data: Option<Data>,
    pub evidence: Option<EvidenceList>,
    pub last_commit: Option<Commit>,
}

impl From<Block> for SdkBlock {
    fn from(value: Block) -> Self {
        todo!()
    }
}

impl From<SdkBlock> for Block {
    fn from(value: SdkBlock) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockResponse {
    pub id: BlockId,
    pub block: Option<Block>,
    pub sdk_block: Option<SdkBlock>,
}

impl From<GetLatestBlockResponse> for BlockResponse {
    fn from(res: GetLatestBlockResponse) -> Self {
        Ok(Self {
            id: res.block_id,
            sdk_block: res.sdk_block,
        })
    }
}

impl From<BlockResponse> for GetLatestBlockResponse {
    fn from(res: BlockResponse) -> Self {
        Self {
            block_id: Some(res.id),
            block: res.block,
            sdk_block: res.sdk_block,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Validator {
    pub address: Address,
    pub pubkey: Option<PublicKey>,
    pub voting_power: i64,
    pub proposer_priority: i64,
}

impl From<Validator> for ProtoValidator {
    fn from(val: Validator) -> Self {
        Self {
            address: val.address.into(),
            pub_key: val.pubkey.map(Into::into),
            voting_power: val.voting_power,
            proposer_priority: val.proposer_priority,
        }
    }
}

impl TryFrom<ProtoValidator> for Validator {
    type Error = TendermintError;

    fn try_from(val: ProtoValidator) -> Result<Self, Self::Error> {
        Ok(Self {
            address: val.address.parse()?,
            pubkey: val
                .pub_key
                .map(TryFrom::try_from)
                .transpose()
                .map_err(ChainError::crypto)?,
            voting_power: val.voting_power,
            proposer_priority: val.proposer_priority,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ValidatorSetResponse {
    pub block_height: u64,
    pub validators: Vec<Validator>,
    pub next: Option<PaginationResponse>,
}

impl TryFrom<GetLatestValidatorSetResponse> for ValidatorSetResponse {
    type Error = TendermintError;

    fn try_from(res: GetLatestValidatorSetResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            block_height: res.block_height as u64,
            validators: res
                .validators
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            next: res.pagination.map(Into::into),
        })
    }
}

impl From<ValidatorSetResponse> for GetLatestValidatorSetResponse {
    fn from(res: ValidatorSetResponse) -> Self {
        Self {
            block_height: res.block_height as i64,
            validators: res.validators.into_iter().map(Into::into).collect(),
            pagination: res.next.map(Into::into),
        }
    }
}

impl TryFrom<GetValidatorSetByHeightResponse> for ValidatorSetResponse {
    type Error = TendermintError;

    fn try_from(res: GetValidatorSetByHeightResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            block_height: res.block_height as u64,
            validators: res
                .validators
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            next: res.pagination.map(Into::into),
        })
    }
}

impl From<ValidatorSetResponse> for GetValidatorSetByHeightResponse {
    fn from(res: ValidatorSetResponse) -> Self {
        Self {
            block_height: res.block_height as i64,
            validators: res.validators.into_iter().map(Into::into).collect(),
            pagination: res.next.map(Into::into),
        }
    }
}
