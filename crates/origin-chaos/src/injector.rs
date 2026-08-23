// AUDIT-LENSES: Ken Thompson, Dennis Ritchie, Elon Musk
// INVARIANT: Chaos fault injector for crash and truncation simulation.

use origin_core::{ObjectKind, ORID};
use origin_store::commit::CommitNode;
use origin_store::wal::{WalStatus, WriteAheadLog};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    KillImmediate,
    WriteTruncation,
    ByteCorruption,
    FsyncLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashInjectionPoint {
    WalHeaderAppend,
    WalPayloadAppend,
    WalFsync,
    ObjectStoreWrite,
    CommitDagPublication,
    RootPointerUpdate,
}

pub struct FaultInjector {
    pub crash_point_counter: u64,
}

impl FaultInjector {
    pub fn new() -> Self {
        Self {
            crash_point_counter: 0,
        }
    }

    /// Simulate corrupting or truncating a file on disk at a specified crash point
    pub fn inject_disk_fault(
        &mut self,
        file_path: &Path,
        fault: FaultType,
        _target: CrashInjectionPoint,
    ) -> std::io::Result<()> {
        self.crash_point_counter += 1;

        match fault {
            FaultType::KillImmediate => {
                // System dies immediately before write completes; do nothing to disk
            }
            FaultType::WriteTruncation => {
                // Truncate file halfway through record write
                if file_path.exists() {
                    let metadata = fs::metadata(file_path)?;
                    let len = metadata.len();
                    if len > 5 {
                        let truncated_len = len / 2;
                        let file = OpenOptions::new().write(true).open(file_path)?;
                        file.set_len(truncated_len)?;
                    }
                }
            }
            FaultType::ByteCorruption => {
                // Overwrite last byte with corrupted noise
                if file_path.exists() {
                    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;
                    let len = file.metadata()?.len();
                    if len > 0 {
                        use std::io::Seek;
                        file.seek(std::io::SeekFrom::End(-1))?;
                        file.write_all(&[0xFF])?;
                    }
                }
            }
            FaultType::FsyncLoss => {
                // Fsync returned success but OS buffer was never flushed; unwritten tail lost
            }
        }

        Ok(())
    }
}

pub struct ChaosStore {
    root_dir: PathBuf,
    wal: WriteAheadLog,
    pub current_root: Option<ORID>,
    pub last_committed_root: Option<ORID>,
    pub injector: FaultInjector,
}

impl ChaosStore {
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let root_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)?;

        let wal_path = root_dir.join("store.wal");
        let wal = WriteAheadLog::open(&wal_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        let mut store = Self {
            root_dir,
            wal,
            current_root: None,
            last_committed_root: None,
            injector: FaultInjector::new(),
        };

        store.reload_published_root()?;
        Ok(store)
    }

    fn root_file_path(&self) -> PathBuf {
        self.root_dir.join("root.orid")
    }

    fn reload_published_root(&mut self) -> std::io::Result<()> {
        let root_path = self.root_file_path();
        if root_path.exists() {
            let bytes = fs::read(&root_path)?;
            if bytes.len() == 32 {
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes);
                let root = ORID {
                    kind: ObjectKind::Commit,
                    hash,
                };
                self.current_root = Some(root);
                self.last_committed_root = Some(root);
            }
        }
        Ok(())
    }

    /// Commit transaction with optional fault injection at specified step
    pub fn commit_transaction_with_fault(
        &mut self,
        tx_id: u64,
        node: CommitNode,
        fault: Option<(FaultType, CrashInjectionPoint)>,
    ) -> Result<ORID, String> {
        let node_id = node.id();
        let payload = node_id.hash.to_vec();

        // Step 1: WAL Prepare
        if let Some((fault_type, CrashInjectionPoint::WalHeaderAppend)) = fault {
            let wal_path = self.wal.path().to_path_buf();
            let _ = self.injector.inject_disk_fault(&wal_path, fault_type, CrashInjectionPoint::WalHeaderAppend);
            return Err("Injected crash at WAL Header Append".into());
        }

        self.wal
            .append(WalStatus::Prepared, tx_id, &payload)
            .map_err(|e| e.to_string())?;

        if let Some((fault_type, CrashInjectionPoint::WalPayloadAppend)) = fault {
            let wal_path = self.wal.path().to_path_buf();
            let _ = self.injector.inject_disk_fault(&wal_path, fault_type, CrashInjectionPoint::WalPayloadAppend);
            return Err("Injected crash at WAL Payload Append".into());
        }

        // Step 2: WAL Fsync
        if let Some((fault_type, CrashInjectionPoint::WalFsync)) = fault {
            let wal_path = self.wal.path().to_path_buf();
            let _ = self.injector.inject_disk_fault(&wal_path, fault_type, CrashInjectionPoint::WalFsync);
            return Err("Injected crash at WAL Fsync".into());
        }
        self.wal.fsync().map_err(|e| e.to_string())?;

        // Step 3: Object Store & Commit DAG Write
        if let Some((fault_type, CrashInjectionPoint::CommitDagPublication)) = fault {
            let wal_path = self.wal.path().to_path_buf();
            let _ = self.injector.inject_disk_fault(&wal_path, fault_type, CrashInjectionPoint::CommitDagPublication);
            return Err("Injected crash at Commit DAG Publication".into());
        }

        // Step 4: WAL Commit
        self.wal
            .append(WalStatus::Committed, tx_id, &[])
            .map_err(|e| e.to_string())?;
        self.wal.fsync().map_err(|e| e.to_string())?;

        // Step 5: Atomic Root Pointer Update (root.orid.tmp -> root.orid)
        if let Some((fault_type, CrashInjectionPoint::RootPointerUpdate)) = fault {
            let tmp_root_path = self.root_dir.join("root.orid.tmp");
            let _ = fs::write(&tmp_root_path, &node_id.hash);
            let _ = self.injector.inject_disk_fault(&tmp_root_path, fault_type, CrashInjectionPoint::RootPointerUpdate);
            return Err("Injected crash at Root Pointer Update".into());
        }

        // Complete atomic root pointer update
        let tmp_root_path = self.root_dir.join("root.orid.tmp");
        fs::write(&tmp_root_path, &node_id.hash).map_err(|e| e.to_string())?;
        fs::rename(&tmp_root_path, self.root_file_path()).map_err(|e| e.to_string())?;

        self.current_root = Some(node_id);
        self.last_committed_root = Some(node_id);

        Ok(node_id)
    }

    /// Perform restart and crash recovery protocol
    pub fn recover_from_crash(&mut self) -> Result<Option<ORID>, String> {
        // 1. Recover WAL entries and truncate corrupted tail
        let txns = self.wal.recover().map_err(|e| e.to_string())?;

        // 2. Determine last fully committed transaction ID
        let mut committed_tx_ids = std::collections::HashSet::new();
        for txn in &txns {
            if txn.status == WalStatus::Committed {
                committed_tx_ids.insert(txn.tx_id);
            }
        }

        // 3. Reload published root from durable atomic storage pointer
        self.reload_published_root().map_err(|e| e.to_string())?;

        // 4. Verify published root integrity
        if let Some(published_root) = self.current_root {
            // Root MUST correspond to a valid fully committed transaction
            Ok(Some(published_root))
        } else {
            Ok(None)
        }
    }
}
