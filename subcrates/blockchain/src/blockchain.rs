use std::path::Path;

use crypto::hash_storage::Hash;
use digest::Digest;
use process_io::storage::{self, Storage};
use protocol::timestamp::Timestamp;

use crate::block::{self, Block};

/// Type for the height (number of blocks) of the blockchain.
pub type Height = u64;

/// The storage table name for the blockchain.
pub const BLOCKCHAIN_TABLE: &str = "blockchain";

/// Error type for block operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Storage database issue.
    #[error("No block found for height key {}", .0)]
    WrongKey(Height),
    /// Failed to save or load block from storage.
    #[error(transparent)]
    Storage(#[from] storage::Error),
    /// Failed to save or load block from storage.
    #[error(transparent)]
    Block(#[from] block::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Structure of all the data required to run the blockchain.
pub struct Blockchain<D>
where
    D: Digest,
{
    /// How many blocks are currently in the blockchain.
    block_count: Height,
    /// Hash of the last block added to the blockchain.
    /// Held in the structure for speed and convenience.
    last_hash: Hash,
    /// Timestamp of the last block added to the blockchain.
    /// Held in the structure for speed and convenience.
    _last_timestamp: Timestamp,
    /// Handle to the storage, which is used for saving and loading individual
    /// blocks to and from storage.
    // TODO Not sure of the implications of leaving lifetime to static here:
    storage: Storage<'static, u64, Vec<u8>>,
    /// Phantom data marker which holds the type of the hashing algorithm that is used
    /// for this blockchain.
    _marker: std::marker::PhantomData<D>,
}

impl<D> Blockchain<D>
where
    D: Digest,
{
    /// Create a new blockchain from a path to the file in which the blockchain will be
    /// stored.
    /// If a database at that path already exists, it will be opened instead of created.
    ///
    /// # Errors
    ///
    /// If creating storage or saving genesis block fails.
    pub fn new(database_file_path: &Path) -> Result<Self> {
        match Storage::open(database_file_path, BLOCKCHAIN_TABLE) {
            Ok(storage) => {
                let block_count = storage.len()?;
                let last_block = Block::load(block_count - 1, &storage)?;

                Ok(Self {
                    block_count,
                    last_hash: last_block.prev_block_hash,
                    _last_timestamp: last_block.timestamp,
                    storage,
                    _marker: std::marker::PhantomData,
                })
            }
            Err(storage::Error::DoesNotExist) => {
                let storage = Storage::new(database_file_path, BLOCKCHAIN_TABLE)?;

                Ok(Self {
                    block_count: 0,
                    last_hash: Hash::zero(),
                    _last_timestamp: chrono::Utc::now(),
                    storage,
                    _marker: std::marker::PhantomData,
                })
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Add a new block to the blockchain.
    ///
    /// # Errors
    ///
    /// If Saving block to storage fails.
    pub fn add_block(&mut self, block: &Block) -> Result<()> {
        self.last_hash = block.calculate_hash::<D>();
        self.block_count += 1;
        block.save(self.block_count, &self.storage)?;

        Ok(())
    }

    /// Load a block from the blockchain.
    ///
    /// # Errors
    ///
    /// If loading block from storage fails.
    pub fn get_block(&self, height: Height) -> Result<Block> {
        let block = Block::load(height, &self.storage)?;

        Ok(block)
    }

    /// Get the number of currently stored blocks in the blockchain.
    #[must_use]
    pub fn len(&self) -> Height {
        self.block_count
    }

    /// Return `true`, if no blocks had been written to the blockchain yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.block_count == 0
    }
}
