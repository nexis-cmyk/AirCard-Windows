//! Testable framing for the raw payload phase of Apple's StreamingZip service.
//!
//! `AMDServiceConnectionSend` reports the number of bytes accepted by the
//! connection (the Airlift reference uses the same contract).  A positive
//! return value is therefore not necessarily the whole requested buffer.

use anyhow::{Result, bail};

pub const STANDARD_CHUNK_BYTES: usize = 64 * 1024;
pub const COMPATIBILITY_CHUNK_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferProfile {
    Standard,
    Compatibility,
}

impl TransferProfile {
    pub fn chunk_bytes(self) -> usize {
        match self {
            Self::Standard => STANDARD_CHUNK_BYTES,
            Self::Compatibility => COMPATIBILITY_CHUNK_BYTES,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "standard (64 KiB)",
            Self::Compatibility => "compatibility (16 KiB)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferProgress {
    pub bytes_total: usize,
    pub bytes_transferred: usize,
    pub bytes_attempted: usize,
    pub native_result: i32,
    pub chunk_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferMetrics {
    pub bytes_total: usize,
    pub bytes_transferred: usize,
    pub send_calls: usize,
}

/// Small interface around the undocumented Apple DLL call.  Keeping it here
/// lets the partial-write and error paths be verified without an iPhone.
pub trait ArchiveWriter {
    fn send(&mut self, bytes: &[u8]) -> i32;
}

pub fn send_archive<W, F>(
    writer: &mut W,
    archive: &[u8],
    profile: TransferProfile,
    mut on_progress: F,
) -> Result<TransferMetrics>
where
    W: ArchiveWriter,
    F: FnMut(TransferProgress),
{
    let mut transferred = 0usize;
    let mut calls = 0usize;
    let chunk_limit = profile.chunk_bytes();

    while transferred < archive.len() {
        let requested = (archive.len() - transferred).min(chunk_limit);
        let result = writer.send(&archive[transferred..transferred + requested]);
        calls += 1;

        if result <= 0 {
            on_progress(TransferProgress {
                bytes_total: archive.len(),
                bytes_transferred: transferred,
                bytes_attempted: requested,
                native_result: result,
                chunk_bytes: requested,
            });
            bail!(
                "AMDServiceConnectionSend returned {} after {}/{} bytes (attempted {} bytes, chunk limit {})",
                result,
                transferred,
                archive.len(),
                requested,
                chunk_limit,
            );
        }

        let accepted = result as usize;
        if accepted > requested {
            bail!(
                "AMDServiceConnectionSend returned impossible byte count {} for {}-byte request",
                accepted,
                requested,
            );
        }

        transferred += accepted;
        on_progress(TransferProgress {
            bytes_total: archive.len(),
            bytes_transferred: transferred,
            bytes_attempted: requested,
            native_result: result,
            chunk_bytes: requested,
        });
    }

    Ok(TransferMetrics {
        bytes_total: archive.len(),
        bytes_transferred: transferred,
        send_calls: calls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ScriptedWriter {
        results: Vec<i32>,
        calls: usize,
        requested: Vec<usize>,
    }

    impl ArchiveWriter for ScriptedWriter {
        fn send(&mut self, bytes: &[u8]) -> i32 {
            self.requested.push(bytes.len());
            let result = self.results[self.calls];
            self.calls += 1;
            result
        }
    }

    #[test]
    fn chunks_at_the_standard_limit() {
        let mut writer = ScriptedWriter {
            results: vec![STANDARD_CHUNK_BYTES as i32, 7],
            calls: 0,
            requested: Vec::new(),
        };
        let metrics = send_archive(
            &mut writer,
            &vec![0; STANDARD_CHUNK_BYTES + 7],
            TransferProfile::Standard,
            |_| {},
        )
        .unwrap();

        assert_eq!(writer.requested, vec![STANDARD_CHUNK_BYTES, 7]);
        assert_eq!(metrics.bytes_transferred, STANDARD_CHUNK_BYTES + 7);
    }

    #[test]
    fn retries_the_remainder_after_a_partial_write() {
        let mut writer = ScriptedWriter {
            results: vec![10, 6],
            calls: 0,
            requested: Vec::new(),
        };
        let metrics = send_archive(
            &mut writer,
            &[0; 16],
            TransferProfile::Compatibility,
            |_| {},
        )
        .unwrap();

        assert_eq!(writer.requested, vec![16, 6]);
        assert_eq!(metrics.send_calls, 2);
    }

    #[test]
    fn preserves_the_native_failure_code_and_offset() {
        let mut writer = ScriptedWriter {
            results: vec![8, -42],
            calls: 0,
            requested: Vec::new(),
        };
        let error = send_archive(
            &mut writer,
            &[0; 16],
            TransferProfile::Compatibility,
            |_| {},
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("after 8/16 bytes"));
        assert!(error.contains("AMDServiceConnectionSend returned"));
    }
}
