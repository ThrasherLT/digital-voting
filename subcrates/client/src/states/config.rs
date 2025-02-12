use anyhow::{anyhow, Result};

use protocol::config::BlockchainConfig;

use crate::{states::user::User, storage::Storage};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub blockchain_config: BlockchainConfig,
}

impl Config {
    fn storage_key(username: &str, blockchain: &str) -> String {
        format!("{}/{}/config", username, blockchain)
    }

    pub fn save(blockchain_config: BlockchainConfig, user: &User, blockchain: &str) -> Result<()> {
        let config = Config { blockchain_config };

        Storage::encrypt(&user.encryption, &config)?
            .save(&Self::storage_key(&user.username, blockchain));
        Ok(())
    }

    pub fn load(user: &User, blockchain: &str) -> Result<Self> {
        let config_storage = Storage::load(&Self::storage_key(&user.username, blockchain))
            .ok_or(anyhow!("Failed to load blockchain config"))?;
        let config: Self = config_storage.decrypt(&user.encryption)?;

        Ok(config)
    }

    pub fn get_authorities(&self) -> Vec<String> {
        self.blockchain_config
            .authorities
            .iter()
            .map(|auth| auth.addr.clone())
            .collect()
    }

    pub fn delete(user: &User, blockchain: &str) {
        Storage::delete(&Self::storage_key(&user.username, blockchain));
    }
}
