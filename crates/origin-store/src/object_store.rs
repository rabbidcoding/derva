// INVARIANT: Append-only object store indexed by ORID; 100% read verification against corruption.
// KPI: Bitflip detected 100%; Write-after-read identity 100%; crash during append never publishes partial object.

use origin_core::{ObjectKind, ORID};
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum StoreError {
    ObjectNotFound(ORID),
    CorruptedObject { expected: ORID, found: ORID },
    AlreadyExists(ORID),
    IoError(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::ObjectNotFound(id) => write!(f, "Object not found in store: {}", id),
            StoreError::CorruptedObject { expected, found } => write!(
                f,
                "Store corruption detected! Expected ORID {}, found {}",
                expected, found
            ),
            StoreError::AlreadyExists(id) => write!(f, "Object already exists in store: {}", id),
            StoreError::IoError(e) => write!(f, "Store I/O error: {}", e),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::IoError(e)
    }
}

pub struct ObjectStore {
    root_dir: PathBuf,
}

impl ObjectStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root = root_dir.as_ref().to_path_buf();
        let objects_dir = root.join("objects");
        let tmp_dir = root.join("tmp");

        fs::create_dir_all(&objects_dir)?;
        fs::create_dir_all(&tmp_dir)?;

        Ok(Self { root_dir: root })
    }

    fn object_path(&self, id: &ORID) -> PathBuf {
        let hex_hash = format!("{:02x}{:02x}", id.hash[0], id.hash[1]);
        self.root_dir
            .join("objects")
            .join(hex_hash)
            .join(format!("{}.bin", id))
    }

    fn tmp_path(&self, id: &ORID) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        self.root_dir
            .join("tmp")
            .join(format!("{}_{}.tmp", id, nonce))
    }

    /// Appends a new canonical object to the store atomically.
    /// If the object already exists, verifies its content identity.
    pub fn put(&self, kind: ObjectKind, bytes: &[u8]) -> Result<ORID, StoreError> {
        let id = ORID::compute(kind, bytes);
        let final_path = self.object_path(&id);

        if final_path.exists() {
            // Object already exists: verify content identity
            let existing_bytes = self.get(id)?;
            if existing_bytes == bytes {
                return Ok(id);
            } else {
                return Err(StoreError::CorruptedObject {
                    expected: id,
                    found: ORID::compute(kind, &existing_bytes),
                });
            }
        }

        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temporary file first for atomic append durability
        let tmp_path = self.tmp_path(&id);
        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }

        // Atomic rename guarantees no partial write is published
        fs::rename(&tmp_path, &final_path)?;

        Ok(id)
    }

    /// Reads an object by ORID and verifies SHA-256 hash match on read.
    pub fn get(&self, id: ORID) -> Result<Vec<u8>, StoreError> {
        let final_path = self.object_path(&id);
        if !final_path.exists() {
            return Err(StoreError::ObjectNotFound(id));
        }

        let mut file = File::open(&final_path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let computed_id = ORID::compute(id.kind, &bytes);
        if computed_id != id {
            return Err(StoreError::CorruptedObject {
                expected: id,
                found: computed_id,
            });
        }

        Ok(bytes)
    }

    /// Checks if an object exists in the store without reading full contents.
    pub fn contains(&self, id: &ORID) -> bool {
        self.object_path(id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_store_write_after_read_identity() {
        let temp_dir = std::env::temp_dir().join("origin_test_objstore_identity");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = ObjectStore::new(&temp_dir).unwrap();
        let payload = b"canonical_object_test_payload";
        let id = store.put(ObjectKind::Claim, payload).unwrap();

        assert!(store.contains(&id));
        let read_bytes = store.get(id).unwrap();
        assert_eq!(read_bytes, payload);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_object_store_bitflip_corruption_detection() {
        let temp_dir = std::env::temp_dir().join("origin_test_objstore_corruption");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = ObjectStore::new(&temp_dir).unwrap();
        let payload = b"uncorrupted_canonical_payload";
        let id = store.put(ObjectKind::Evidence, payload).unwrap();

        // Mutate single bit on disk to simulate media corruption
        let path = store.object_path(&id);
        let mut bytes = fs::read(&path).unwrap();
        bytes[0] ^= 0x01; // Corrupt first byte!
        fs::write(&path, &bytes).unwrap();

        // Reading corrupted file MUST fail with CorruptedObject error!
        let result = store.get(id);
        match result {
            Err(StoreError::CorruptedObject { expected, .. }) => assert_eq!(expected, id),
            other => panic!("Expected StoreError::CorruptedObject, got {:?}", other),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_object_store_partial_crash_tmp_cleanup() {
        let temp_dir = std::env::temp_dir().join("origin_test_objstore_partial");
        let _ = fs::remove_dir_all(&temp_dir);

        let store = ObjectStore::new(&temp_dir).unwrap();
        let dummy_id = ORID::compute(ObjectKind::Artifact, b"dummy_crash");

        // Simulate crash mid-write by creating a dangling .tmp file
        let tmp_path = store.tmp_path(&dummy_id);
        fs::write(&tmp_path, b"partial_corrupt_data").unwrap();

        // The partial tmp object MUST NOT be visible as a valid object in the store
        assert!(!store.contains(&dummy_id));
        assert!(matches!(
            store.get(dummy_id),
            Err(StoreError::ObjectNotFound(_))
        ));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
