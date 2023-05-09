use crate::modules::auth::model::Address;

pub trait GetCodeIds {
    fn get_code_ids(&self) -> Vec<u64>;
}

pub trait GetInstantiateAddrs {
    fn get_instantiate_addr(&self) -> Vec<Address>;
}
