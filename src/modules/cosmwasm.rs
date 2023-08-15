use std::str::FromStr;

use cosmrs::{cosmwasm::{MsgStoreCode, MsgStoreCodeResponse}, tendermint::Hash};

use crate::chain::{request::{ExecRequest, TryFromEvents}, error::ChainError};

impl TryFromEvents for MsgStoreCodeResponse {
    fn try_from_events(events: &mut std::slice::Iter<'_, cosmrs::tendermint::abci::Event>) -> Result<Self, crate::chain::error::ChainError> {
        #[cfg(feature = "generic")]
        let code_ids = res
            .find_event_tags("store_code".to_string(), "code_id".to_string())
            .into_iter()
            .map(|x| x.value.parse::<u64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CosmwasmError::MissingEvent)?;

        #[cfg(feature = "injective")] {
            let event_str = "cosmwasm.wasm.v1.EventCodeStored";
            let code_id_str = "code_id";
            let checksum_str = "checksum";
            let event = events.find(|event| event.kind == event_str).ok_or(ChainError::MissingEvent(event_str.to_string()))?;
            let code_id = event.attributes
                .iter()
                .find_map(|attr| { 
                    if attr.key == code_id_str {
                        Some(attr.value.clone())
                    } else {
                        None
                    }
                }).ok_or(ChainError::MissingEvent(code_id_str.to_string()))?;
            let code_id = code_id.replace('\"',"").parse::<u64>().unwrap(); // TODO
            let checksum = event.attributes
                .iter()
                .find_map(|attr| { 
                    if attr.key == checksum_str {
                        Some(attr.value.clone())
                    } else {
                        None
                    }
                }).ok_or(ChainError::MissingEvent(checksum_str.to_string()))?;
            Ok(MsgStoreCodeResponse { code_id, checksum: Hash::from_str(&checksum)? })
        }
    }
}

impl ExecRequest for MsgStoreCode {
    type Response = MsgStoreCodeResponse;
}