use std::fmt::Display;

use ring::digest;
use thiserror::Error;

use crate::Timestamp;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Blockchain hash integrity error {} != {}", .0, .1)]
    BlockchainHashIntegrity(Hash, Hash),
    #[error("Array length mismatch error: {}", .0)]
    ArrayLenMismatch(#[from] std::array::TryFromSliceError),
    #[error("Binary serialization error: {}", .0)]
    BinSerialization(#[from] bincode::Error),
    #[error("File IO error: {}", .0)]
    FileIO(#[from] std::io::Error),
    #[error("Unknown blockchain error")]
    Unknown,
}

pub trait BlockValue: for<'de> serde::Deserialize<'de> + serde::Serialize + Display {}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Blockchain<T> {
    blocks: Vec<Block<T>>,
}

impl<T: BlockValue> Blockchain<T> {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn add_block(&mut self, block_value: Vec<T>) -> Result<(), Error> {
        let prev_block_hash = match self.blocks.last() {
            Some(block) => block.get_hash()?,
            None => Hash([0; 32]),
        };
        let block = Block::new(block_value, prev_block_hash);
        self.blocks.push(block);
        Ok(())
    }

    pub fn iter(&self) -> ChainIter<T> {
        ChainIter {
            container: self,
            index: 0,
        }
    }

    pub fn validate_hashes(&self) -> Result<(), Error> {
        let mut prev_block_hash = Hash([0; 32]);
        for block in &self.blocks {
            let block_hash = block.get_hash()?;
            if block.prev_block_hash != prev_block_hash {
                return Err(Error::BlockchainHashIntegrity(
                    block.prev_block_hash.clone(),
                    prev_block_hash.clone(),
                ));
            }
            prev_block_hash = block_hash;
        }
        Ok(())
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Error> {
        let file = std::fs::File::create(filename)?;
        bincode::serialize_into(file, &self)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Error> {
        let file = std::fs::File::open(filename)?;
        let blockchain: Self = bincode::deserialize_from(file)?;
        Ok(blockchain)
    }
}

pub struct ChainIter<'a, T> {
    container: &'a Blockchain<T>,
    index: usize,
}

impl<'a, T> Iterator for ChainIter<'a, T> {
    type Item = &'a Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.container.blocks.len() {
            return None;
        }
        let block = self.container.blocks.get(self.index)?;
        self.index += 1;
        Some(&block.values)
    }
}

impl<T: BlockValue> Display for Blockchain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for block in &self.blocks {
            writeln!(f, "{block}")?;
        }
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Block<T> {
    values: Vec<T>,
    timestamp: Timestamp,
    prev_block_hash: Hash,
}

impl<T: BlockValue> Block<T> {
    fn new(block_value: Vec<T>, prev_block_hash: Hash) -> Self {
        // TODO not sure if now is the correct timestamp to use here.
        let timestamp = chrono::Utc::now();

        Self {
            values: block_value,
            timestamp,
            prev_block_hash,
        }
    }

    fn get_hash(&self) -> Result<Hash, Error> {
        let bytes = bincode::serialize(&self)?;
        let hash = digest::digest(&digest::SHA256, &bytes).try_into()?;
        Ok(hash)
    }
}

impl<T: BlockValue> Display for Block<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Block timestamp: {}", self.timestamp)?;
        writeln!(
            f,
            "Block hash: {}",
            self.get_hash().map_err(|_| std::fmt::Error)?
        )?;
        writeln!(f, "Previous block hash: {}", self.prev_block_hash)?;
        for block_value in &self.values {
            writeln!(f, "{block_value}")?;
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct Hash([u8; 32]);

impl TryFrom<digest::Digest> for Hash {
    type Error = Error;

    fn try_from(value: digest::Digest) -> Result<Self, Self::Error> {
        let bytes = value.as_ref().try_into()?;
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
