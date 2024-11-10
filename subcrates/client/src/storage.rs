//! This file contains the logic for encrypting, storing and loading the client's state.

use anyhow::Result;
use codee::string::JsonSerdeCodec;
use crypto::{
    encryption::symmetric,
    signature::{blind_sign, digital_sign},
};
use leptos::{SignalGet, SignalSet};
use leptos_use::storage::use_local_storage;
use protocol::candidate_id::CandidateId;

// TODO Add documentation.

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct KeyStore {
    pub signer_sk: Option<digital_sign::SecretKey>,
    pub authority_key: Option<blind_sign::PublicKey>,
    pub unblinding_secret: Option<blind_sign::UnblindingSecret>,
    pub access_token: Option<blind_sign::Signature>,
    pub candidate: Option<CandidateId>,
}

impl KeyStore {
    pub fn encrypt(self, encryption: &symmetric::Encryption) -> Result<Storage> {
        let mut storage = serde_json::to_vec(&self)?;
        let metadata = encryption.encrypt(&mut storage)?;

        Ok(Storage {
            metadata,
            encrypted_bytes: storage,
        })
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Storage {
    metadata: symmetric::MetaData,
    encrypted_bytes: Vec<u8>,
}

impl Storage {
    pub fn get_metadata(&self) -> &symmetric::MetaData {
        &self.metadata
    }

    pub fn load() -> Option<Self> {
        let (storage, _, _) = use_local_storage::<Option<Storage>, JsonSerdeCodec>("signle-user");

        storage.get()
    }

    pub fn save(self) {
        let (_, set_storage, _) =
            use_local_storage::<Option<Storage>, JsonSerdeCodec>("signle-user");
        set_storage.set(Some(self));
    }

    pub fn decrypt(self, encryption: &symmetric::Encryption) -> Result<KeyStore> {
        // TODO This might be a major hazzard so should rethink the whole decryption in place thing:
        // Cloning to avoid decrypting in storage.
        let mut encrypted_bytes = self.encrypted_bytes.clone();
        let decrypted = encryption.decrypt(&mut encrypted_bytes, &self.metadata)?;
        let key_store: KeyStore = serde_json::from_slice(decrypted)?;

        Ok(key_store)
    }
}
