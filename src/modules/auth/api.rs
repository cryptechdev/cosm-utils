use crate::chain::error::ChainError;
use crate::chain::request::PaginationRequest;
use crate::clients::client::ClientAbciQuery;
use async_trait::async_trait;
use cosmrs::proto::cosmos::auth::v1beta1::{
    QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest,
    QueryAccountsResponse, QueryParamsRequest, QueryParamsResponse,
};
use cosmrs::proto::traits::Message;

use super::error::AccountError;
use super::model::{AccountResponse, AccountsResponse, Address, ParamsResponse, Account};

impl<T> Auth for T where T: ClientAbciQuery {}

#[async_trait]
pub trait Auth: ClientAbciQuery + Sized {
    async fn auth_query_account(&self, address: Address) -> Result<AccountResponse, AccountError> {
        let req = QueryAccountRequest {
            address: address.into(),
        };

        let res = self
            .query::<_, QueryAccountResponse>(req, "/cosmos.auth.v1beta1.Query/Account")
            .await?;

        let account = res.account.ok_or(AccountError::Address {
            message: "Invalid account address".to_string(),
        })?;

        #[cfg(feature = "generic")] {
            let base_account = cosmrs::proto::cosmos::auth::v1beta1::BaseAccount::decode(account.value.as_slice())
                .map_err(ChainError::prost_proto_decoding)?;

            Ok(AccountResponse {
                account: base_account.try_into()?,
            })
        }

        #[cfg(feature = "injective")] {
            let eth_account = injective_std::types::injective::types::v1beta1::EthAccount::decode(account.value.as_slice()).unwrap();
            let base_account = eth_account.base_account.ok_or(AccountError::Address {
                message: "Invalid account address".to_string(),
            })?;
            Ok(AccountResponse {
                account: base_account.try_into()?,
            })
        }
    }

    async fn auth_query_accounts(
        &self,
        pagination: Option<PaginationRequest>,
    ) -> Result<AccountsResponse, AccountError> {
        let req = QueryAccountsRequest {
            pagination: pagination.map(Into::into),
        };

        let res = self
            .query::<_, QueryAccountsResponse>(req, "/cosmos.auth.v1beta1.Query/Accounts")
            .await?;

            #[cfg(feature = "generic")] {
                let accounts: Vec<Account> = res
                .accounts
                .into_iter()
                .map(|a| {
                    cosmrs::proto::cosmos::auth::v1beta1::BaseAccount::decode(a.value.as_slice())
                        .map_err(ChainError::prost_proto_decoding)?
                        .try_into()
                })
                .collect::<Result<Vec<Account>, AccountError>>()?;
    
                Ok(AccountsResponse {
                    accounts,
                    next: res.pagination.map(Into::into),
                })
            }
    
            #[cfg(feature = "injective")] {
                let accounts: Vec<Account> = res
                    .accounts
                    .into_iter()
                    .map(|a| {
                        injective_std::types::injective::types::v1beta1::EthAccount::decode(a.value.as_slice())
                            .map_err(ChainError::prost_proto_decoding)?
                            .base_account
                            .ok_or(AccountError::Address {
                                message: "Invalid account address".to_string(),
                            })?
                            .try_into()
                    })
                    .collect::<Result<Vec<Account>, AccountError>>()?;
    
                Ok(AccountsResponse {
                    accounts,
                    next: res.pagination.map(Into::into),
                })
            }


    }

    async fn auth_query_params(&self) -> Result<ParamsResponse, AccountError> {
        let req = QueryParamsRequest {};

        let res = self
            .query::<_, QueryParamsResponse>(req, "/cosmos.auth.v1beta1.Query/Params")
            .await?;

        Ok(ParamsResponse {
            params: res.params.map(Into::into),
        })
    }
}
