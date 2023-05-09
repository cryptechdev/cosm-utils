use std::str::FromStr;

use cosm_utils::{
    chain::{
        coin::{Coin, Denom},
        request::TxOptions,
    },
    clients::client::HashSearch,
    config::cfg::ChainConfig,
    modules::{auth::model::Address, bank::model::SendRequest},
    prelude::*,
    signing_key::key::{Key, SigningKey},
};
use tendermint_rpc::{Client, WebSocketClient};

#[tokio::test]
async fn test() {
    let mnemonic = "evoke another library napkin rich clutch evil hungry supreme smart idea discover admit remain high torch dumb immense economy truck episode coral way pupil";
    let address = "noria1ds0jnp7ful8hxmkstr5d2gxm28d8l0ecuff2v9";
    let other_address = "noria1ya34jc44vvqzdhmwnfhkax7v4l3sj3stkwy9h5";
    let key = SigningKey {
        name: "test".to_string(),
        key: Key::Mnemonic(mnemonic.to_string()),
    };
    // let rpc_endpoint = "http://localhost:26657";
    let websocket_endpoint = "ws://localhost:26657/websocket";
    // let websocket_endpoint = "wss://archive-rpc.noria.nextnet.zone/websocket";
    // let rpc_endpoint = "https://archive-rpc.noria.nextnet.zone";
    let chain_cfg = ChainConfig {
        denom: "ucrd".to_string(),
        prefix: "noria".to_string(),
        chain_id: "oasis-3".to_string(),
        derivation_path: "m/44'/118'/0'/0/0".to_string(),
        gas_price: 0.0025f64,
        gas_adjustment: 2.0f64,
    };
    // let http_client = HttpClient::new(rpc_endpoint).unwrap();
    let (client, driver) = WebSocketClient::new(websocket_endpoint).await.unwrap();
    let _handle = tokio::spawn(async move { driver.run().await });
    println!("health: {:?}", client.health().await.unwrap());

    let balance = client
        .bank_query_balance(
            Address::from_str(address).unwrap(),
            Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
        )
        .await
        .unwrap();
    println!("balance: {:?}", balance);

    let req = SendRequest {
        from: Address::from_str(address).unwrap(),
        to: Address::from_str(other_address).unwrap(),
        amounts: vec![Coin {
            denom: Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
            amount: 1u128,
        }],
    };
    let tx_options = TxOptions::default();
    let async_res = client
        .bank_send_async(&chain_cfg, req, &key, &tx_options)
        .await
        .unwrap();
    println!("async_res: {:?}", async_res);
    let res = client.hash_search(&async_res.hash).await.unwrap();
    println!("res: {:?}", res);
}
