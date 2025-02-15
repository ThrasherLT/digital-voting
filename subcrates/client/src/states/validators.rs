use anyhow::{anyhow, bail, Result};

use crypto::signature::{blind_sign, digital_sign};
use protocol::config::ElectionConfig;

use crate::{states::user::User, storage::Storage};

use super::signature::Signature;

pub struct Validators(Vec<Validator>);

struct Validator {
    blinded_pk: blind_sign::BlindedMessage,
    unblinder: blind_sign::Unblinder,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ValidatorStorage {
    blinded_pk: blind_sign::BlindedMessage,
    unblinding_secret: blind_sign::UnblindingSecret,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ValidatorsStorage(Vec<ValidatorStorage>);

impl Validators {
    fn storage_key(username: &str, blockchain: &str) -> String {
        format!("{}/{}/validators", username, blockchain)
    }

    pub fn new(
        blockchain_config: &ElectionConfig,
        signer_pk: digital_sign::PublicKey,
        user: &User,
        blockchain: &str,
    ) -> Result<Self> {
        let mut validators = Self(Vec::new());
        let mut validators_storage = ValidatorsStorage::default();

        for auth in &blockchain_config.authorities {
            let (blinded_pk, unblinder) =
                blind_sign::Blinder::new(auth.authority_key.clone())?.blind(&signer_pk)?;
            validators_storage.0.push(ValidatorStorage {
                blinded_pk: blinded_pk.clone(),
                unblinding_secret: unblinder.get_unblinding_secret(),
            });
            validators.0.push(Validator {
                blinded_pk,
                unblinder,
            });
        }
        Storage::encrypt(&user.encryption, &validators_storage)?
            .save(&Self::storage_key(&user.username, blockchain));

        Ok(validators)
    }

    pub fn load(blockchain_config: &ElectionConfig, user: &User, blockchain: &str) -> Result<Self> {
        let validators_storage = Storage::load(&Self::storage_key(&user.username, blockchain))
            .ok_or(anyhow!("User or password are incorrect"))?;
        let validators_storage: ValidatorsStorage = validators_storage.decrypt(&user.encryption)?;

        let mut validators = Validators(Vec::new());

        for (i, validator_storage) in validators_storage.0.iter().enumerate() {
            validators.0.push(Validator {
                blinded_pk: validator_storage.blinded_pk.clone(),
                unblinder: blind_sign::Unblinder::from_pk_and_secret(
                    blockchain_config.authorities[i].authority_key.clone(),
                    validator_storage.unblinding_secret.clone(),
                )?,
            });
        }

        Ok(validators)
    }

    pub fn validate(
        &self,
        signature: &Signature,
        index: usize,
        blind_signature: blind_sign::BlindSignature,
    ) -> Result<blind_sign::Signature> {
        if self.0.len() <= index {
            bail!("Validator index does not exist");
        }

        let access_token = self.0[index]
            .unblinder
            .unblind_signature(blind_signature.clone(), &signature.signer.get_public_key())?;

        Ok(access_token)
    }

    pub fn get_blinded_pks(&self) -> Vec<blind_sign::BlindedMessage> {
        self.0
            .iter()
            .map(|validator| validator.blinded_pk.clone())
            .collect()
    }

    pub fn delete(user: &User, blockchain: &str) {
        Storage::delete(&Self::storage_key(&user.username, blockchain));
    }
}
