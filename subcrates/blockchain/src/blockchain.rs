use std::path::Path;

use crypto::hash_storage::Hash;
use digest::Digest;
use protocol::timestamp::Timestamp;

use crate::{
    block::{self, Block},
    storage::{self, Storage},
};

pub type Height = u128;

/// Error type for block operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Storage database issue.
    #[error("No block found for height key {}", .0)]
    WrongKey(u128),
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
    block_count: u128,
    /// Hash of the last block added to the blockchain.
    /// Held in the structure for speed and convenience.
    last_hash: Hash,
    /// Timestamp of the last block added to the blockchain.
    /// Held in the structure for speed and convenience.
    _last_timestamp: Timestamp,
    /// Handle to the storage, which is used for saving and loading individual
    /// blocks to and from storage.
    storage: Storage,
    /// Phantom data marker which holds the type of the hashing algorithm that is used
    /// for this blockchain.
    _marker: std::marker::PhantomData<D>,
}

impl<D> Blockchain<D>
where
    D: Digest,
{
    /// Create a new blockchain from a path to the file in which the blockchain will be
    /// stored and a genesis block.
    ///
    /// # Errors
    ///
    /// If Saving creating storage or saving genesis block fails.
    pub fn new(database_file_path: &Path, genesis: Block) -> Result<Self> {
        let storage = Storage::new(database_file_path)?;

        genesis.save(0, &storage)?;

        Ok(Self {
            block_count: 1,
            last_hash: genesis.prev_block_hash,
            _last_timestamp: genesis.timestamp,
            storage,
            _marker: std::marker::PhantomData,
        })
    }

    // TODO Load?

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
}
