//! Minimal placeholder for Process Activity Log events used in tests.
use crate::task::TaskId;

/// Events emitted to the Process Activity Log.
#[allow(dead_code)]
pub enum TaskEvent {
    /// A task failed due to panic.
    Failed(TaskId),
}

/// Emit a PAL event.
#[allow(dead_code)]
pub fn emit(_event: TaskEvent) {
    // Placeholder implementation
}
