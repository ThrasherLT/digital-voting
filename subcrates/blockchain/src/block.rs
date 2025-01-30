use crypto::hash_storage::Hash;
use digest::Digest;
use protocol::timestamp::{self, Timestamp};

use crate::{
    blockchain::Height,
    storage::{self, Storage},
};

/// Error type for block operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Storage database issue.
    #[error("No block found for height key {}", .0)]
    WrongKey(u128),
    /// Failed to save or load block from storage.
    #[error("Block storage failed {}", .0)]
    Storage(#[from] storage::Error),
    /// Failed to save or load block from storage.
    #[error("Block Binary serialization or deserialization failed {}", .0)]
    BinarySerialization(#[from] bincode::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Datastructure of a single block of a blockchain.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Block {
    /// Because data is stored as binary, this is needed for parsing to know what type
    /// of data should be parsed.
    pub value_type_id: u16,
    /// The actual data that is stored, but in bytes.
    pub value: Vec<u8>,
    /// The timestamp at which this struct had been created.
    pub timestamp: Timestamp,
    /// Hash of the previous block in the blockchain.
    pub prev_block_hash: Hash,
}

impl Block {
    /// Create a new block containing a value.
    // TODO figure out value type ID translation.
    #[must_use]
    pub fn new(value_type_id: u16, value: Vec<u8>, prev_block_hash: Hash) -> Self {
        let timestamp = chrono::Utc::now();

        Self {
            value_type_id,
            value,
            timestamp,
            prev_block_hash,
        }
    }

    /// Calculate the hash of this block.
    #[must_use]
    pub fn calculate_hash<D>(&self) -> Hash
    where
        D: Digest,
    {
        let mut hasher = D::new();

        hasher.update(self.value_type_id.to_le_bytes());
        hasher.update(&self.value);
        hasher.update(self.timestamp.timestamp().to_le_bytes());
        hasher.update(&self.prev_block_hash);

        Hash::from(hasher.finalize())
    }

    /// Verify this block against the previous block's hash and timestamp limits.
    #[must_use]
    pub fn verify(&self, previous_hash: &Hash, timestamp_limits: &timestamp::Limits) -> bool {
        timestamp_limits.verify(self.timestamp) && previous_hash == &self.prev_block_hash
    }

    /// Save this block to persistent storage.
    ///
    /// # Errors
    ///
    /// If serialization or writing to storage fails.
    pub fn save(&self, height: Height, storage: &Storage) -> Result<()> {
        storage.write(height, bincode::serialize(self)?)?;

        Ok(())
    }

    /// Load a block from persistent storage.
    ///
    /// # Errors
    ///
    /// If reading from storage fails, deserialization fails, or key is invalid
    pub fn load(height: Height, storage: &Storage) -> Result<Self> {
        let block_bytes = storage.read(height)?.ok_or(Error::WrongKey(height))?;

        Ok(bincode::deserialize(&block_bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;

    use super::*;

    #[test]
    fn test_block() {
        use blake3::Hasher;
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let storage = Storage::new(temp_file.path()).unwrap();

        let values = vec![
            (2u16, vec![1u8, 2u8, 3u8, 4u8]),
            (2u16, vec![1u8, 2u8, 3u8, 4u8]),
        ];
        let max_timestmap = Utc::now() + Duration::from_secs(10);

        let mut prev_hash = Hash::from(0u8.to_le_bytes());
        let mut prev_timestamp = Utc::now();

        for (i, (value_type_id, value)) in values.iter().enumerate() {
            let block = Block::new(*value_type_id, value.clone(), prev_hash);
            prev_hash = block.calculate_hash::<Hasher>();
            block.save(i.try_into().unwrap(), &storage).unwrap();
        }

        prev_hash = Hash::from(0u8.to_le_bytes());
        for i in 0..2 {
            let block = Block::load(i, &storage).unwrap();
            let timestamp_limits = timestamp::Limits::new(prev_timestamp, max_timestmap).unwrap();
            prev_timestamp = block.timestamp;

            assert!(block.verify(&prev_hash, &timestamp_limits));

            prev_hash = block.calculate_hash::<Hasher>();
        }
    }
}
