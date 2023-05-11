# cosm-utils

Simple utility trait extensions for tendermint-rpc and cosmrs.

## Goals

This crate is in very early development. It is not recommended for use in production.
I am currently using this specifically for development purposes for other projects. 

The goal of this crate is to provide simple utility methods for interacting with tendermint_rpc. 
## Inspiration

Forked from cosm-tome, but with an emphasis on easier maintenance and a more modular approach.

## Crate Status

### Features

| features | Dev Status |
| -------- | ---------- | 
| tendermint 0.34 | ✅ |
| tendermint 0.37 | ✅ |
| automatic tendermint version negotiation | ✅ |


### Clients

| Backing API | Dev Status |
| ----------- | ---------- | 
| Tendermint RPC HTTP/S | ✅ |
| Tendermint RPC Websocket | ✅ | 

### Modules

| Cosmos Module | Dev Status |
| ------------- | ------------- | 
| Auth | ✅ |
| Authz | 🚫 |
| Bank | ✅ |
| Tendermint | 🔨 |
| Crisis | 🚫 |
| Distribution | 🚫 |
| Evidence | 🚫 |
| Feegrant | 🚫 |
| Gov | 🚫 |
| Mint | 🚫 |
| Params | 🚫 |
| Slashing | 🚫 |
| Staking | 🚫 |
| Tx | 🔨 |
| Upgrade | 🚫 |
| Vesting | 🚫 |
| CosmWasm | 🔨 |
| IBC | 🚫 |


## Usage

Simply import the `prelude` and use the provided methods directly on a supported client.

```rust
    // bring traits into scope
    use std::str::FromStr;

    use cosm_utils::{
        prelude::*,
        chain::{
            coin::{Coin, Denom},
            request::TxOptions,
        },
        config::cfg::ChainConfig,
        modules::{
            auth::model::Address,
            bank::{
                api::{BankCommit, BankQuery},
                model::SendRequest,
            },
        },
        signing_key::key::{Key, SigningKey},
    };
    use tendermint_rpc::{Client, HttpClient, WebSocketClient};

    // Get you're relevant info
    // Here are some examples of what that could look like
    let mnemonic = "clump subway install trick split fiction mixed hundred much lady loyal crime fuel wrap book loud mammal plunge round penalty cereal desert essence chuckle";
    let address = "cosmos1ya34jc44vvqzdhmwnfhkax7v4l3sj3stkwy9h5";
    let key = SigningKey {
        name: "test".to_string(),
        key: Key::Mnemonic(mnemonic.to_string()),
    };
    let chain_cfg = ChainConfig {
        denom: "uatom".to_string(),
        prefix: "cosmos".to_string(),
        chain_id: "cosmoshub-4".to_string(),
        derivation_path: "m/44'/118'/0'/0/0".to_string(),
        gas_price: 0.025f64,
        gas_adjustment: 1.3f64,
    };
    let req = SendRequest {
        from: Address::from_str(address).unwrap(),
        to: Address::from_str(address).unwrap(),
        amounts: vec![Coin {
            denom: Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
            amount: 1u128,
        }],
    };
    let tx_options = TxOptions::default();

    // Create your client as usual
    let rpc_endpoint = "http://localhost:26657";
    let client = HttpClient::new(rpc_endpoint).unwrap();

    // Then use the provided methods
    let res = client
        .bank_send_commit(&chain_cfg, req, &key, &tx_options)
        .await
        .unwrap();
```