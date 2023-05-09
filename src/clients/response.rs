use cosmrs::tendermint::abci::{response, EventAttribute};

use crate::modules::auth::model::Address;

pub trait GetCodeIds {
    fn get_code_ids(&self) -> Vec<u64>;
}

pub trait GetInstantiateAddrs {
    fn get_instantiate_addr(&self) -> Vec<Address>;
}

pub trait FindEventTags {
    fn find_event_tags(&self, event_type: String, key_name: String) -> Vec<&EventAttribute>;
}

impl FindEventTags for response::DeliverTx {
    fn find_event_tags(&self, event_type: String, key_name: String) -> Vec<&EventAttribute> {
        let mut events = vec![];
        for event in &self.events {
            if event.kind == event_type {
                for attr in &event.attributes {
                    if attr.key == key_name {
                        events.push(attr);
                    }
                }
            }
        }
        events
    }
}
