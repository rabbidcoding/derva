// INVARIANT: Write-Ahead Log (WAL) with strict checksum verification, fsync durability, and crash recovery.
// KPI: 0 corrupted records unhandled; 100% idempotent recovery of prepared/committed transactions.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const WAL_MAGIC: u32 = 0x4F57414C; // "OWAL"

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WalStatus {
    Prepared = 1,
    Committed = 2,
    Aborted = 3,
}

impl WalStatus {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(WalStatus::Prepared),
            2 => Some(WalStatus::Committed),
            3 => Some(WalStatus::Aborted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingTxn {
    pub tx_id: u64,
    pub status: WalStatus,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum WalError {
    IoError(std::io::Error),
    CorruptedRecord(String),
    InvalidMagic,
}

impl std::fmt::Display for WalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalError::IoError(e) => write!(f, "WAL I/O error: {}", e),
            WalError::CorruptedRecord(msg) => write!(f, "WAL corrupted record: {}", msg),
            WalError::InvalidMagic => write!(f, "Invalid WAL magic header"),
        }
    }
}

impl std::error::Error for WalError {}

impl From<std::io::Error> for WalError {
    fn from(e: std::io::Error) -> Self {
        WalError::IoError(e)
    }
}

pub struct WriteAheadLog {
    file_path: PathBuf,
    file: File,
}

fn compute_checksum(status: u8, tx_id: u64, payload: &[u8]) -> u32 {
    let mut sum: u32 = status as u32 ^ (tx_id as u32) ^ ((tx_id >> 32) as u32);
    for &b in payload {
        sum = sum.wrapping_add(b as u32).rotate_left(3);
    }
    sum
}

impl WriteAheadLog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, WalError> {
        let file_path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&file_path)?;

        Ok(Self { file_path, file })
    }

    pub fn path(&self) -> &Path {
        &self.file_path
    }

    pub fn append(
        &mut self,
        status: WalStatus,
        tx_id: u64,
        payload: &[u8],
    ) -> Result<(), WalError> {
        self.file.seek(SeekFrom::End(0))?;

        let checksum = compute_checksum(status as u8, tx_id, payload);
        let payload_len = payload.len() as u32;

        self.file.write_all(&WAL_MAGIC.to_be_bytes())?;
        self.file.write_all(&(status as u8).to_be_bytes())?;
        self.file.write_all(&tx_id.to_be_bytes())?;
        self.file.write_all(&payload_len.to_be_bytes())?;
        self.file.write_all(&checksum.to_be_bytes())?;
        self.file.write_all(payload)?;

        Ok(())
    }

    pub fn fsync(&mut self) -> Result<(), WalError> {
        self.file.sync_all()?;
        Ok(())
    }

    pub fn recover(&mut self) -> Result<Vec<PendingTxn>, WalError> {
        self.file.seek(SeekFrom::Start(0))?;

        let mut txns = Vec::new();
        let mut last_valid_pos = 0u64;

        loop {
            let _current_pos = self.file.stream_position()?;
            let mut header_buf = [0u8; 4 + 1 + 8 + 4 + 4]; // 21 bytes header

            match self.file.read_exact(&mut header_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Clean end of file reached
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            let magic = u32::from_be_bytes(header_buf[0..4].try_into().unwrap());
            if magic != WAL_MAGIC {
                // Truncate file at last valid position to discard corrupted tail
                self.file.set_len(last_valid_pos)?;
                break;
            }

            let status_u8 = header_buf[4];
            let status = match WalStatus::from_u8(status_u8) {
                Some(s) => s,
                None => {
                    self.file.set_len(last_valid_pos)?;
                    break;
                }
            };

            let tx_id = u64::from_be_bytes(header_buf[5..13].try_into().unwrap());
            let payload_len = u32::from_be_bytes(header_buf[13..17].try_into().unwrap()) as usize;
            let checksum = u32::from_be_bytes(header_buf[17..21].try_into().unwrap());

            let mut payload = vec![0u8; payload_len];
            if self.file.read_exact(&mut payload).is_err() {
                // Incomplete payload written prior to crash; truncate at last valid pos
                self.file.set_len(last_valid_pos)?;
                break;
            }

            if compute_checksum(status_u8, tx_id, &payload) != checksum {
                // Checksum mismatch; truncate corrupted record
                self.file.set_len(last_valid_pos)?;
                break;
            }

            last_valid_pos = self.file.stream_position()?;
            txns.push(PendingTxn {
                tx_id,
                status,
                payload,
            });
        }

        Ok(txns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wal_append_fsync_and_recover() {
        let temp_path = std::env::temp_dir().join("origin_test_wal_append.wal");
        let _ = fs::remove_file(&temp_path);

        let mut wal = WriteAheadLog::open(&temp_path).unwrap();
        wal.append(WalStatus::Prepared, 1, b"payload_tx_1").unwrap();
        wal.append(WalStatus::Committed, 1, b"").unwrap();
        wal.fsync().unwrap();

        let txns = wal.recover().unwrap();
        assert_eq!(txns.len(), 2);
        assert_eq!(txns[0].tx_id, 1);
        assert_eq!(txns[0].status, WalStatus::Prepared);
        assert_eq!(txns[1].status, WalStatus::Committed);

        let _ = fs::remove_file(&temp_path);
    }

    #[test]
    fn test_wal_corrupted_tail_truncation_on_recovery() {
        let temp_path = std::env::temp_dir().join("origin_test_wal_corrupt.wal");
        let _ = fs::remove_file(&temp_path);

        {
            let mut wal = WriteAheadLog::open(&temp_path).unwrap();
            wal.append(WalStatus::Prepared, 42, b"valid_data").unwrap();
            wal.fsync().unwrap();
        }

        // Append corrupted garbage at tail simulating mid-syscall crash
        {
            let mut f = OpenOptions::new().append(true).open(&temp_path).unwrap();
            f.write_all(b"corrupted_garbage_bytes_tail").unwrap();
        }

        let mut wal = WriteAheadLog::open(&temp_path).unwrap();
        let txns = wal.recover().unwrap();
        assert_eq!(txns.len(), 1);
        assert_eq!(txns[0].tx_id, 42);

        let _ = fs::remove_file(&temp_path);
    }
}
