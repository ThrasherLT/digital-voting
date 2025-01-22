//! This is a wrapper module for symmetric encryption using AEAD.

use ring::{
    aead, pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use thiserror::Error;

/// Error type for symmetric encryption operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Could not generate an SALT from system random.
    #[error("Failed to generate SALT")]
    SaltGeneration,
    /// Could not generate an SALT from system random.
    #[error("Failed to generate SALT")]
    NonceGeneration,
    /// Could not convert iteration count to non zero number.
    #[error("Bad non zero iteration count")]
    IterationCount,
    /// Failed to derive encryption key.
    #[error("Failed to derive encryption key")]
    KeyDerive,
    /// Encryption failed with the provided username and password.
    #[error("Encryption failed")]
    Encryption,
    /// Decryption failed with the provided username and password.
    #[error("Decryption failed")]
    Decryption,
}
type Result<T> = std::result::Result<T, Error>;

/// Length of the salt segment in bytes. Chosen because this is the usual recommended byte cound.
const SALT_LEN: usize = 32;

/// Newtype for unique SALT generated for each user and used for deriving salt for encryption key.
#[derive(Clone)]
struct Salt([u8; SALT_LEN]);

impl Salt {
    /// Generate new SALT.
    ///
    /// # Returns
    ///
    /// New SALT.
    ///
    /// # Errors
    ///
    /// If `SystemRandom` fails to generate random bytes.
    fn new() -> Result<Self> {
        let mut salt = [0u8; SALT_LEN];
        SystemRandom::new()
            .fill(&mut salt)
            .map_err(|_| Error::SaltGeneration)?;

        Ok(Salt(salt))
    }
}

impl AsRef<[u8]> for Salt {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Newtype for Nonce which is used by AEAD to prevent replay attacks.
struct Nonce([u8; aead::NONCE_LEN]);

impl Nonce {
    /// Generate new nonce.
    ///
    /// # Returns
    ///
    /// New nonce.
    ///
    /// # Errors
    ///
    /// If `SystemRandom` fails to generate random bytes.
    fn new() -> Result<Self> {
        let mut nonce = [0u8; aead::NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| Error::NonceGeneration)?;

        Ok(Self(nonce))
    }
}

impl AsRef<[u8]> for Nonce {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Newtype for metadata which is stored alongside the encrypted message.
#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct MetaData(
    #[serde_as(as = "serde_with::base64::Base64")] [u8; SALT_LEN + aead::NONCE_LEN],
);

impl MetaData {
    /// Create new metadata containing SALT and nonce.
    ///
    /// # Arguments
    ///
    /// `salt` - The SALT generated for the specific user.
    /// `nonce` - The nonce used to decrypt, specific to each encrypted message.
    ///
    /// # Returns
    ///
    /// New metadata.
    fn new(salt: &Salt, nonce: &Nonce) -> Self {
        let mut buffer = [0u8; SALT_LEN + aead::NONCE_LEN];
        buffer[0..SALT_LEN].copy_from_slice(&salt.0);
        buffer[SALT_LEN..].copy_from_slice(&nonce.0);

        Self(buffer)
    }

    /// Create new metadata from bytes containing SALT and nonce.
    ///
    /// # Arguments
    ///
    /// `bytes` - The bytes of size `SALT_LEN` + `NONCE_LEN` containing SALT and nonce.
    ///
    /// # Returns
    ///
    /// New metadata.
    #[must_use]
    pub fn from_bytes(bytes: [u8; SALT_LEN + aead::NONCE_LEN]) -> Self {
        Self(bytes)
    }

    /// Get nonce from metadata.
    ///
    /// # Returns
    ///
    /// Nonce.
    fn get_nonce(&self) -> Nonce {
        let mut buffer = [0u8; aead::NONCE_LEN];
        buffer.copy_from_slice(&self.0[SALT_LEN..]);

        Nonce(buffer)
    }

    /// Get SALT from metadata.
    ///
    /// # Returns
    ///
    /// SALT.
    fn get_salt(&self) -> Salt {
        let mut buffer = [0u8; SALT_LEN];
        buffer.copy_from_slice(&self.0[..SALT_LEN]);

        Salt(buffer)
    }
}

impl AsRef<[u8]> for MetaData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// Encryption state structure.
#[derive(Clone)]
pub struct Encryption {
    /// The AEAD key used to encrypt and decrypt messages.
    key: aead::LessSafeKey,
    /// The SALT specific to the user and required to encrypt and decrypt messages.
    /// Will also be stored alongside the encrypted message.
    salt: Salt,
}

impl Encryption {
    /// Create new encryption instance with a new username and password.
    ///
    /// # Arguments
    ///
    /// `password` - The new password for the user which will be used to encrypt and decrypt messages.
    ///
    /// # Returns
    ///
    /// A new instance of encryption state.
    ///
    /// # Errors
    ///
    /// If key derivation fails.
    /// If SALT generation fails.
    pub fn new(password: &[u8]) -> Result<Self> {
        Self::derive_key(password, Salt::new()?)
    }

    /// Load an existing encryption instance with a the username and password which were used to create it.
    ///
    /// # Arguments
    ///
    /// `password` - The password for the user which will be used to encrypt and decrypt messages.
    /// `metadata` - The metadata for loading the SALT of the user.
    ///
    /// # Returns
    ///
    /// A new instance of encryption state.
    ///
    /// # Errors
    ///
    /// If key derivation fails.
    pub fn load(password: &[u8], metadata: &MetaData) -> Result<Self> {
        Self::derive_key(password, metadata.get_salt())
    }

    /// Derive key for the username, password and SALT.
    ///
    /// # Arguments
    ///
    /// `username` - The username for the user which will be used to encrypt and decrypt messages.
    /// `password` - The password for the user which will be used to encrypt and decrypt messages.
    /// `salt` - The SALT of the user.
    ///
    /// # Returns
    ///
    /// A new instance of encryption state.
    ///
    /// # Errors
    ///
    /// If key derivation fails.
    fn derive_key(password: &[u8], salt: Salt) -> Result<Self> {
        let mut key = [0; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            100.try_into().map_err(|_| Error::IterationCount)?,
            salt.as_ref(),
            password,
            &mut key,
        );
        let key = aead::UnboundKey::new(&ring::aead::CHACHA20_POLY1305, &key)
            .map_err(|_| Error::KeyDerive)?;
        let key = aead::LessSafeKey::new(key);

        Ok(Self { key, salt })
    }

    /// Encrypt a message.
    /// Note: The encryption will happen in place, so the bytes in the buffer will be
    /// modified and the buffer will probably be expanded.
    ///
    /// # Arguments
    ///
    /// `to_encrypt` - The message to be encrypted.
    ///
    /// # Returns
    ///
    /// The metadata for the encrypted message, which should be stored alongside the encrypted message.
    ///
    /// # Errors
    ///
    /// If encryption fails.
    pub fn encrypt(&self, to_encrypt: &mut Vec<u8>) -> Result<MetaData> {
        let nonce = Nonce::new()?;
        let metadata = MetaData::new(&self.salt, &nonce);
        self.key
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce.0),
                aead::Aad::from(metadata.as_ref()),
                to_encrypt,
            )
            .map_err(|_| Error::Encryption)?;

        Ok(metadata)
    }

    // TODO not sure about to_decrypt buffer having the tag attached after this returns.
    /// Decrypt a message.
    /// Note: The decryption will happen in place, so the bytes in the buffer will be
    /// modified and the buffer will probably be expanded. The original buffer will also
    /// contain the encryption tag though, so the return value should be used to read
    /// the decrypted message.
    ///
    /// # Arguments
    ///
    /// `to_encrypt` - The message to be encrypted.
    /// `metadata` - The metadata which was used during message encryption.
    ///
    /// # Returns
    ///
    /// A slice into the decrypted message.
    ///
    /// # Errors
    ///
    /// If encryption fails.
    pub fn decrypt<'a>(
        &self,
        to_decrypt: &'a mut [u8],
        metadata: &MetaData,
    ) -> Result<&'a mut [u8]> {
        let decrypted = self
            .key
            .open_in_place(
                aead::Nonce::assume_unique_for_key(metadata.get_nonce().0),
                aead::Aad::from(metadata.as_ref()),
                to_decrypt,
            )
            .map_err(|_| Error::Decryption)?;

        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_encryption() {
        let password = b"Password";
        let plaintext = b"Big secret";

        let encryption = Encryption::new(password).unwrap();

        // Running twice just in case there are issues with nonce.
        for _ in 0..1 {
            let mut buffer: Vec<u8> = plaintext.into();

            let metadata = encryption.encrypt(&mut buffer).unwrap();

            assert_ne!(buffer[..plaintext.len()], *plaintext);

            // Make sure the same nonce is allowed to be used twice to decrypt.
            for _ in 0..1 {
                let decryption = Encryption::load(password, &metadata).unwrap();
                let decrypted_plaintext = decryption.decrypt(&mut buffer, &metadata).unwrap();

                assert_eq!(decrypted_plaintext, plaintext);
            }
        }
    }

    #[wasm_bindgen_test]
    #[test]
    fn test_encryption_wrong_password() {
        let password = b"Password";
        let wrong_password = b"Passwordd";
        let plaintext = b"Big secret";

        let encryption = Encryption::new(password).unwrap();

        let mut buffer: Vec<u8> = plaintext.into();

        let metadata = encryption.encrypt(&mut buffer).unwrap();

        assert_ne!(buffer[..plaintext.len()], *plaintext);

        let decryption = Encryption::load(wrong_password, &metadata).unwrap();
        assert!(decryption.decrypt(&mut buffer, &metadata).is_err());
        assert_ne!(buffer[..plaintext.len()], *plaintext);
    }
}
