//! Blind signature for allowing the user to send his personal data to a signer, get a signature
//! then use it as authentication in the blockchain without the signer being able to link the
//! user with the signature.
//! Also acts as a wrapper around the `blind_rsa_signatures` crate.

// TODO `blind_rsa_signatures` uses the rsa crate which is vulnerable to the Marvin attack, but
// the `blind_rsa_signatures` crate uses pss padding, so in theory the vulnerability should be mitigated,
// but proper tests should still be done.
// Also the `blind_rsa_signatures` crate has a message_randomizer feature which does not seem useful
// but should still be investigated if not using it opens us up to vulnerabilities.

use blind_rsa_signatures::{self, KeyPair, Options};
use thiserror::Error;

/// Errors that can occur when working with blind signatures.
#[derive(Error, Debug)]
pub enum Error {
    /// Error from the underlying blind signature crate.
    #[error("Blind signature failure {:?}", .0)]
    BlindSignature(#[from] blind_rsa_signatures::Error),
    /// Couldn't unblind the signature, because the unblinding secret was missing.
    /// This should never happen, since the unblinding secret is set during the blinding process.
    #[error("Unblinding secret is missing from unblinder")]
    UnblindingSecretMissing,
    /// Invalid base64 encoding found while parsing. Perhaps there's an issue with the public key input?
    #[error("Invalid base64 {:?}", .0)]
    InvalidBase64(#[from] base64::DecodeError),
}
type Result<T> = std::result::Result<T, Error>;

// The following few structs are thin wrappers around the types from the blind signature crate.
// So if in the future we needed to use a different crate, it wouldn't be tedious to swap out.

crate::crypto_key!(PublicKey, "Public key for blind signer");

impl TryFrom<PublicKey> for blind_rsa_signatures::PublicKey {
    type Error = Error;

    fn try_from(pk: PublicKey) -> Result<blind_rsa_signatures::PublicKey> {
        blind_rsa_signatures::PublicKey::from_der(&pk.0).map_err(Error::from)
    }
}

crate::crypto_key!(SecretKey, "Secret key for blind signer");

impl TryFrom<SecretKey> for blind_rsa_signatures::SecretKey {
    type Error = Error;

    fn try_from(pk: SecretKey) -> Result<blind_rsa_signatures::SecretKey> {
        blind_rsa_signatures::SecretKey::from_der(&pk.0).map_err(Error::from)
    }
}

crate::crypto_key!(BlindSignature, "Blind signature");

impl From<blind_rsa_signatures::BlindSignature> for BlindSignature {
    fn from(blind_signature: blind_rsa_signatures::BlindSignature) -> BlindSignature {
        BlindSignature(blind_signature.0)
    }
}

impl From<BlindSignature> for blind_rsa_signatures::BlindSignature {
    fn from(blind_signature: BlindSignature) -> blind_rsa_signatures::BlindSignature {
        blind_rsa_signatures::BlindSignature(blind_signature.0)
    }
}

crate::crypto_key!(Signature, "Unblinded signature");

impl From<Signature> for blind_rsa_signatures::Signature {
    fn from(signature: Signature) -> blind_rsa_signatures::Signature {
        blind_rsa_signatures::Signature(signature.0)
    }
}

impl From<blind_rsa_signatures::Signature> for Signature {
    fn from(signature: blind_rsa_signatures::Signature) -> Signature {
        Signature(signature.0)
    }
}

crate::crypto_key!(BlindedMessage, "Blinded message");

impl From<blind_rsa_signatures::BlindedMessage> for BlindedMessage {
    fn from(blinded_message: blind_rsa_signatures::BlindedMessage) -> BlindedMessage {
        BlindedMessage(blinded_message.0)
    }
}

crate::crypto_key!(UnblindingSecret, "Unblinding secret");

impl From<blind_rsa_signatures::Secret> for UnblindingSecret {
    fn from(secret: blind_rsa_signatures::Secret) -> Self {
        UnblindingSecret(secret.0)
    }
}

impl From<UnblindingSecret> for blind_rsa_signatures::Secret {
    fn from(secret: UnblindingSecret) -> Self {
        blind_rsa_signatures::Secret(secret.0)
    }
}

/// The signer for blindly signing messages.
#[derive(Debug, Clone)]
pub struct BlindSigner {
    /// The public key of the signer. This is sent to both the user and the verifier
    pk: blind_rsa_signatures::PublicKey,
    /// The secret key of the signer. This must never leave the server.
    sk: blind_rsa_signatures::SecretKey,
    /// Options for the blind signature scheme.
    options: Options,
}

impl BlindSigner {
    /// Create a new blind signer with it's own unique generated public key.
    ///
    /// # Returns
    ///
    /// A new blind signer.
    ///
    /// # Errors
    ///
    /// If the key generation fails, an error is returned.
    pub fn new() -> Result<Self> {
        // Since we're not supporting message randomizer, hardcoding default options for now.
        let options = Options::default();
        let rng = &mut rand::thread_rng();
        let kp = KeyPair::generate(rng, 2048)?;
        let (pk, sk) = (kp.pk, kp.sk);

        Ok(Self { pk, sk, options })
    }

    /// Create a new blind signer from the given secret and private keys.
    ///
    /// # Returns
    ///
    /// A new blind signer.
    ///
    /// # Errors
    ///
    /// If parsing fails for any of the two keys, an error is returned.
    pub fn new_from_keys(pk: PublicKey, sk: SecretKey) -> Result<Self> {
        let options = Options::default();

        Ok(Self {
            pk: pk.try_into()?,
            sk: sk.try_into()?,
            options,
        })
    }

    /// Get the public key of the signer in DER format.
    ///
    /// # Returns
    ///
    /// The public key of the signer in DER format.
    ///
    /// # Errors
    ///
    /// If the public key cannot be serialized to DER format, an error is returned.
    pub fn get_public_key(&self) -> Result<PublicKey> {
        Ok(PublicKey(self.pk.to_der()?))
    }

    /// Get the secret key of the signer in DER format.
    /// NOTE: This key is secret and thus must not be shared!
    ///
    /// # Returns
    ///
    /// The secret key of the signer in DER format.
    ///
    /// # Errors
    ///
    /// If the secret key cannot be serialized to DER format, an error is returned.
    pub fn get_secret_key(&self) -> Result<SecretKey> {
        Ok(SecretKey(self.sk.to_der()?))
    }

    /// Blindly sign a message.
    ///
    /// # Arguments
    ///
    /// * `blinded_msg` - The blinded message to sign.
    ///
    /// # Returns
    ///
    /// The blind signature.
    ///
    /// # Errors
    ///
    /// If the signing fails, an error is returned.
    pub fn bling_sign(&self, blinded_msg: &BlindedMessage) -> Result<BlindSignature> {
        let rng = &mut rand::thread_rng();
        let blind_sig = self.sk.blind_sign(rng, &blinded_msg.0, &self.options)?;

        Ok(blind_sig.into())
    }
}

/// The verifier for verifying blind signatures.
pub struct Verifier {
    /// The public key received from the signer.
    pk: blind_rsa_signatures::PublicKey,
    /// Options for the blind signature scheme. Must match the options used by the signer and user.
    options: Options,
}

impl Verifier {
    /// Create a new verifier with the public key of the signer.
    ///
    /// # Arguments
    ///
    /// * `pk` - The public key of the signer.
    ///
    /// # Returns
    ///
    /// A new verifier.
    ///
    /// # Errors
    ///
    /// If the public key is invalid, an error is returned.
    pub fn new(pk: PublicKey) -> Result<Self> {
        // Since we're not supporting message randomizer, hardcoding default options for now.
        let options = Options::default();

        Ok(Self {
            pk: pk.try_into()?,
            options,
        })
    }

    /// Verify a signature.
    ///
    /// # Arguments
    ///
    /// * `signature` - The signature to verify.
    ///
    /// # Returns
    ///
    /// If the signature is valid, `Ok(())` is returned.
    /// If the signature is invalid, an error is returned.
    ///
    /// # Errors
    ///
    /// If signature is forged or invalid.
    pub fn verify_signature(&self, signature: Signature, msg: &[u8]) -> Result<()> {
        let sig = blind_rsa_signatures::Signature::from(signature);
        sig.verify(&self.pk, None, msg, &self.options)?;

        Ok(())
    }
}

/// The blinder for blinding messages before sending them to the signer.
pub struct Blinder {
    /// The public key received from the signer.
    pk: blind_rsa_signatures::PublicKey,
    /// Options for the blind signature scheme. Must match the options used by the signer and verifier.
    options: Options,
}

impl Blinder {
    /// Create a new blinder with the public key of the signer.
    ///
    /// # Arguments
    ///
    /// * `pk` - The public key of the signer.
    ///
    /// # Returns
    ///
    /// A new blinder.
    ///
    /// # Errors
    ///
    /// If the public key is invalid, an error is returned.
    pub fn new(pk: PublicKey) -> Result<Self> {
        // Since we're not supporting message randomizer, hardcoding default options for now.
        let options = Options::default();

        Ok(Self {
            pk: pk.try_into()?,
            options,
        })
    }

    /// Blind a message.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to blind.
    ///
    /// # Returns
    ///
    /// The blinded message and the unblinder.
    ///
    /// # Errors
    ///
    /// If the blinding fails, an error is returned.
    pub fn blind(&self, msg: &[u8]) -> Result<(BlindedMessage, Unblinder)> {
        let rng = &mut rand::thread_rng();
        let blinding_result = self.pk.blind(rng, msg, false, &self.options)?;

        let unblinder = Unblinder {
            pk: self.pk.clone(),
            options: self.options.clone(),
            unblinding_secret: blinding_result.secret,
        };

        Ok((blinding_result.blind_msg.into(), unblinder))
    }
}

/// Struct used for unblinding a signature with the information gathered during
/// the blinding process.
pub struct Unblinder {
    /// The public key received from the signer.
    pk: blind_rsa_signatures::PublicKey,
    /// Options for the blind signature scheme. Must match the options used by the signer and verifier.
    options: Options,
    /// The unblinding secret used to unblind the signature. This must never leave the user.
    unblinding_secret: blind_rsa_signatures::Secret,
}

impl Unblinder {
    /// Unblind a signature.
    ///
    /// # Arguments
    ///
    /// * `blind_signature` - The blind signature to unblind.
    /// * `msg` - The original secret message before it was blinded.
    ///
    /// # Returns
    ///
    /// The unblinded signature.
    ///
    /// # Errors
    ///
    /// If the unblinding secret is missing, an error is returned.
    /// If the unblinding fails, an error is returned.
    pub fn unblind_signature(
        &self,
        blind_signature: BlindSignature,
        msg: &[u8],
    ) -> Result<Signature> {
        if self.unblinding_secret.0.is_empty() {
            return Err(Error::UnblindingSecretMissing);
        }

        let signature = self.pk.finalize(
            &blind_signature.into(),
            &self.unblinding_secret,
            None,
            msg,
            &self.options,
        )?;

        Ok(signature.into())
    }

    /// Get the blinding secret for this `Unblinder` instance.
    /// Primarily used to store the `Unblinder` and recreate it when loaded.
    ///
    /// # Returns
    ///
    /// The unblinding secret for this unblinder instance.
    ///
    #[must_use]
    pub fn get_unblinding_secret(&self) -> UnblindingSecret {
        self.unblinding_secret.clone().into()
    }

    /// Recreate an `Unblinder` from a `Signer` public key and an unblinding secret.
    ///
    /// # Arguments
    ///
    /// * `pk` - The public key of the `Blinder`.
    /// * `unblinding_secret` - The unblinding secret from which the new `Unblinder` will be constructed.
    ///
    /// # Returns
    ///
    /// A recreated instance of an `Unblinder`.
    ///
    /// # Errors
    ///
    /// If public key conversion fails.
    pub fn from_pk_and_secret(pk: PublicKey, unblinding_secret: UnblindingSecret) -> Result<Self> {
        Ok(Self {
            pk: pk.try_into()?,
            options: Options::default(),
            unblinding_secret: unblinding_secret.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_blind_signature() {
        // Signer:
        let blind_signer = BlindSigner::new().unwrap();

        // Testing saving and loading of blind_signer:
        let pk = blind_signer.get_public_key().unwrap();
        let sk = blind_signer.get_secret_key().unwrap();

        let blind_signer = BlindSigner::new_from_keys(pk.clone(), sk.clone()).unwrap();

        // User:
        let msg = b"secret_message";
        let blinder = Blinder::new(blind_signer.get_public_key().unwrap()).unwrap();
        let (blind_msg, unblinder) = blinder.blind(msg).unwrap();
        assert_ne!(msg, blind_msg.0.as_slice());

        // Signer:
        let blind_signature = blind_signer.bling_sign(&blind_msg).unwrap();

        // User:
        let unblinding_secret = unblinder.get_unblinding_secret();
        let unblinder = Unblinder::from_pk_and_secret(pk, unblinding_secret).unwrap();
        let signature = unblinder
            .unblind_signature(blind_signature.clone(), msg)
            .unwrap();
        assert_ne!(blind_signature.0, signature.0);

        // Verifier:
        let verifier = Verifier::new(blind_signer.get_public_key().unwrap()).unwrap();
        verifier.verify_signature(signature.clone(), msg).unwrap();
        // Asserting that the blind signature can't be used to verify the original message.
        // Since then the signer can link the blind signature to the original message.
        assert!(verifier
            .verify_signature(Signature(blind_signature.0), msg)
            .is_err());
        // Same for the blind message and the unblinded signature
        assert!(verifier.verify_signature(signature, &blind_msg.0).is_err());
    }
}
