use std::os::unix::io::RawFd;

/// Source of I/O readiness that can be registered with \[`Scheduler`\].
///
/// Implementors provide access to the underlying file descriptor and a stable
/// identifier used by the scheduler when waiting for readiness events.
pub trait IoSource: Send + Sync {
    /// Return the raw file descriptor associated with this source.
    fn raw_fd(&self) -> RawFd;

    /// Unique identifier for this source.
    fn id(&self) -> u64;
}
