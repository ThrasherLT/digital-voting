/// Error type for Hash operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Invalid base64 encoding found while parsing. Perhaps there's an issue with the public key input?
    #[error("Invalid base64 {:?}", .0)]
    InvalidBase64(#[from] base64::DecodeError),
}

// A unified type for hashes. Used more as a wrapper around other hash types so that
// the application is less coupled to a specific algorithm. Also implements useful traits for
// serialization and display.
crate::crypto_key!(Hash, "Cryptographic hash");

impl Hash {
    /// Create `Hash` type from hash bytes.
    pub fn from<H>(hash: H) -> Self
    where
        H: AsRef<[u8]>,
    {
        Hash(hash.as_ref().to_vec())
    }
}
