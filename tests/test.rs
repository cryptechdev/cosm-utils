use std::str::FromStr;

use cosm_utils::{
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

#[tokio::test]
async fn test() {
    let mnemonic = "clump subway install trick split fiction mixed hundred much lady loyal crime fuel wrap book loud mammal plunge round penalty cereal desert essence chuckle";
    let address = "noria1ya34jc44vvqzdhmwnfhkax7v4l3sj3stkwy9h5";
    let other_address = "noria1xd9qy35ys7jzp8a3nl334hexs5c9m8gwuylfj5";
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
    let _ = tokio::spawn(async move { driver.run().await });
    println!("health {:?}", client.health().await.unwrap());

    let balance = client
        .bank_query_balance(
            Address::from_str(other_address).unwrap(),
            Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
        )
        .await
        .unwrap();
    println!("{:?}", balance);

    let req = SendRequest {
        from: Address::from_str(address).unwrap(),
        to: Address::from_str(address).unwrap(),
        amounts: vec![Coin {
            denom: Denom::from_str(chain_cfg.denom.as_str()).unwrap(),
            amount: 1u128,
        }],
    };
    let tx_options = TxOptions::default();
    let res = client
        .bank_send_commit(&chain_cfg, req, &key, &tx_options)
        .await
        .unwrap();
    println!("{:?}", res);
}
