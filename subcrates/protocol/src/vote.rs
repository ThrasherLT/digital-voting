//! This is the module in which the vote protocol is defined.
//! Each vote stores all the data required to validate it.

use crypto::{
    self,
    signature::{blind_sign, digital_sign},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::candidate_id::CandidateId;
use crate::timestamp::{Limits as TimestampLimits, Timestamp};

/// Errors that can occur when working with election votes.
#[derive(Error, Debug)]
pub enum Error {
    /// Access token is invalid.
    #[error("Failed verify access token: {}", .0)]
    AccessTokenVerification(#[from] blind_sign::Error),
    /// Message signature is invalid.
    #[error("Failed verify signature: {}", .0)]
    SignatureVerification(#[from] digital_sign::Error),
    /// Something went wrong while serializing or deserializing the timestamp.
    #[error("Failed to serialize or deserialize timestamp: {}", .0)]
    TimestampSerialization(#[from] bincode::Error),
    /// The timestamp is invalid.
    #[error("Timestamp is invalid: {}", .0)]
    InvalidTimestmap(Timestamp),
}
type Result<T> = std::result::Result<T, Error>;

/// Structure of a vote in the blockchain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    /// Digital signature public key of the blockchain user who cast the vote.
    public_key: digital_sign::PublicKey,
    /// The candidate for whom the vote is being cast.
    candidate: CandidateId,
    /// Access token issued by the election authority.
    /// It is a blind signature used to sign the `public_key` of the user.
    /// The public key of this signature is the public key of the election authority.
    /// Each access token on the blockchain must be unique.
    timestamp: Timestamp,
    access_token: blind_sign::Signature,
    /// Digital signature corresponding to the `public_key`.
    /// It signs all previous fields.
    signature: digital_sign::Signature,
}

impl Vote {
    /// Create new Vote to be sent to the blockchain.
    ///
    /// # Arguments
    ///
    /// - `signer` - Digital signer used to sign messages with the blockchain user's public key.
    /// - `candidate` - The candidate for whom the vote is being cast.
    /// - `access_token` - Access token issued by the election authority, needed to write to the blockchain.
    ///
    /// # Returns
    ///
    /// A new Vote instance.
    ///
    /// # Errors
    ///
    /// If serializing the struct to bytes for signing fails.
    pub fn new(
        signer: &digital_sign::Signer,
        candidate: CandidateId,
        timestamp: Timestamp,
        access_token: &blind_sign::Signature,
    ) -> Result<Self> {
        let public_key = signer.get_public_key();
        let to_sign = Self::signed_bytes(&public_key, &candidate, &timestamp, access_token)?;

        Ok(Self {
            public_key,
            candidate,
            timestamp,
            access_token: access_token.clone(),
            signature: signer.sign(&to_sign),
        })
    }

    #[must_use]
    pub fn get_candidate(&self) -> &CandidateId {
        &self.candidate
    }

    /// Create new Vote to be sent to the blockchain.
    ///
    /// # Arguments
    ///
    /// - `signer` - Digital signer used to sign messages with the blockchain user's public key.
    /// - `candidate` - The candidate for whom the vote is being cast.
    /// - `access_token` - Access token issued by the election authority, needed to write to the blockchain.
    ///
    /// # Returns
    ///
    /// A new Vote instance.
    fn signed_bytes(
        public_key: &digital_sign::PublicKey,
        candidate: &CandidateId,
        timestamp: &Timestamp,
        access_token: &blind_sign::Signature,
    ) -> Result<Vec<u8>> {
        let mut to_sign =
            Vec::with_capacity(public_key.len() + candidate.as_ref().len() + access_token.len());
        to_sign.extend_from_slice(public_key.as_ref());
        to_sign.extend_from_slice(candidate.as_ref());
        to_sign.extend_from_slice(access_token.as_ref());
        to_sign.append(&mut bincode::serialize(&timestamp)?);

        Ok(to_sign)
    }

    /// Verify an isntance of a vote.
    ///
    /// # Arguments
    ///
    /// - `access_token_verifyer` - Verifyer of the blind signature of the election authority.
    ///
    /// # Errors
    ///
    /// If the vote is invalid or corrupted.
    pub fn verify(
        &self,
        access_token_verifyer: &blind_sign::Verifier,
        timestamp_limits: &TimestampLimits,
    ) -> Result<()> {
        if !timestamp_limits.verify(self.timestamp) {
            return Err(Error::InvalidTimestmap(self.timestamp));
        }
        access_token_verifyer.verify_signature(self.access_token.clone(), &self.public_key)?;
        let signed_bytes = Self::signed_bytes(
            &self.public_key,
            &self.candidate,
            &self.timestamp,
            &self.access_token,
        )?;
        Ok(digital_sign::verify(
            &signed_bytes,
            &self.signature,
            &self.public_key,
        )?)
    }
}

impl std::fmt::Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} voted for candidate {} on {}",
            self.public_key,
            self.candidate,
            self.timestamp.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;

    fn generate_vote_for_testing(
        timestamp: Timestamp,
        candidate: CandidateId,
    ) -> (Vote, blind_sign::PublicKey) {
        let blind_signer = blind_sign::BlindSigner::new().unwrap();
        let authority_pubkey = blind_signer.get_public_key().unwrap();
        let digital_signer = digital_sign::Signer::new().unwrap();
        let msg = digital_signer.get_public_key();
        let blinder = blind_sign::Blinder::new(blind_signer.get_public_key().unwrap()).unwrap();
        let (blind_msg, unblinder) = blinder.blind(&msg).unwrap();
        let blind_signature = blind_signer.bling_sign(&blind_msg).unwrap();
        let access_token = unblinder
            .unblind_signature(blind_signature.clone(), &msg)
            .unwrap();
        let vote = Vote::new(&digital_signer, candidate, timestamp, &access_token).unwrap();

        (vote, authority_pubkey)
    }

    // TODO Not sure if it's a good idea to couple this test to crypto subcrate.
    #[wasm_bindgen_test]
    #[test]
    fn test_vote() {
        let timestamp = chrono::Utc::now();
        let (vote, access_token) = generate_vote_for_testing(timestamp, CandidateId::new(1));

        let verifier = blind_sign::Verifier::new(access_token).unwrap();
        let timestamp_limits = TimestampLimits::new(
            timestamp - std::time::Duration::from_secs(1),
            timestamp + std::time::Duration::from_secs(1),
        )
        .unwrap();
        vote.verify(&verifier, &timestamp_limits).unwrap();
    }
}
