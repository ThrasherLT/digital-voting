//! Module to construct, use and track commitment schemes.

use serde::{Deserialize, Serialize};

/// The actual commitment value wrapped in a struct for convenience and with Serde implementations.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commitment(Vec<u8>);

/// A type alias for cleaning up boiler plate code regarding the combine and hash (or commitment) function.
type CommitmentFn<V, N, H> = Box<dyn Fn(&V, &N) -> H>;

/// A commitment scheme based on a the provided commitment function.
pub struct HashCommitmentScheme<V, N, H> {
    commitment_fn: CommitmentFn<V, N, H>,
}

impl<V, N, H> HashCommitmentScheme<V, N, H>
where
    H: AsRef<[u8]>,
{
    /// Create a new commitment scheme from the provided commitment function.
    pub fn new(hash_fn: CommitmentFn<V, N, H>) -> Self {
        Self {
            commitment_fn: hash_fn,
        }
    }

    /// Commit to a value with a nonce.
    pub fn commit(&self, value: &V, nonce: &N) -> Commitment {
        let hash = (self.commitment_fn)(value, nonce);
        Commitment(hash.as_ref().to_vec())
    }

    /// Verify a commitment.
    pub fn verify(&self, value: &V, nonce: &N, commitment: &Commitment) -> bool {
        let hash = (self.commitment_fn)(value, nonce);
        hash.as_ref() == commitment.0.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO add tests with more different data types.

    // A mock hasher to avoid having to link full blown hashers to this crate
    // just for testing.
    fn mock_hash(preimages: [u64; 2]) -> [u8; 8] {
        (preimages[0] ^ preimages[1]).to_le_bytes()
    }

    #[test]
    fn test_commitment() {
        let hash_fn = Box::new(|value: &u64, nonce: &u64| mock_hash([*value, *nonce]));

        let commitment_scheme = HashCommitmentScheme::new(hash_fn);

        let value = 42;
        let nonce = 0;
        let commitment = commitment_scheme.commit(&value, &nonce);

        assert!(commitment_scheme.verify(&value, &nonce, &commitment));
    }
}