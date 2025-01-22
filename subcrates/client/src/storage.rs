//! This file contains the logic for encrypting, storing and loading the client's state.

use anyhow::Result;
use codee::string::JsonSerdeCodec;
use crypto::encryption::symmetric;
use leptos::{
    logging::log,
    prelude::{Get, Set},
};
use leptos_use::storage::use_local_storage;

// TODO Update codee version without breaking.
// TODO Make sure in place encryption doesn't leak keys.
// TODO Dynamically import this file so that `states` module could be used in a non-wasm environment.

/// Encrypted storage containing metadata and all of the storage related operations.
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Storage {
    metadata: symmetric::MetaData,
    encrypted_bytes: Vec<u8>,
}

impl Storage {
    /// Encrypt any serializable struct.
    pub fn encrypt<T>(encryption: &symmetric::Encryption, to_encrypt: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        let mut storage = serde_json::to_vec(to_encrypt)?;
        let metadata = encryption.encrypt(&mut storage)?;

        Ok(Self {
            metadata,
            encrypted_bytes: storage,
        })
    }

    /// Decrypt any deserializable owned struct.
    pub fn decrypt<T>(self, encryption: &symmetric::Encryption) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut encrypted_bytes = self.encrypted_bytes.clone();
        let decrypted_bytes = encryption.decrypt(&mut encrypted_bytes, &self.metadata)?;
        let decrypted_value: T = serde_json::from_slice(decrypted_bytes)?;

        Ok(decrypted_value)
    }

    /// Return metadata of the encrypted data.
    pub fn get_metadata(&self) -> &symmetric::MetaData {
        &self.metadata
    }

    /// Load encrypoted data from browser's local storage.
    pub fn load(storage_key: &str) -> Option<Self> {
        let (storage, _, _) = use_local_storage::<Option<Storage>, JsonSerdeCodec>(storage_key);

        storage.get()
    }

    /// Save encrypted data to browser's local storage.
    pub fn save(self, storage_key: &str) {
        let (_, set_storage, _) = use_local_storage::<Option<Storage>, JsonSerdeCodec>(storage_key);
        set_storage.set(Some(self));
    }

    /// Delete encrypted data from browser's local storage.
    pub fn delete(storage_key: &str) {
        let (_, _, clear) = use_local_storage::<Option<Storage>, JsonSerdeCodec>(storage_key);
        // TODO Make sure data doesn't stay in leftover garbage:
        clear();

        // TODO Figure out why this workaround is necessary:
        if let Some(_) = Self::load(storage_key) {
            log!("Known bug: local storage deletion failed!");
        }
    }
}
