use cosmrs::bip32;
use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmrs::crypto::{secp256k1, PublicKey};
use cosmrs::tendermint::block::Height;
use cosmrs::tx::{Body, SignDoc, SignerInfo};

use ethers_signers::Signer;
#[cfg(feature = "keyring")]
use keyring::Entry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::chain::error::ChainError;
use crate::chain::fee::Fee;
use crate::chain::msg::IntoAny;
use crate::chain::tx::RawTx;
use crate::modules::auth::model::{Account, Address};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct UserKey {
    /// human readable key name
    pub name: String,
    /// private key associated with `name`
    pub key: Key,
}

impl UserKey {
    #[cfg(not(feature = "injective"))]
    pub async fn public_key(&self, derivation_path: &str) -> Result<PublicKey, ChainError> {
        match &self.key {
            Key::Raw(bytes) => {
                let key = raw_bytes_to_signing_key(bytes)?;
                Ok(key.signer().public_key())
            }

            Key::Mnemonic(phrase) => {
                let key = mnemonic_to_signing_key(phrase, derivation_path)?;
                Ok(key.public_key())
            }

            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&self.name, &params.user)?;
                let key = mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?;
                Ok(key.public_key())
            }
        }
    }

    #[cfg(feature = "injective")]
    pub async fn verifying_key(
        &self,
        derivation_path: &str,
    ) -> Result<ecdsa::VerifyingKey<bip32::secp256k1::Secp256k1>, ChainError> {
        println!("verifying key");
        match &self.key {
            Key::Raw(bytes) => {
                let key = raw_bytes_to_signing_key(bytes)?;
                Ok(key.signer().verifying_key().clone())
            }

            Key::Mnemonic(phrase) => {
                let key = mnemonic_to_signing_key(phrase, derivation_path)?;
                Ok(key.signer().verifying_key().clone())
            }

            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&self.name, &params.user)?;
                let key = mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?;
                Ok(key.signer().verifying_key().clone())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(not(feature = "injective"))]
    pub async fn sign(
        &self,
        msgs: Vec<impl IntoAny>,
        timeout_height: u64,
        memo: &str,
        account: Account,
        fee: Fee,
        chain_id: &str,
        derivation_path: &str,
    ) -> Result<RawTx, ChainError> {
        let public_key = if account.pubkey.is_none() {
            Some(self.public_key(derivation_path).await?)
        } else {
            account.pubkey
        };

        let sign_doc = build_sign_doc(
            msgs,
            timeout_height,
            memo,
            &account,
            fee,
            public_key,
            chain_id,
        )?;

        match &self.key {
            Key::Raw(bytes) => {
                let key = raw_bytes_to_signing_key(bytes)?;
                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }

            Key::Mnemonic(phrase) => {
                let key = mnemonic_to_signing_key(phrase, derivation_path)?;
                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }

            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&self.name, &params.user)?;
                let key = mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?;
                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "injective")]
    pub async fn sign(
        &self,
        msgs: Vec<impl IntoAny>,
        timeout_height: u64,
        memo: &str,
        account: Account,
        fee: Fee,
        chain_id: &str,
        derivation_path: &str,
    ) -> Result<RawTx, ChainError> {
        // let public_key = if account.pubkey.is_none() {
        //     Some(self.public_key(derivation_path).await?)
        // } else {
        //     account.pubkey
        // };

        println!("before build");
        let sign_doc = build_sign_doc(msgs, timeout_height, memo, &account, fee, None, chain_id)?;

        println!("after build");
        match &self.key {
            Key::Raw(bytes) => {
                let key = raw_bytes_to_signing_key(bytes)?;
                let raw = sign_doc_sign(sign_doc, key).await;
                Ok(raw.into())
            }

            Key::Mnemonic(phrase) => {
                let key = mnemonic_to_signing_key(phrase, derivation_path)?;
                let raw = sign_doc_sign(sign_doc, key).await;
                Ok(raw.into())
            }

            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&self.name, &params.user)?;
                let key = mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?;
                let raw = sign_doc_sign(sign_doc, key).await;
                Ok(raw.into())
            }
        }
    }

    #[cfg(not(feature = "injective"))]
    pub async fn to_addr(
        &self,
        prefix: &str,
        derivation_path: &str,
    ) -> Result<Address, ChainError> {
        let account = self
            .public_key(derivation_path)
            .await?
            .account_id(prefix)
            .map_err(ChainError::crypto)?;
        Ok(account.into())
    }

    #[cfg(feature = "injective")]
    pub async fn to_addr(
        &self,
        prefix: &str,
        derivation_path: &str,
    ) -> Result<Address, ChainError> {
        use std::str::FromStr;

        use bech32::ToBase32;
        use ethers_signers::{Signer, Wallet};

        println!("to addr");
        let wallet = match &self.key {
            Key::Raw(bytes) => raw_bytes_to_signing_key(&bytes)?,
            Key::Mnemonic(mnemonic) => mnemonic_to_signing_key(&mnemonic, derivation_path)?,
            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let entry = Entry::new(&self.name, &params.user)?;
                mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?
            }
        };

        let inj_addr = bech32::encode(
            "inj",
            wallet.address().as_bytes().to_base32(),
            bech32::Variant::Bech32,
        )
        .unwrap();

        Ok(Address::from_str(&inj_addr).unwrap())
    }

    pub fn random_mnemonic(key_name: String) -> UserKey {
        let mnemonic = bip32::Mnemonic::random(OsRng, Default::default());

        UserKey {
            name: key_name,
            key: Key::Mnemonic(mnemonic.phrase().to_string()),
        }
    }
}

#[cfg(feature = "injective")]
pub async fn sign_doc_sign(
    sign_doc: SignDoc,
    signing_key: ethers_signers::Wallet<ecdsa::SigningKey<bip32::secp256k1::Secp256k1>>,
) -> cosmrs::tx::Raw {
    // TODO(tarcieri): optimize away `Clone` calls with reference conversions
    println!("sign doc sign");
    let sign_doc_bytes = sign_doc.clone().into_bytes().unwrap();
    let signature = signing_key.sign_message(&sign_doc_bytes).await.unwrap();

    cosmrs::proto::cosmos::tx::v1beta1::TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature.to_vec()],
    }
    .into()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[non_exhaustive]
pub enum Key {
    /// Create a key from a set of bytes.
    Raw(Vec<u8>),

    /// Mnemonic allows you to pass the private key mnemonic words
    /// to Cosm-orc for configuring a transaction signing key.
    /// DO NOT USE FOR MAINNET
    Mnemonic(String),

    // TODO: Add Keyring password CRUD operations
    /// Use OS Keyring to access private key.
    /// Safe for testnet / mainnet.
    #[cfg(feature = "keyring")]
    Keyring(KeyringParams),
    // TODO: Add ledger support(under a new ledger feature flag / Key variant)
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct KeyringParams {
    /// The account that this key will be associated with
    /// in the system keyring.
    pub user: String,
}

#[cfg(not(feature = "injective"))]
fn mnemonic_to_signing_key(
    mnemonic: &str,
    derivation_path: &str,
) -> Result<secp256k1::SigningKey, ChainError> {
    let seed = bip32::Mnemonic::new(mnemonic, bip32::Language::English)
        .map_err(|_| ChainError::Mnemonic)?
        .to_seed("");

    secp256k1::SigningKey::derive_from_path(
        seed,
        &derivation_path
            .parse()
            .map_err(|_| ChainError::DerviationPath)?,
    )
    .map_err(|_| ChainError::DerviationPath)
}

#[cfg(feature = "injective")]
fn mnemonic_to_signing_key(
    mnemonic: &str,
    derivation_path: &str,
) -> Result<ethers_signers::Wallet<ecdsa::SigningKey<bip32::secp256k1::Secp256k1>>, ChainError> {
    println!("mnemonic to signing key");
    use ethers_signers::{coins_bip39::English, MnemonicBuilder};
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .index(0u32)
        .unwrap()
        .build()
        .unwrap();
    Ok(wallet)
}

#[cfg(not(feature = "injective"))]
fn raw_bytes_to_signing_key(bytes: &[u8]) -> Result<secp256k1::SigningKey, ChainError> {
    secp256k1::SigningKey::from_slice(bytes).map_err(ChainError::crypto)
}

#[cfg(feature = "injective")]
fn raw_bytes_to_signing_key(
    bytes: &[u8],
) -> Result<ethers_signers::Wallet<ecdsa::SigningKey<bip32::secp256k1::Secp256k1>>, ChainError> {
    use ethers_signers::Wallet;

    Ok(Wallet::from_bytes(bytes).unwrap())
}

fn build_sign_doc(
    msgs: Vec<impl IntoAny>,
    timeout_height: u64,
    memo: &str,
    account: &Account,
    fee: Fee,
    public_key: Option<PublicKey>,
    chain_id: &str,
) -> Result<SignDoc, ChainError> {
    println!("build sign doc");
    let timeout: Height = timeout_height.try_into()?;

    let tx = Body::new(
        msgs.into_iter()
            .map(|m| m.into_any())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ChainError::ProtoEncoding {
                message: e.to_string(),
            })?,
        memo,
        timeout,
    );

    //  NOTE: if we are making requests in parallel with the same key, we need to serialize `account.sequence` to avoid errors
    println!("before single");
    let auth_info =
        SignerInfo::single_direct(public_key, account.sequence).auth_info(fee.try_into()?);
    println!("afer single");
    // let auth_info = SignerInfo {
    //     public_key: None,
    //     mode_info: todo!(),
    //     sequence: todo!(),
    // };

    SignDoc::new(
        &tx,
        &auth_info,
        &chain_id.parse().map_err(|_| ChainError::ChainId {
            chain_id: chain_id.to_string(),
        })?,
        account.account_number,
    )
    .map_err(ChainError::proto_encoding)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bech32::{self, ToBase32};
    use cosmrs::AccountId;
    use ethers::signers::{coins_bip39::English, MnemonicBuilder, Signer};
    use tendermint_rpc::HttpClient;

    use crate::{
        chain::{
            coin::{Coin, Denom},
            request::TxOptions,
        },
        config::cfg::ChainConfig,
        modules::{auth::model::Address, bank::model::SendRequest},
        prelude::BankTxCommit,
        signing_key::key::{Key, UserKey},
    };

    /// Attempt at getting injective key generation to work
    #[tokio::test]
    async fn mnemonic_deterministic() {
        let mnemonic = "device relax sibling follow seminar bless admit ticket attract other cabin tackle crumble venture bunker prosper wise monster patrol wrestle royal latin effort pilot"; // for this test, the  Cryptech Dev Wallet was used
        let addr = "inj1rmxnw6nmqqsnsk0d4c72v9794zfkgxkx23fart"; // address taken from injective and keplr
        let config = ChainConfig {
            denom: "inj".into(),
            prefix: "inj".into(),
            chain_id: "injective-1".into(),
            derivation_path: "m/44'/60'/0'/0/0".into(),
            gas_price: 500000000.0,
            gas_adjustment: 1.3,
        };
        let index = 0u32;

        println!("confirmed injective address from keplr: {}", addr);

        let user_key = UserKey {
            name: "test".to_string(),
            key: Key::Mnemonic(mnemonic.to_string()),
        };

        let inj_addr = user_key.to_addr("inj", "m/44'/60'/0'/0/0").await.unwrap();

        assert_eq!(inj_addr.as_ref(), addr);

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(mnemonic)
            .index(index)
            .unwrap()
            .build()
            .unwrap();

        let inj_addr = bech32::encode(
            "inj",
            wallet.address().as_bytes().to_base32(),
            bech32::Variant::Bech32,
        )
        .unwrap();

        println!("inj_addr: {}", inj_addr);
        assert_eq!(addr, inj_addr.to_string());

        let account = AccountId::new("inj", wallet.address().as_bytes()).unwrap();

        println!("account address: {}", account);
        assert_eq!(addr, account.to_string());
    }
    #[tokio::test]
    async fn test_injective_signing() {
        let mnemonic = "device relax sibling follow seminar bless admit ticket attract other cabin tackle crumble venture bunker prosper wise monster patrol wrestle royal latin effort pilot"; // for this test, the  Cryptech Dev Wallet was used
        let addr = "inj1rmxnw6nmqqsnsk0d4c72v9794zfkgxkx23fart"; // address taken from injective and keplr
        let chain_cfg = ChainConfig {
            denom: "inj".into(),
            prefix: "inj".into(),
            chain_id: "injective-888".into(),
            derivation_path: "m/44'/60'/0'/0/0".into(),
            gas_price: 700000000.0,
            gas_adjustment: 1.3,
        };
        let rpc_endpoint = "https://injective-testnet-rpc.polkachu.com:443".to_string();
        let key = UserKey {
            name: "test".to_string(),
            key: Key::Mnemonic(mnemonic.to_string()),
        };
        let client = HttpClient::new(rpc_endpoint.as_str()).unwrap();
        let req = SendRequest {
            from: Address::from_str(addr).unwrap(),
            to: Address::from_str(addr).unwrap(),
            amounts: vec![Coin {
                denom: Denom::from_str("inj").unwrap(),
                amount: 3,
            }],
        };
        let tx_options = TxOptions {
            timeout_height: None,
            fee: None,
            account: None,
            memo: "test".to_string(),
        };
        let _ = client
            .bank_send_commit(&chain_cfg, req, &key, &tx_options)
            .await
            .unwrap();
    }
}
