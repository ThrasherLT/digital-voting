//! A simple digital signature based on ed25519.

// TODO add examples when API is more stable.
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use thiserror::Error;

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

crate::crypto_key!(PublicKey, "Public key for digital signatures");
crate::crypto_key!(Signature, "Digital signature");
crate::crypto_key!(SecretKey, "Secret key for digital signatures");

impl SecretKey {
    /// Get secret key from pkcs8 bytes.
    ///
    /// # Returns
    ///
    /// The secret key.
    #[must_use]
    pub fn from_pkcs8(pkcs8: Vec<u8>) -> Self {
        Self(pkcs8)
    }
}

/// Associated function for verifying a signature.
///
/// # Arguments
///
/// * `message` - The message that was signed.
/// * `signature` - The signature to verify.
/// * `peer_public_key` - The public key of the peer that signed the message.
///
/// # Errors
pub fn verify(message: &[u8], signature: &Signature, peer_public_key: &PublicKey) -> Result<()> {
    let unparsed_public_key = UnparsedPublicKey::new(&signature::ED25519, peer_public_key);
    unparsed_public_key
        .verify(message, signature.as_ref())
        .map_err(|_| Error::SignatureInvalid)?;

    Ok(())
}

/// The signature struct containing the key pair.
/// Constructed only for signing, verification is done with an associated function.
pub struct Signer {
    /// Key pair for signing.
    key_pair: Ed25519KeyPair,
    /// Secret key for saving the key to a file and then creating a new signer from it once loaded.
    secret_key: SecretKey,
}

impl Signer {
    /// Create a new signature key pair (or a signer).
    ///
    /// # Returns
    ///
    /// The signer struct containing the keypair.
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
        Ok(Self {
            key_pair,
            secret_key: SecretKey::from_pkcs8(pkcs8.as_ref().to_vec()),
        })
    }

    /// Get existing keypair from pkcs8.
    ///
    /// # Returns
    ///
    /// The signer struct containing the keypair.
    ///
    /// # Errors
    ///
    /// If deriving the key pair from Pkcs8 fails.
    pub fn from_secret_key(secret_key: SecretKey) -> Result<Self> {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(secret_key.as_ref())
            .map_err(|_| Error::KeyPairGenerationFailed)?;
        Ok(Self {
            key_pair,
            secret_key,
        })
    }

    /// Get secret key encoded as pkcs8 document for storing the key.
    ///
    /// # Returns
    ///
    /// secret key in bytes encoded in pkcs8.
    #[must_use]
    pub fn get_secret_key(&self) -> &SecretKey {
        &self.secret_key
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
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature(self.key_pair.sign(message).as_ref().to_vec())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_signature() {
        let message = b"hello world";
        let signer_old = Signer::new().unwrap();

        // Test saving and loading of signer:
        let pkcs8 = signer_old.get_secret_key().as_ref();
        let signer = Signer::from_secret_key(SecretKey::from_pkcs8(pkcs8.to_owned())).unwrap();

        let signature_bytes = signer.sign(message);
        let public_key = signer.get_public_key();
        verify(message, &signature_bytes, &public_key).unwrap();
    }
}
