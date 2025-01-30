use std::path::Path;

use redb::{Database, TableDefinition};

const TABLE: TableDefinition<u128, Vec<u8>> = TableDefinition::new("blockchain");

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
pub struct Storage {
    db: Database,
}

// TODO Compacting?

impl Storage {
    /// If the file provided is an existing storage file, that file will be opened,
    /// otherwise a new storage will be created.
    pub fn new(storage_file_path: &Path) -> Result<Self> {
        // If the database already exists, it will be opened instead.
        Ok(Self {
            db: Database::create(storage_file_path)?,
        })
    }

    // TODO Make sure this exact error is always returned when the database isn't initialized.
    /// Open storage from existing database file
    ///
    /// # Errors
    ///
    /// `DoesNotExist`, if the database does not exist yet.
    /// Or if opening the database failed.
    pub fn open(storage_file_path: &Path) -> Result<Self> {
        let db = match Database::open(storage_file_path) {
            Ok(db) => db,
            Err(redb::DatabaseError::Storage(redb::StorageError::Io(e))) => match e.kind() {
                std::io::ErrorKind::InvalidData => return Err(Error::DoesNotExist),
                _ => Err(redb::StorageError::Io(e))?,
            },
            e => e?,
        };

        Ok(Self { db })
    }

    /// Write a key:value pair into the storage.
    pub fn write(&self, key: u128, value: Vec<u8>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Read a value, which corresponds to the provided key, from the storage.
    pub fn read(&self, key: u128) -> Result<Option<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;
        if let Some(value) = table.get(key)? {
            Ok(Some(value.value()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_storage() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let storage = Storage::new(temp_file.path()).unwrap();
        let expected_values = vec![
            (1, vec![1u8, 2u8, 3u8, 4u8]),
            (3, vec![2u8, 1u8, 3u8, 4u8]),
            (0, vec![1u8, 2u8, 3u8, 4u8]),
        ];

        for (key, value) in &expected_values {
            storage.write(*key, value.clone()).unwrap();
        }

        assert_eq!(storage.read(2).unwrap(), None);
        for (key, value) in expected_values {
            assert_eq!(storage.read(key).unwrap(), Some(value));
        }
    }

    #[test]
    fn test_storage_not_initialized() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let storage = Storage::open(temp_file.path());
        assert!(matches!(storage, Err(Error::DoesNotExist)));
    }
}
