//! Code for storage on the host machine.

use std::path::Path;

use redb::{Database, ReadableTableMetadata, TableDefinition};

/// Error type for blockchain storage operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Storage database issue.
    #[error("Database error: {}", .0)]
    Database(#[from] redb::DatabaseError),
    /// Storage transaction failed.
    #[error("Database error: {}", .0)]
    Transaction(#[from] redb::TransactionError),
    /// Storage table issue.
    #[error("Table error: {}", .0)]
    Table(#[from] redb::TableError),
    /// Storage table issue.
    #[error("Commit error: {}", .0)]
    Commit(#[from] redb::CommitError),
    /// Storage issue.
    #[error("Storage error: {}", .0)]
    Storage(#[from] redb::StorageError),
    /// Storage does not exist.
    #[error("Storage doesn't exist")]
    DoesNotExist,
}
type Result<T> = std::result::Result<T, Error>;

/// Handle for the storage metadata.
pub struct Storage<'a, K, V>
where
    K: redb::Key + 'static + std::borrow::Borrow<<K as redb::Value>::SelfType<'a>>,
    V: redb::Value + 'static + std::borrow::Borrow<<V as redb::Value>::SelfType<'a>>,
{
    db: Database,
    table: TableDefinition<'a, K, V>,
}

// TODO Compacting?

impl<'a, K, V> Storage<'a, K, V>
where
    K: redb::Key + std::borrow::Borrow<<K as redb::Value>::SelfType<'a>>,
    V: redb::Value + 'static + std::borrow::Borrow<<V as redb::Value>::SelfType<'a>>,
{
    /// If the file provided is an existing storage file, that file will be opened,
    /// otherwise a new storage will be created.
    ///
    /// # Errors
    ///
    /// If creating the database fails.
    pub fn new(storage_file_path: &Path, table: &'a str) -> Result<Self> {
        // If the database already exists, it will be opened instead.
        Ok(Self {
            db: Database::create(storage_file_path)?,
            table: TableDefinition::new(table),
        })
    }

    /// Open storage from existing database file
    ///
    /// # Errors
    ///
    /// `DoesNotExist`, if the database does not exist yet.
    /// Or if opening the database failed.
    pub fn open(storage_file_path: &Path, table: &'a str) -> Result<Self> {
        let db = match Database::open(storage_file_path) {
            Ok(db) => db,
            Err(redb::DatabaseError::Storage(redb::StorageError::Io(e))) => match e.kind() {
                std::io::ErrorKind::InvalidData => return Err(Error::DoesNotExist),
                _ => Err(redb::StorageError::Io(e))?,
            },
            e => e?,
        };

        Ok(Self {
            db,
            table: TableDefinition::new(table),
        })
    }

    /// Write a key:value pair into the storage.
    ///
    /// # Errors
    ///
    /// If Writingto the database fails.
    pub fn write(&self, key: K, value: V) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.table)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Read a value, which corresponds to the provided key, from the storage.
    ///
    /// # Errors
    ///
    /// If reading from the database fails.
    pub fn read(&self, key: K) -> Result<Option<V>>
    where
        V: for<'b> From<<V as redb::Value>::SelfType<'b>>,
    {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(self.table)?;
        if let Some(value) = table.get(key)? {
            let extracted: <V as redb::Value>::SelfType<'_> = value.value();
            Ok(Some(V::from(extracted)))
        } else {
            Ok(None)
        }
    }

    /// Remove a key-value pair from the storage database.
    ///
    /// # Errors
    ///
    /// If writing to the database fails.
    pub fn remove(&self, key: K) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(self.table)?;
            table.remove(key)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get the number of blocks in the blockchain.
    ///
    /// # Errors
    ///
    /// If we fail to read from the database.
    pub fn len(&self) -> Result<u64> {
        let read_txn = self.db.begin_read()?;
        // If nothing had been written to the table before, it will not had been created yet
        // and will return error.
        // TODO Make sure that we're not missing any edge cases here.
        match read_txn.open_table(self.table) {
            Ok(table) => {
                Ok(table.len()?)

            },
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Get boolean indicating that the storage is empty, if true.
    ///
    /// # Errors
    ///
    /// If we fail to read from the database.
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    pub const BLOCKCHAIN_TABLE: &str = "blockchain";

    #[test]
    fn test_storage() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let storage = Storage::new(temp_file.path(), BLOCKCHAIN_TABLE).unwrap();
        let expected_values = vec![
            (1, vec![1u8, 2u8, 3u8, 4u8]),
            (3, vec![2u8, 1u8, 3u8, 4u8]),
            (0, vec![1u8, 2u8, 3u8, 4u8]),
        ];

        assert_eq!(storage.len().unwrap(), 0);
        for (key, value) in &expected_values {
            storage.write(*key, value.clone()).unwrap();
        }
        assert_eq!(storage.len().unwrap(), 3);

        assert_eq!(storage.read(2).unwrap(), None);
        for (key, value) in expected_values {
            assert_eq!(storage.read(key).unwrap(), Some(value));
        }

        storage.remove(0).unwrap();
        assert_eq!(storage.len().unwrap(), 2);
        assert!(storage.read(0).unwrap().is_none());
    }

    #[test]
    fn test_storage_not_initialized() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let storage = Storage::<u64, Vec<u8>>::open(temp_file.path(), BLOCKCHAIN_TABLE);
        assert!(matches!(storage, Err(Error::DoesNotExist)));
    }
}
