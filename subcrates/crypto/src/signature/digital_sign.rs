//! A simple digital signature based on ed25519.

// TODO add examples when API is more stable.
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::impl_key_display;

/// Errors that can occur when working with digital signatures.
#[derive(Error, Debug)]
pub enum Error {
    /// The signature could not be verified, possibly because the message was tampered with.
    #[error("Signature could not be verified")]
    SignatureInvalid,
    /// pkcs8 generation failed.
    #[error("Failed to generate pkcs8 while generating a new keypair")]
    Pkcs8GenerationFailed,
    // TODO there should be a way to include key rejection reason.
    /// Key pair generation failed.
    #[error("Failed to generate a new keypair")]
    KeyPairGenerationFailed,
    /// Base64 conversion error.
    #[error("Invalid base64 {:?}", .0)]
    InvalidBase64(#[from] base64::DecodeError),
}
type Result<T> = std::result::Result<T, Error>;

/// A public key for digital signatures.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PublicKey(pub Vec<u8>);

impl_key_display!(PublicKey);

/// A digital signature.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Signature(pub Vec<u8>);

impl_key_display!(Signature);

/// The signature struct containing the key pair.
/// Constructed only for signing, verification is done with an associated function.
pub struct Signer {
    /// Key pair for signing.
    key_pair: Ed25519KeyPair,
}

impl Signer {
    /// Associated function for verifying a signature.
    ///
    /// # Arguments
    ///
    /// * `message` - The message that was signed.
    /// * `signature` - The signature to verify.
    /// * `peer_public_key` - The public key of the peer that signed the message.
    ///
    /// # Errors
    pub fn verify(message: &[u8], signature: &[u8], peer_public_key: &PublicKey) -> Result<()> {
        let unparsed_public_key = UnparsedPublicKey::new(&signature::ED25519, peer_public_key);
        unparsed_public_key
            .verify(message, signature)
            .map_err(|_| Error::SignatureInvalid)?;

        Ok(())
    }

    /// Sign a message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to sign.
    ///
    /// # Returns
    ///
    /// The signature.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.key_pair.sign(message).as_ref().to_vec()
    }

    /// Get the public key.
    ///
    /// # Returns
    ///
    /// The public key.
    #[must_use]
    pub fn get_public_key(&self) -> PublicKey {
        let public_key = self.key_pair.public_key();
        PublicKey(public_key.as_ref().to_vec())
    }

    /// Create a new signature.
    ///
    /// # Returns
    ///
    /// The signature struct containing the keypair.
    ///
    /// # Errors
    ///
    /// If Pkcs8 or key pair generation fails.
    pub fn new() -> Result<Self> {
        let rng = &ring::rand::SystemRandom::new();
        let pkcs8 = signature::Ed25519KeyPair::generate_pkcs8(rng)
            .map_err(|_| Error::Pkcs8GenerationFailed)?;
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())
            .map_err(|_| Error::KeyPairGenerationFailed)?;

        Ok(Self { key_pair })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_signature() {
        let message = b"hello world";
        let signature = Signer::new().unwrap();
        let signature_bytes = signature.sign(message);
        let public_key = signature.get_public_key();
        Signer::verify(message, &signature_bytes, &public_key).unwrap();
    }
}
