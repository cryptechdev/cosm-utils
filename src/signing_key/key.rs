use cosmrs::bip32;
use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmrs::crypto::{secp256k1, PublicKey};
use cosmrs::tendermint::block::Height;
use cosmrs::tx::{Body, SignDoc, SignerInfo};

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
    pub async fn public_key(&self, derivation_path: &str) -> Result<PublicKey, ChainError> {
        match &self.key {
            Key::Raw(bytes) => {
                let key = raw_bytes_to_signing_key(bytes)?;
                Ok(key.public_key())
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

    #[allow(clippy::too_many_arguments)]
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

        match &self.key {
            Key::Raw(bytes) => {
                let sign_doc = build_sign_doc(
                    msgs,
                    timeout_height,
                    memo,
                    &account,
                    fee,
                    public_key,
                    chain_id,
                )?;

                let key = raw_bytes_to_signing_key(bytes)?;

                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }

            Key::Mnemonic(phrase) => {
                let sign_doc = build_sign_doc(
                    msgs,
                    timeout_height,
                    memo,
                    &account,
                    fee,
                    public_key,
                    chain_id,
                )?;

                let key = mnemonic_to_signing_key(phrase, derivation_path)?;

                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }

            #[cfg(feature = "keyring")]
            Key::Keyring(params) => {
                let sign_doc = build_sign_doc(
                    msgs,
                    timeout_height,
                    memo,
                    &account,
                    fee,
                    public_key,
                    chain_id,
                )?;

                let entry = Entry::new(&self.name, &params.user)?;
                let key = mnemonic_to_signing_key(&entry.get_password()?, derivation_path)?;

                let raw = sign_doc.sign(&key).map_err(ChainError::crypto)?;
                Ok(raw.into())
            }
        }
    }

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

    pub fn random_mnemonic(key_name: String) -> UserKey {
        let mnemonic = bip32::Mnemonic::random(OsRng, Default::default());

        UserKey {
            name: key_name,
            key: Key::Mnemonic(mnemonic.phrase().to_string()),
        }
    }
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
    _derivation_path: &str,
) -> Result<secp256k1::SigningKey, ChainError> {
    use ethers_signers::{coins_bip39::English, MnemonicBuilder};

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic)
        .index(0u32)
        .unwrap()
        .build()
        .unwrap();
    let bytes = wallet.signer().clone().to_bytes();
    Ok(secp256k1::SigningKey::from_slice(&bytes).unwrap())
}

fn raw_bytes_to_signing_key(bytes: &[u8]) -> Result<secp256k1::SigningKey, ChainError> {
    secp256k1::SigningKey::from_slice(bytes).map_err(ChainError::crypto)
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

    // NOTE: if we are making requests in parallel with the same key, we need to serialize `account.sequence` to avoid errors
    let auth_info =
        SignerInfo::single_direct(public_key, account.sequence).auth_info(fee.try_into()?);

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
    use bech32::{self, ToBase32};
    use cosmrs::AccountId;
    use ethers::signers::{coins_bip39::English, MnemonicBuilder, Signer};

    use crate::signing_key::key::{Key, UserKey};

    /// Attempt at getting injective key generation to work
    #[ignore]
    #[tokio::test]
    async fn mnemonic_deterministic() {
        let mnemonic = "cryptech dev key"; // for this test, the  Cryptech Dev Wallet was used
        let addr = "inj1rmxnw6nmqqsnsk0d4c72v9794zfkgxkx23fart"; // address taken from injective and keplr
        let index = 0u32;

        println!("confirmed injective address from keplr: {}", addr);

        let user_key = UserKey {
            name: "test".to_string(),
            key: Key::Mnemonic(mnemonic.to_string()),
        };

        let inj_addr = user_key.to_addr("inj", "test").await.unwrap();

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
}
