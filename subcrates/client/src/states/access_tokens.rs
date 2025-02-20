use anyhow::{anyhow, bail, Result};
use crypto::signature::blind_sign;

use crate::storage::Storage;

use super::{config::Config, signature::Signature, user::User};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AccessTokens(Vec<Option<blind_sign::Signature>>);

impl AccessTokens {
    fn storage_key(username: &str, blockchain: &str) -> String {
        format!("{}/{}/access_tokens", username, blockchain)
    }

    pub fn new(user: &User, blockchain: &str, count: usize) -> Result<Self> {
        let access_tokens = AccessTokens(vec![None; count]);

        Storage::encrypt(&user.encryption, &access_tokens)?
            .save(&Self::storage_key(&user.username, blockchain));
        Ok(access_tokens)
    }

    pub fn load(user: &User, blockchain: &str) -> Result<Self> {
        let access_tokens_storage = Storage::load(&Self::storage_key(&user.username, blockchain))
            .ok_or(anyhow!("User or password are incorrect"))?;
        let access_tokens: Self = access_tokens_storage.decrypt(&user.encryption)?;

        Ok(access_tokens)
    }

    pub fn set(
        &mut self,
        user: &User,
        blockchain: &str,
        index: usize,
        access_token: Option<blind_sign::Signature>,
        config: &Config,
        signature: &Signature,
    ) -> Result<()> {
        if self.0.len() <= index {
            bail!("Access token index does not exist");
        }
        if let Some(access_token) = &access_token {
            let verifier = blind_sign::Verifier::new(config.get_authority_pk(index).clone())?;
            verifier.verify_signature(access_token.clone(), &signature.signer.get_public_key())?;
        }

        self.0[index] = access_token;
        Storage::encrypt(&user.encryption, self)?
            .save(&Self::storage_key(&user.username, blockchain));

        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<Option<blind_sign::Signature>> {
        if self.0.len() <= index {
            bail!("Access token index does not exist");
        }

        Ok(self.0[index].clone())
    }

    pub fn is_complete(&self) -> bool {
        !self.0.is_empty() && !self.0.contains(&None)
    }

    pub fn prepare(&self) -> Result<Vec<blind_sign::Signature>> {
        let mut access_tokens = Vec::new();

        for access_token in &self.0 {
            access_tokens.push(
                access_token
                    .clone()
                    .ok_or(anyhow!("An access token is missing"))?,
            );
        }

        Ok(access_tokens)
    }

    pub fn delete(user: &User, blockchain: &str) {
        Storage::delete(&Self::storage_key(&user.username, blockchain));
    }
}
