use anyhow::{anyhow, bail, Result};

use crypto::encryption::symmetric;
use protocol::config::BlockchainConfig;

use crate::storage::Storage;

use super::blockchain;

/// Storage for list of blockchains that the user had added to his profile.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UserBlockchains {
    pub blockchains: Vec<String>,
}

/// State of the user related data.
#[derive(Clone)]
pub struct User {
    pub username: String,
    pub encryption: symmetric::Encryption,
    pub blockchains: Vec<String>,
}

impl User {
    /// Login as an existing user in the browser's local storage.
    pub fn login(username: String, password: &str) -> Result<Self> {
        let user_blockchains_storage =
            Storage::load(&username).ok_or(anyhow!("User or password are incorrect"))?;
        let encryption = symmetric::Encryption::load(
            password.as_bytes(),
            user_blockchains_storage.get_metadata(),
        )?;
        let UserBlockchains { blockchains } = user_blockchains_storage.decrypt(&encryption)?;

        Ok(Self {
            username,
            encryption,
            blockchains,
        })
    }

    /// Register a new user to the browser local storage.
    pub fn register(username: String, password: &str) -> Result<Self> {
        if Storage::load(&username).is_some() {
            bail!("User already exists")
        }

        let encryption = symmetric::Encryption::new(password.as_bytes())?;
        Storage::encrypt(
            &encryption,
            &UserBlockchains {
                blockchains: Vec::new(),
            },
        )?
        .save(&username);

        Ok(Self {
            username,
            encryption,
            blockchains: Vec::new(),
        })
    }

    // TODO this leaks storage:
    /// Remove the user from browser local storage.
    pub fn delete(mut self) -> Result<()> {
        for blockchain in std::mem::take(&mut self.blockchains) {
            blockchain::delete_from_storage(&blockchain, &mut self);
        }
        Storage::delete(&self.username);

        Ok(())
    }

    /// Add a blockchain address to the user.
    pub fn add_blockchain(
        &mut self,
        blockchain: String,
        blockchain_config: BlockchainConfig,
    ) -> Result<()> {
        if self.blockchains.contains(&blockchain) {
            bail!("Blockchain already added");
        }
        self.blockchains.push(blockchain.clone());
        Storage::encrypt(
            &self.encryption,
            &UserBlockchains {
                blockchains: self.blockchains.clone(),
            },
        )?
        .save(&self.username);

        blockchain::create_in_storage(blockchain, self, blockchain_config)
    }

    /// Remove a blockchain address from the user.
    pub fn remove_blockchain(&mut self, blockchain: &str) -> Result<()> {
        if let Some(index) = self
            .blockchains
            .iter()
            .position(|existing_blockchain| existing_blockchain == blockchain)
        {
            self.blockchains.remove(index);
            Storage::encrypt(
                &self.encryption,
                &UserBlockchains {
                    blockchains: self.blockchains.clone(),
                },
            )?
            .save(&self.username);
            blockchain::delete_from_storage(blockchain, self);

            Ok(())
        } else {
            bail!("User is not a member of this blockchain");
        }
    }
}
